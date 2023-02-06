mod bridge;

use lsp_server::Connection;
use lsp_server::ErrorCode;
use lsp_server::Message;
use lsp_server::RequestId;
use lsp_server::Response;

use lsp_types::DidOpenTextDocumentParams;
use lsp_types::GotoDefinitionParams;
use lsp_types::InitializeParams;
use lsp_types::InitializeResult;
use lsp_types::OneOf;
use lsp_types::Position;
use lsp_types::ServerCapabilities;
use lsp_types::ServerInfo;
use lsp_types::TextDocumentSyncKind;
use lsp_types::TextDocumentSyncOptions;

use std::error::Error;
use std::fmt::Display;
use std::fmt::Formatter;
use std::time::Instant;

#[derive(Debug)]
struct HandlerError {
    code: ErrorCode,
    message: String,
}

impl HandlerError {
    fn invalid_params(err: impl Error) -> HandlerError {
        HandlerError {
            code: ErrorCode::InvalidParams,
            message: err.to_string(),
        }
    }

    fn request_failed(err: impl Error) -> HandlerError {
        HandlerError {
            code: ErrorCode::RequestFailed,
            message: err.to_string(),
        }
    }

    fn into_response(self, id: RequestId) -> Response {
        Response::new_err(id, self.code as i32, self.message)
    }
}

impl Display for HandlerError {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "{}: {}", self.code as i32, self.message)
    }
}

impl Error for HandlerError {}

fn main() {
    let (connection, io_threads) = Connection::stdio();
    serve(connection, None);
    io_threads.join().unwrap();
}

fn serve(connection: Connection, deadline: Option<Instant>) {
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

    loop {
        let msg = match deadline {
            None => connection.receiver.recv().unwrap(),
            Some(d) => connection.receiver.recv_deadline(d).unwrap(),
        };
        match msg {
            // TODO: this requires each handler to convert from a serde_json::Value to ...Params
            // themselves. is there a better way?
            Message::Request(req) if connection.handle_shutdown(&req).unwrap() => break,
            Message::Request(req) => {
                let resp = match req.method.as_str() {
                    "textDocument/definition" => handle_gotodef(req.id.clone(), req.params),
                    _ => {
                        continue;
                    }
                };
                let r = match resp {
                    Ok(r) => r,
                    Err(err) => err.into_response(req.id.clone()),
                };
                connection.sender.send(Message::Response(r)).unwrap();
            }
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

fn handle_gotodef(id: RequestId, parameters: serde_json::Value) -> Result<Response, HandlerError> {
    let params: GotoDefinitionParams =
        serde_json::from_value(parameters).map_err(HandlerError::invalid_params)?;
    let pos = params.text_document_position_params.position;
    let symbol =
        bridge::ffi::symbol_at(pos.line, pos.character).map_err(HandlerError::request_failed)?;
    let pos = bridge::ffi::definition(symbol.as_str()).map_err(HandlerError::request_failed)?;
    let start = Position::new(pos[0], pos[1]);
    let end = Position::new(pos[0], pos[1] + 1); //dodgy logic
    let loc = lsp_types::Location::new(
        params.text_document_position_params.text_document.uri,
        lsp_types::Range::new(start, end),
    );
    Ok(Response {
        id,
        result: Some(serde_json::to_value(loc).unwrap()),
        error: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    use lsp_server::Notification;
    use lsp_server::Request;

    use lsp_types::GotoDefinitionResponse::Scalar;

    use serde_json::json;

    use serial_test::serial;

    use std::thread;
    use std::thread::JoinHandle;
    use std::time::Duration;

    struct TestFixture {
        next_id: i32,
        connection: Connection,
        server_thread: Option<JoinHandle<()>>,
    }

    impl TestFixture {
        fn new() -> TestFixture {
            let (client_side, server_side) = Connection::memory();
            let deadline = Instant::now() + Duration::from_secs(5);
            let server_thread = thread::spawn(move || serve(server_side, Some(deadline)));
            TestFixture {
                next_id: 0,
                connection: client_side,
                server_thread: Some(server_thread),
            }
        }

        fn request(&mut self, method: &str, params: serde_json::Value) {
            self.next_id += 1;
            let req = Message::Request(Request {
                id: RequestId::from(self.next_id),
                method: method.to_string(),
                params,
            });
            self.connection
                .sender
                .send_timeout(req, Duration::from_secs(1))
                .unwrap();
        }

        fn notification(&mut self, method: &str, params: serde_json::Value) {
            let note = Message::Notification(Notification {
                method: method.to_string(),
                params,
            });
            self.connection
                .sender
                .send_timeout(note, Duration::from_secs(1))
                .unwrap();
        }

        fn response(&mut self) -> Message {
            self.connection
                .receiver
                .recv_timeout(Duration::from_secs(1))
                .unwrap()
        }

        fn setup(&mut self) {
            self.request(&"initialize", json!({"capabilities": {}}));

            match self.response() {
                Message::Response(init_resp) => {
                    assert_eq!(init_resp.id, RequestId::from(1));
                    assert!(!init_resp.result.is_none());
                    assert!(init_resp.error.is_none());
                }
                msg => panic!("got {msg:?}, wanted a response"),
            }

            self.notification("initialized", json!({}));
        }

        fn teardown(&mut self) {
            self.request("shutdown", serde_json::Value::Null);

            match self.response() {
                Message::Response(shutdown_resp) => {
                    assert_eq!(shutdown_resp.id, RequestId::from(3));
                    assert!(!shutdown_resp.result.is_none());
                    assert!(shutdown_resp.error.is_none());
                }
                msg => panic!("got {msg:?}, wanted a response"),
            }

            self.notification("exit", serde_json::Value::Null);
        }
    }

    impl Drop for TestFixture {
        fn drop(&mut self) {
            self.server_thread.take().map(JoinHandle::join);
            bridge::ffi::clear_indexes();
        }
    }

    // Run an in-process (language server client and server run as two threads, rather than
    // separate processes), in-memory (communication happens over an in-memory channel rather than
    // over stdin/stdout) test.
    #[test]
    #[serial]
    fn test_gotodef() {
        let mut test = TestFixture::new();
        test.setup();

        test.notification(
            "textDocument/didOpen",
            json!({
                "textDocument": {
                    "uri": "file:///inc.m",
                    "languageId": "octave",
                    "version": 1,
                    "text": "function y = inc (x)\ny = x + 1;\nendfunction\ninc (0);\n",
                }
            }),
        );

        test.request(
            "textDocument/definition",
            json!({
                "textDocument": {"uri": "file:///inc.m"},
                "position": {"line": 3, "character": 0},
            }),
        );

        match test.response() {
            Message::Response(goto_def_resp) => {
                assert_eq!(goto_def_resp.id, RequestId::from(2));
                assert!(goto_def_resp.error.is_none());
                let Some(value) = goto_def_resp.result else {
                    panic!("missing params in goto_def response");
                };
                let Scalar(location) = serde_json::from_value(value).unwrap() else {
                    panic!("it wasn't a scalar");
                };
                assert_eq!(location.uri.to_string(), "file:///inc.m");
                assert_eq!(location.range.start, Position::new(0, 0));
            }
            msg => panic!("got {msg:?}, wanted a response"),
        }

        test.teardown();
    }
}
