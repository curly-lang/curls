use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};

use curlyc::frontend::ir::{IR};
use curlyc::{check};

use std::collections::HashMap;

use curls::diagnostics::{convert_diagnostics};

#[derive(Debug)]
struct Backend {
    client: Client,
}

impl Backend {
    async fn analyze(&self, filenames: &[(String, bool)], codes: &[String], uri: Url, version: Option<i64>) {
        let mut ir = IR { modules: HashMap::new() };
        let mut diagnostics = Vec::new();

        match check(&filenames, codes, &mut ir, false, false) {
            Ok(_) => {}
            Err((raw_diagnostics, files)) => convert_diagnostics(&raw_diagnostics, &files, &mut diagnostics)
        }

        self.client.publish_diagnostics(uri, diagnostics, version).await;
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            server_info: None,
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(TextDocumentSyncKind::Full)),
                ..ServerCapabilities::default()
            }
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client.log_message(MessageType::Info, "server initialized!").await;
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let content = &params.text_document.text;
        let filenames = vec![(String::from(params.text_document.uri.as_str()), false)];
        self.analyze(&filenames, &vec![content.to_string()], params.text_document.uri, Some(params.text_document.version)).await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let content = &params.content_changes[0].text;
        let filenames = vec![(String::from(params.text_document.uri.as_str()), false)];
        self.analyze(&filenames, &vec![content.to_string()], params.text_document.uri, params.text_document.version).await;
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }
}

#[tokio::main]
async fn main() {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, messages) = LspService::new(|client| Backend { client });
    Server::new(stdin, stdout).interleave(messages).serve(service).await;
}
