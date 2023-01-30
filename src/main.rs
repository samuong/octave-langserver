mod bridge;

use lsp_server::Connection;
use lsp_server::Message;
use lsp_server::RequestId;
use lsp_server::Response;
use lsp_types::DidOpenTextDocumentParams;
use lsp_types::GotoDefinitionParams;

use lsp_types::GotoDefinitionResponse;
use lsp_types::InitializeParams;
use lsp_types::InitializeResult;
use lsp_types::OneOf;
use lsp_types::Position;
use lsp_types::ServerCapabilities;
use lsp_types::ServerInfo;
use lsp_types::TextDocumentSyncKind;
use lsp_types::TextDocumentSyncOptions;

fn main() {
    let (connection, io_threads) = Connection::stdio();
    serve(connection);
    io_threads.join().unwrap();
}

fn serve(connection: Connection) {
    let initialize_result = serde_json::to_value(InitializeResult {
        capabilities: ServerCapabilities {
            definition_provider: Some(OneOf::Left(true)),

            text_document_sync: Some(lsp_types::TextDocumentSyncCapability::Options(
                TextDocumentSyncOptions {
                    open_close: Some(true),
                    change: Some(TextDocumentSyncKind::FULL),
                    will_save: None,
                    will_save_wait_until: None,
                    save: None,
                },
            )),

            ..Default::default()
        },
        server_info: Some(ServerInfo {
            name: String::from("octave-langserver"),
            version: Some(String::from("0.1")),
        }),
    })
    .unwrap();

    {
        bridge::ffi::init().unwrap();
        let (id, params) = connection.initialize_start().unwrap();
        let _initialize_params = serde_json::from_value::<InitializeParams>(params).unwrap();
        connection.initialize_finish(id, initialize_result).unwrap();
    }

    for msg in &connection.receiver {
        match msg {
            // TODO: this requires each handler to convert from a serde_json::Value to ...Params
            // themselves. is there a better way?
            Message::Request(req) if connection.handle_shutdown(&req).unwrap() => break,
            Message::Request(req) => match req.method.as_str() {
                "textDocument/definition" => handle_gotodef(&connection, req.id, req.params),
                _ => (),
            },
            Message::Response(_resp) => {}
            Message::Notification(note) => match note.method.as_str() {
                "textDocument/didOpen" => handle_didopen(note.params),
                _ => (),
            },
        }
    }
}

fn handle_didopen(params: serde_json::Value) {
    let params: DidOpenTextDocumentParams = serde_json::from_value(params).unwrap();
    bridge::ffi::analyse(params.text_document.text.as_str());
}

fn handle_gotodef(connection: &Connection, id: RequestId, params: serde_json::Value) {
    // TODO: return an InvalidParams response here
    let params: GotoDefinitionParams = serde_json::from_value(params).unwrap();
    let pos = &params.text_document_position_params.position;
    // TODO: both of these should return RequestFailed error instead of unwrapping
    let symbol = bridge::ffi::symbol_at(pos.line, pos.character).unwrap();
    let pos = bridge::ffi::definition(symbol.as_str()).unwrap();
    let loc = lsp_types::Location::new(
        params.text_document_position_params.text_document.uri,
        lsp_types::Range::new(
            Position::new(pos[0], pos[1]),
            //Position::new(pos[0], pos[1] + symbol.len() as u32), // dodgy logic
            Position::new(pos[0], pos[1] + 1), // dodgy logic
        ),
    );
    let result = Some(GotoDefinitionResponse::Scalar(loc));
    // TODO: should return a InvalidParams error instead of unwrapping
    let result = serde_json::to_value(&result).unwrap();
    let resp = Response {
        id,
        result: Some(result),
        error: None,
    };
    eprintln!("sending gotoDefinition response: {resp:?}");
    // TODO: should just panic if the connection is broken
    connection.sender.send(Message::Response(resp)).unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;

    use lsp_server::Notification;
    use lsp_server::Request;

    use lsp_types::GotoDefinitionResponse::Scalar;
    use lsp_types::InitializedParams;
    use lsp_types::PartialResultParams;
    use lsp_types::TextDocumentIdentifier;
    use lsp_types::TextDocumentItem;
    use lsp_types::TextDocumentPositionParams;
    use lsp_types::Url;
    use lsp_types::WorkDoneProgressParams;

    use serial_test::serial;

    use std::thread;
    use std::time::Duration;

    // Run an in-process (language server client and server run as two threads, rather than
    // separate processes), in-memory (communication happens over an in-memory channel rather than
    // over stdin/stdout) test.
    #[test]
    #[serial]
    fn test_gotodef() {
        // TODO:
        // - look into https://docs.rs/expect-test/latest/expect_test/
        // - look into https://docs.rs/serde_json/latest/serde_json/#constructing-json-values
        // - automatically clean up after test runs
        // - have common setup/teardown functions
        let (client_side, server_side) = Connection::memory();
        let server_thread = thread::spawn(move || serve(server_side));
        let one_sec = Duration::from_secs(1);

        let init_req = Message::Request(Request {
            id: RequestId::from(1),
            method: "initialize".to_string(),
            params: serde_json::to_value(InitializeParams {
                ..Default::default()
            })
            .unwrap(),
        });
        client_side.sender.send_timeout(init_req, one_sec).unwrap();

        match client_side.receiver.recv_timeout(one_sec).unwrap() {
            Message::Response(init_resp) => {
                assert_eq!(init_resp.id, RequestId::from(1));
                assert!(!init_resp.result.is_none());
                assert!(init_resp.error.is_none());
            }
            msg => panic!("got {msg:?}, wanted a response"),
        }

        let init_note = Message::Notification(Notification {
            method: "initialized".to_string(),
            params: serde_json::to_value(InitializedParams {}).unwrap(),
        });
        client_side.sender.send_timeout(init_note, one_sec).unwrap();

        let did_open_note = Message::Notification(Notification {
            method: "textDocument/didOpen".to_string(),
            params: serde_json::to_value(DidOpenTextDocumentParams {
                text_document: TextDocumentItem {
                    uri: Url::parse("file:///add.m").unwrap(),
                    language_id: "octave".to_string(),
                    version: 1,
                    text: r#"
function sum = add (augend, addend)
    sum = augend + addend;
endfunction
f = @add;
y = f (1, 2);
"#
                    .to_string(),
                },
            })
            .unwrap(),
        });
        client_side
            .sender
            .send_timeout(did_open_note, one_sec)
            .unwrap();

        let goto_def_req = Message::Request(Request {
            id: RequestId::from(2),
            method: "textDocument/definition".to_string(),
            params: serde_json::to_value(GotoDefinitionParams {
                text_document_position_params: TextDocumentPositionParams {
                    text_document: TextDocumentIdentifier::new(
                        Url::parse("file:///add.m").unwrap(),
                    ),
                    position: Position::new(3, 5),
                },
                work_done_progress_params: WorkDoneProgressParams {
                    work_done_token: None,
                },
                partial_result_params: PartialResultParams {
                    partial_result_token: None,
                },
            })
            .unwrap(),
        });
        client_side
            .sender
            .send_timeout(goto_def_req, one_sec)
            .unwrap();

        match client_side.receiver.recv_timeout(one_sec).unwrap() {
            Message::Response(goto_def_resp) => {
                assert_eq!(goto_def_resp.id, RequestId::from(2));
                assert!(goto_def_resp.error.is_none());
                let Some(value) = goto_def_resp.result else {
                    panic!("missing params in goto_def response");
                };
                let Scalar(location) = serde_json::from_value(value).unwrap() else {
                    panic!("it wasn't a scalar");
                };
                assert_eq!(location.uri.to_string(), "file:///add.m");
                assert_eq!(location.range.start, Position::new(0, 0));
            }
            msg => panic!("got {msg:?}, wanted a response"),
        }

        let shutdown_req = Message::Request(Request {
            id: RequestId::from(3),
            method: "shutdown".to_string(),
            params: serde_json::Value::Null,
        });
        client_side
            .sender
            .send_timeout(shutdown_req, one_sec)
            .unwrap();

        match client_side.receiver.recv_timeout(one_sec).unwrap() {
            Message::Response(shutdown_resp) => {
                assert_eq!(shutdown_resp.id, RequestId::from(3));
                assert!(!shutdown_resp.result.is_none());
                assert!(shutdown_resp.error.is_none());
            }
            msg => panic!("got {msg:?}, wanted a response"),
        }

        let exit_note = Message::Notification(Notification {
            method: "exit".to_string(),
            params: serde_json::Value::Null,
        });
        client_side.sender.send_timeout(exit_note, one_sec).unwrap();

        server_thread.join().unwrap();

        bridge::ffi::clear_indexes();
    }
}
