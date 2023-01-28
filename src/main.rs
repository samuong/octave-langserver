mod bridge;

use lsp_server::Connection;
use lsp_server::Message;
use lsp_server::ProtocolError;
use lsp_server::Request;
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
            Message::Request(req) => match req.method.as_str() {
                "shutdown" => handle_shutdown(&connection, &req).unwrap(),
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

fn handle_shutdown(connection: &Connection, req: &Request) -> Result<(), ProtocolError> {
    let req_is_shutdown = connection.handle_shutdown(req)?;
    debug_assert!(req_is_shutdown);
    Ok(())
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
