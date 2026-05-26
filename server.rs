use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer};
use tokio::sync::RwLock;
use crate::document::DocumentStore;

pub struct Backend {
    client: Client,
    document_store: RwLock<DocumentStore>,
}

impl Backend {
    pub fn new(client: Client) -> Self {
        Self {
            client,
            document_store: RwLock::new(DocumentStore::new()),
        }
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(TextDocumentSyncKind::FULL)),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                completion_provider: Some(CompletionOptions {
                    resolve_provider: Some(false),
                    trigger_characters: Some(vec![".".to_string()]),
                    ..Default::default()
                }),
                definition_provider: Some(OneOf::Left(true)),
                document_formatting_provider: Some(OneOf::Left(true)),
                ..Default::default()
            },
            ..Default::default()
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "Finix Language Server initialized!")
            .await;
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let mut store = self.document_store.write().await;
        store.update_document(params.text_document.uri.clone(), params.text_document.text.clone());
        self.client
            .log_message(MessageType::INFO, format!("Opened file: {}", params.text_document.uri))
            .await;
        
        // TODO: Hook up Parser and TypeChecker here, and publish diagnostics!
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let mut store = self.document_store.write().await;
        if let Some(change) = params.content_changes.into_iter().next() {
            store.update_document(params.text_document.uri.clone(), change.text);
        }
        // TODO: Re-run parser pipeline and send updated diagnostics
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        let uri = params.text_document_position_params.text_document.uri;
        let position = params.text_document_position_params.position;
        
        let store = self.document_store.read().await;
        if let Some(_doc) = store.get_document(&uri) {
            // Placeholder: A real LSP maps the Position to an AST Node, and queries the TypeEnvironment!
            let hover_contents = HoverContents::Scalar(MarkedString::LanguageString(LanguageString {
                language: "finix".to_string(),
                value: format!("Finix symbol hovered at Line: {}, Char: {}", position.line, position.character),
            }));
            
            return Ok(Some(Hover {
                contents: hover_contents,
                range: None,
            }));
        }
        
        Ok(None)
    }

    async fn completion(&self, _params: CompletionParams) -> Result<Option<CompletionResponse>> {
        let completions = vec![
            CompletionItem {
                label: "print".to_string(),
                kind: Some(CompletionItemKind::FUNCTION),
                detail: Some("Print to standard output".to_string()),
                ..Default::default()
            },
            CompletionItem {
                label: "let".to_string(),
                kind: Some(CompletionItemKind::KEYWORD),
                ..Default::default()
            },
        ];
        
        Ok(Some(CompletionResponse::Array(completions)))
    }

    async fn goto_definition(&self, _params: GotoDefinitionParams) -> Result<Option<GotoDefinitionResponse>> {
        Ok(None) // TODO: Lookup symbol in environment using mapped AST node position
    }
    
    async fn formatting(&self, _params: DocumentFormattingParams) -> Result<Option<Vec<TextEdit>>> {
        Ok(None) // TODO: Format AST tree via ast::printer
    }
}