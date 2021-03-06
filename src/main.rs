use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};

use codespan_reporting::diagnostic::{LabelStyle, Severity};
use codespan_reporting::files::{Files};

use curlyc::frontend::ir::{IR};

use std::collections::HashMap;
use std::convert::TryFrom;

use curls::check::{check};

#[derive(Debug)]
struct Backend {
    client: Client,
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
        self.client.log_message(MessageType::Info, "opened").await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        self.client.log_message(MessageType::Info, "changed").await;
        let content = &params.content_changes[0].text;
        let mut ir = IR { modules: HashMap::new() };
        let filenames = vec![(String::from(params.text_document.uri.as_str()), false)];
        let (rawDiagnostics, files) = check(&filenames, &vec![content.to_string()], &mut ir, false);
        let mut diagnostics = Vec::new();
        
        for rawDiagnostic in rawDiagnostics {
            if let Some(label) = rawDiagnostic.labels.iter().find(|label| label.style == LabelStyle::Primary) {
                if let Some(start) = files.location(label.file_id, label.range.start) {
                    if let Some(end) = files.location(label.file_id, label.range.end) {
                        diagnostics.push(Diagnostic {
                            message: rawDiagnostic.message,
                            code: None,
                            range: Range::new(
                                Position::new(u64::try_from(start.line_number - 1).expect("cannot convert usize to u64"), u64::try_from(start.column_number - 1).expect("cannot convert usize to u64")),
                                Position::new(u64::try_from(end.line_number - 1).expect("cannot convert usize to u64"), u64::try_from(end.column_number - 1).expect("cannot convert usize to u64"))),
                            severity: Some(match rawDiagnostic.severity {
                                Severity::Error => DiagnosticSeverity::Error,
                                Severity::Warning => DiagnosticSeverity::Warning,
                                Severity::Note => DiagnosticSeverity::Information,
                                Severity::Help => DiagnosticSeverity::Hint,
                                Severity::Bug => DiagnosticSeverity::Error
                            }),
                            source: Some(String::from("curls")),
                            related_information: None,
                            tags: None,
                        })
                    }
                }
            }
        }

        self.client.publish_diagnostics(params.text_document.uri, diagnostics, params.text_document.version).await;
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
