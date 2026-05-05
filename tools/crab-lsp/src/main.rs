mod crate_cache;
mod completion;

use crate::crate_cache::CrateCache;
use crate::completion::CompletionEngine;
use crab_parser::{Parser, Program};
use dashmap::DashMap;
use ropey::Rope;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct CrabSettings {
    max_number_of_problems: i32,
}

impl Default for CrabSettings {
    fn default() -> Self {
        CrabSettings {
            max_number_of_problems: 100,
        }
    }
}

struct Backend {
    client: Client,
    document_map: Arc<DashMap<Url, Rope>>,
    ast_map: Arc<DashMap<Url, Program>>,
    completion_engine: Arc<CompletionEngine>,
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            server_info: Some(ServerInfo {
                name: "crab-lsp".to_string(),
                version: Some("0.1.0".to_string()),
            }),
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                definition_provider: Some(OneOf::Left(true)),
                diagnostic_provider: Some(DiagnosticServerCapabilities::Options(
                    DiagnosticOptions {
                        identifier: Some("crab".to_string()),
                        inter_file_dependencies: true,
                        workspace_diagnostics: false,
                        work_done_progress_options: WorkDoneProgressOptions::default(),
                    },
                )),
                document_formatting_provider: Some(OneOf::Left(true)),
                document_highlight_provider: Some(OneOf::Left(true)),
                completion_provider: Some(CompletionOptions {
                    resolve_provider: Some(false),
                    trigger_characters: Some(vec![".".to_string(), ":".to_string()]),
                    all_commit_characters: None,
                    work_done_progress_options: WorkDoneProgressOptions::default(),
                    completion_item: None,
                }),
                ..Default::default()
            },
            ..Default::default()
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "Crab language server initialized")
            .await;
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let uri = params.text_document.uri;
        let text = params.text_document.text;
        let rope = Rope::from_str(&text);
        self.document_map.insert(uri.clone(), rope);
        self.validate_document(uri).await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let uri = params.text_document.uri;
        if let Some(mut rope) = self.document_map.get_mut(&uri) {
            for change in params.content_changes {
                if let Some(range) = change.range {
                    let start_line = range.start.line as usize;
                    let start_char = range.start.character as usize;
                    let end_line = range.end.line as usize;
                    let end_char = range.end.character as usize;
                    let start_idx = rope.line_to_char(start_line) + start_char;
                    let end_idx = rope.line_to_char(end_line) + end_char;
                    rope.remove(start_idx..end_idx);
                    rope.insert(start_idx, &change.text);
                } else {
                    *rope = Rope::from_str(&change.text);
                }
            }
        }
        self.validate_document(uri).await;
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        self.document_map.remove(&params.text_document.uri);
        self.ast_map.remove(&params.text_document.uri);
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        let uri = params.text_document_position_params.text_document.uri;
        let position = params.text_document_position_params.position;

        if let Some(rope) = self.document_map.get(&uri) {
            let line = position.line as usize;
            let character = position.character as usize;
            let char_idx = rope.line_to_char(line) + character;
            let word = self.get_word_at_position(&rope, char_idx);

            let contents = match word.as_str() {
                "var" => HoverContents::Markup(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: "**var** - Declare a mutable variable\n\nExample: `var x = 10;`".to_string(),
                }),
                "final" => HoverContents::Markup(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: "**final** - Declare an immutable variable\n\nExample: `final x = 10;`".to_string(),
                }),
                "int" => HoverContents::Markup(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: "**int** - 64-bit signed integer type\n\nEquivalent to Rust's `i64`".to_string(),
                }),
                "double" => HoverContents::Markup(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: "**double** - 64-bit floating point type\n\nEquivalent to Rust's `f64`".to_string(),
                }),
                "bool" => HoverContents::Markup(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: "**bool** - Boolean type\n\nValues: `true`, `false`".to_string(),
                }),
                "String" => HoverContents::Markup(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: "**String** - UTF-8 encoded string type\n\nEquivalent to Rust's `String`".to_string(),
                }),
                "void" => HoverContents::Markup(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: "**void** - No return value\n\nEquivalent to Rust's `()`".to_string(),
                }),
                "async" => HoverContents::Markup(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: "**async** - Mark a function as asynchronous\n\nCompiles to Rust async/await".to_string(),
                }),
                "await" => HoverContents::Markup(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: "**await** - Await an asynchronous operation\n\nExample: `var result = await fetch();`".to_string(),
                }),
                "class" => HoverContents::Markup(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: "**class** - Define a class\n\nCompiles to Rust struct with impl blocks".to_string(),
                }),
                "if" | "else" => HoverContents::Markup(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: format!("**{}** - Conditional branching", word).to_string(),
                }),
                "for" | "while" => HoverContents::Markup(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: format!("**{}** - Loop construct", word).to_string(),
                }),
                "return" => HoverContents::Markup(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: "**return** - Return from function\n\nExample: `return value;`".to_string(),
                }),
                "try" | "catch" | "finally" => HoverContents::Markup(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: format!("**{}** - Exception handling", word).to_string(),
                }),
                "import" => HoverContents::Markup(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: "**import** - Import a module\n\nExample: `import 'package:foo/bar.dart';`".to_string(),
                }),
                _ => HoverContents::Markup(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: format!("**{}**", word),
                }),
            };

            return Ok(Some(Hover {
                contents,
                range: None,
            }));
        }

        Ok(None)
    }

    async fn goto_definition(
        &self,
        params: GotoDefinitionParams,
    ) -> Result<Option<GotoDefinitionResponse>> {
        let uri = params.text_document_position_params.text_document.uri;
        let position = params.text_document_position_params.position;

        if let Some(rope) = self.document_map.get(&uri) {
            let line = position.line as usize;
            let character = position.character as usize;
            let char_idx = rope.line_to_char(line) + character;
            let word = self.get_word_at_position(&rope, char_idx);

            if let Some(ast) = self.ast_map.get(&uri) {
                if let Some(location) = self.find_definition_in_ast(&word, &ast, position) {
                    return Ok(Some(GotoDefinitionResponse::Scalar(location)));
                }
            }
        }

        Ok(None)
    }

    async fn formatting(&self, params: DocumentFormattingParams) -> Result<Option<Vec<TextEdit>>> {
        let uri = params.text_document.uri;

        if let Some(rope) = self.document_map.get(&uri) {
            let text = rope.to_string();
            let formatted = self.format_crab_code(&text);

            if formatted != text {
                let lines = rope.len_lines() as u32;
                let last_line_len = rope.line(lines as usize - 1).len_chars() as u32;

                return Ok(Some(vec![TextEdit {
                    range: Range {
                        start: Position::new(0, 0),
                        end: Position::new(lines - 1, last_line_len),
                    },
                    new_text: formatted,
                }]));
            }
        }

        Ok(None)
    }

    async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
        let uri = params.text_document_position.text_document.uri;
        let position = params.text_document_position.position;

        if let Some(rope) = self.document_map.get(&uri) {
            let text = rope.to_string();
            let line = position.line as usize;
            let character = position.character as usize;

            let items = self.completion_engine.get_completions(&text, line, character).await;
            return Ok(Some(CompletionResponse::Array(items)));
        }

        Ok(Some(CompletionResponse::Array(vec![])))
    }

}

impl Backend {
    fn new(client: Client) -> Self {
        // Initialize synchronously - async init will happen on first use
        let crate_cache = Arc::new(CrateCache::new_sync());
        let completion_engine = Arc::new(CompletionEngine::new_sync(crate_cache.clone()));

        Backend {
            client,
            document_map: Arc::new(DashMap::new()),
            ast_map: Arc::new(DashMap::new()),
            completion_engine,
        }
    }

    fn get_word_at_position(&self, rope: &Rope, char_idx: usize) -> String {
        let line_idx = rope.char_to_line(char_idx);
        let line = rope.line(line_idx);
        let line_text: String = line.chars().collect();
        let col = char_idx - rope.line_to_char(line_idx);

        let mut start = col;
        let mut end = col;

        while start > 0 {
            let c = line_text.chars().nth(start - 1).unwrap_or(' ');
            if c.is_alphanumeric() || c == '_' {
                start -= 1;
            } else {
                break;
            }
        }

        while end < line_text.len() {
            let c = line_text.chars().nth(end).unwrap_or(' ');
            if c.is_alphanumeric() || c == '_' {
                end += 1;
            } else {
                break;
            }
        }

        line_text[start..end].to_string()
    }

    async fn validate_document(&self, uri: Url) {
        if let Some(rope) = self.document_map.get(&uri) {
            let text = rope.to_string();

            self.completion_engine.update_project_crates(&text).await;

            match Parser::new(&text) {
                Ok(mut parser) => {
                    match parser.parse() {
                        Ok(program) => {
                            self.ast_map.insert(uri.clone(), program);
                            let diagnostics = vec![];
                            self.client
                                .publish_diagnostics(uri, diagnostics, None)
                                .await;
                        }
                        Err(e) => {
                            let diagnostic = Diagnostic {
                                range: Range {
                                    start: Position::new(0, 0),
                                    end: Position::new(0, 0),
                                },
                                severity: Some(DiagnosticSeverity::ERROR),
                                code: None,
                                code_description: None,
                                source: Some("crab-lsp".to_string()),
                                message: format!("Parse error: {}", e),
                                related_information: None,
                                tags: None,
                                data: None,
                            };
                            self.client
                                .publish_diagnostics(uri, vec![diagnostic], None)
                                .await;
                        }
                    }
                }
                Err(e) => {
                    let diagnostic = Diagnostic {
                        range: Range {
                            start: Position::new(0, 0),
                            end: Position::new(0, 0),
                        },
                        severity: Some(DiagnosticSeverity::ERROR),
                        code: None,
                        code_description: None,
                        source: Some("crab-lsp".to_string()),
                        message: format!("Lexer error: {}", e),
                        related_information: None,
                        tags: None,
                        data: None,
                    };
                    self.client
                        .publish_diagnostics(uri, vec![diagnostic], None)
                        .await;
                }
            }
        }
    }

    async fn get_diagnostics(&self, uri: &Url) -> Vec<Diagnostic> {
        if let Some(rope) = self.document_map.get(uri) {
            let text = rope.to_string();

            match Parser::new(&text) {
                Ok(mut parser) => {
                    match parser.parse() {
                        Ok(_) => vec![],
                        Err(e) => vec![Diagnostic {
                            range: Range {
                                start: Position::new(0, 0),
                                end: Position::new(0, 0),
                            },
                            severity: Some(DiagnosticSeverity::ERROR),
                            code: None,
                            code_description: None,
                            source: Some("crab-lsp".to_string()),
                            message: format!("Parse error: {}", e),
                            related_information: None,
                            tags: None,
                            data: None,
                        }],
                    }
                }
                Err(e) => vec![Diagnostic {
                    range: Range {
                        start: Position::new(0, 0),
                        end: Position::new(0, 0),
                    },
                    severity: Some(DiagnosticSeverity::ERROR),
                    code: None,
                    code_description: None,
                    source: Some("crab-lsp".to_string()),
                    message: format!("Lexer error: {}", e),
                    related_information: None,
                    tags: None,
                    data: None,
                }],
            }
        } else {
            vec![]
        }
    }

    fn find_definition_in_ast(
        &self,
        _word: &str,
        _program: &Program,
        _position: Position,
    ) -> Option<Location> {
        None
    }

    fn format_crab_code(&self, code: &str) -> String {
        let mut result = String::new();
        let mut indent_level = 0;
        let mut in_string = false;
        let mut prev_char = ' ';

        for line in code.lines() {
            let trimmed = line.trim();

            if trimmed.is_empty() {
                result.push('\n');
                continue;
            }

            if trimmed.starts_with('}') || trimmed.starts_with(')') || trimmed.starts_with(']') {
                if indent_level > 0 {
                    indent_level -= 1;
                }
            }

            for _ in 0..indent_level {
                result.push_str("  ");
            }

            for ch in trimmed.chars() {
                if ch == '"' && prev_char != '\\' {
                    in_string = !in_string;
                }

                if !in_string {
                    match ch {
                        '{' | '(' | '[' => indent_level += 1,
                        '}' | ')' | ']' => {
                            if indent_level > 0 {
                                indent_level -= 1;
                            }
                        }
                        _ => {}
                    }
                }

                result.push(ch);
                prev_char = ch;
            }

            result.push('\n');
        }

        result.trim_end().to_string() + "\n"
    }
}

#[tokio::main]
async fn main() {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::new(|client| Backend::new(client));
    Server::new(stdin, stdout, socket).serve(service).await;
}

