use crate::crate_cache::{CrateCache, CrateItem, ItemKind};
use crab_parser::ast::*;
use crab_parser::Parser;
use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::RwLock;
use tower_lsp::lsp_types::*;

pub struct CompletionEngine {
    crate_cache: Arc<CrateCache>,
    stdlib_items: Arc<RwLock<Vec<CrateItem>>>,
    project_crates: Arc<RwLock<Vec<String>>>,
}

impl CompletionEngine {
    pub async fn new(crate_cache: Arc<CrateCache>) -> Self {
        let stdlib_items = Self::build_stdlib_items();

        Self {
            crate_cache,
            stdlib_items: Arc::new(RwLock::new(stdlib_items)),
            project_crates: Arc::new(RwLock::new(vec![])),
        }
    }

    pub fn new_sync(crate_cache: Arc<CrateCache>) -> Self {
        let stdlib_items = Self::build_stdlib_items();

        Self {
            crate_cache,
            stdlib_items: Arc::new(RwLock::new(stdlib_items)),
            project_crates: Arc::new(RwLock::new(vec![])),
        }
    }

    fn build_stdlib_items() -> Vec<CrateItem> {
        vec![
            CrateItem { name: "String".to_string(), kind: ItemKind::Struct, signature: Some("pub struct String".to_string()), docs: Some("A UTF-8 encoded, growable string.".to_string()) },
            CrateItem { name: "Vec".to_string(), kind: ItemKind::Struct, signature: Some("pub struct Vec<T>".to_string()), docs: Some("A contiguous growable array type.".to_string()) },
            CrateItem { name: "Option".to_string(), kind: ItemKind::Enum, signature: Some("pub enum Option<T>".to_string()), docs: Some("The Option type.".to_string()) },
            CrateItem { name: "Result".to_string(), kind: ItemKind::Enum, signature: Some("pub enum Result<T, E>".to_string()), docs: Some("The Result type.".to_string()) },
            CrateItem { name: "HashMap".to_string(), kind: ItemKind::Struct, signature: Some("pub struct HashMap<K, V>".to_string()), docs: Some("A hash map implementation.".to_string()) },
            CrateItem { name: "Box".to_string(), kind: ItemKind::Struct, signature: Some("pub struct Box<T: ?Sized>".to_string()), docs: Some("A pointer type for heap allocation.".to_string()) },
            CrateItem { name: "Rc".to_string(), kind: ItemKind::Struct, signature: Some("pub structRc<T: ?Sized>".to_string()), docs: Some("A single-threaded reference-counting pointer.".to_string()) },
            CrateItem { name: "Arc".to_string(), kind: ItemKind::Struct, signature: Some("pub struct Arc<T: ?Sized>".to_string()), docs: Some("A thread-safe reference-counting pointer.".to_string()) },
            CrateItem { name: "Mutex".to_string(), kind: ItemKind::Struct, signature: Some("pub struct Mutex<T: ?Sized>".to_string()), docs: Some("A mutual exclusion primitive.".to_string()) },
            CrateItem { name: "RwLock".to_string(), kind: ItemKind::Struct, signature: Some("pub struct RwLock<T: ?Sized>".to_string()), docs: Some("A reader-writer lock.".to_string()) },
            CrateItem { name: "Cell".to_string(), kind: ItemKind::Struct, signature: Some("pub struct Cell<T: ?Sized>".to_string()), docs: Some("A mutable memory location.".to_string()) },
            CrateItem { name: "RefCell".to_string(), kind: ItemKind::Struct, signature: Some("pub struct RefCell<T: ?Sized>".to_string()), docs: Some("A mutable memory location with dynamically checked borrow rules.".to_string()) },
            CrateItem { name: "Duration".to_string(), kind: ItemKind::Struct, signature: Some("pub struct Duration".to_string()), docs: Some("A time duration.".to_string()) },
            CrateItem { name: "Instant".to_string(), kind: ItemKind::Struct, signature: Some("pub struct Instant".to_string()), docs: Some("A measurement of a monotonically nondecreasing clock.".to_string()) },
            CrateItem { name: "thread".to_string(), kind: ItemKind::Module, signature: None, docs: Some("Native threading.".to_string()) },
            CrateItem { name: "spawn".to_string(), kind: ItemKind::Function, signature: Some("pub fn spawn<F, T>(f: F) -> JoinHandle<T>".to_string()), docs: Some("Spawns a new thread.".to_string()) },
            CrateItem { name: "sleep".to_string(), kind: ItemKind::Function, signature: Some("pub fn sleep(dur: Duration)".to_string()), docs: Some("Pauses execution.".to_string()) },
            CrateItem { name: "println".to_string(), kind: ItemKind::Macro, signature: Some("macro_rules! println".to_string()), docs: Some("Prints to stdout.".to_string()) },
            CrateItem { name: "format".to_string(), kind: ItemKind::Macro, signature: Some("macro_rules! format".to_string()), docs: Some("Creates a String.".to_string()) },
            CrateItem { name: "vec".to_string(), kind: ItemKind::Macro, signature: Some("macro_rules! vec".to_string()), docs: Some("Creates a Vec.".to_string()) },
            CrateItem { name: "panic".to_string(), kind: ItemKind::Macro, signature: Some("macro_rules! panic".to_string()), docs: Some("Panics the current thread.".to_string()) },
            CrateItem { name: "assert".to_string(), kind: ItemKind::Macro, signature: Some("macro_rules! assert".to_string()), docs: Some("Asserts a condition.".to_string()) },
            CrateItem { name: "assert_eq".to_string(), kind: ItemKind::Macro, signature: Some("macro_rules! assert_eq".to_string()), docs: Some("Asserts equality.".to_string()) },
            CrateItem { name: "todo".to_string(), kind: ItemKind::Macro, signature: Some("macro_rules! todo".to_string()), docs: Some("Indicates unfinished code.".to_string()) },
            CrateItem { name: "unimplemented".to_string(), kind: ItemKind::Macro, signature: Some("macro_rules! unimplemented".to_string()), docs: Some("Indicates unimplemented code.".to_string()) },
            CrateItem { name: "drop".to_string(), kind: ItemKind::Function, signature: Some("pub fn drop<T>(_x: T)".to_string()), docs: Some("Explicitly drop a value.".to_string()) },
            CrateItem { name: "clone".to_string(), kind: ItemKind::Function, signature: Some("pub fn clone<T: Clone>(t: &T) -> T".to_string()), docs: Some("Clone a value.".to_string()) },
            CrateItem { name: "mem".to_string(), kind: ItemKind::Module, signature: None, docs: Some("Memory manipulation.".to_string()) },
            CrateItem { name: "ptr".to_string(), kind: ItemKind::Module, signature: None, docs: Some("Raw pointers.".to_string()) },
            CrateItem { name: "slice".to_string(), kind: ItemKind::Module, signature: None, docs: Some("Slice utilities.".to_string()) },
            CrateItem { name: "str".to_string(), kind: ItemKind::Module, signature: None, docs: Some("String utilities.".to_string()) },
            CrateItem { name: "char".to_string(), kind: ItemKind::Module, signature: None, docs: Some("Character utilities.".to_string()) },
            CrateItem { name: "iter".to_string(), kind: ItemKind::Module, signature: None, docs: Some("Iterator utilities.".to_string()) },
            CrateItem { name: "cmp".to_string(), kind: ItemKind::Module, signature: None, docs: Some("Comparison utilities.".to_string()) },
            CrateItem { name: "convert".to_string(), kind: ItemKind::Module, signature: None, docs: Some("Type conversions.".to_string()) },
            CrateItem { name: "default".to_string(), kind: ItemKind::Module, signature: None, docs: Some("Default values.".to_string()) },
            CrateItem { name: "ops".to_string(), kind: ItemKind::Module, signature: None, docs: Some("Operators.".to_string()) },
            CrateItem { name: "marker".to_string(), kind: ItemKind::Module, signature: None, docs: Some("Marker traits.".to_string()) },
            CrateItem { name: "any".to_string(), kind: ItemKind::Module, signature: None, docs: Some("Any trait.".to_string()) },
            CrateItem { name: "borrow".to_string(), kind: ItemKind::Module, signature: None, docs: Some("Borrow trait.".to_string()) },
            CrateItem { name: "fmt".to_string(), kind: ItemKind::Module, signature: None, docs: Some("Formatting.".to_string()) },
            CrateItem { name: "io".to_string(), kind: ItemKind::Module, signature: None, docs: Some("I/O utilities.".to_string()) },
            CrateItem { name: "fs".to_string(), kind: ItemKind::Module, signature: None, docs: Some("Filesystem.".to_string()) },
            CrateItem { name: "net".to_string(), kind: ItemKind::Module, signature: None, docs: Some("Networking.".to_string()) },
            CrateItem { name: "env".to_string(), kind: ItemKind::Module, signature: None, docs: Some("Environment.".to_string()) },
            CrateItem { name: "process".to_string(), kind: ItemKind::Module, signature: None, docs: Some("Process management.".to_string()) },
            CrateItem { name: "sync".to_string(), kind: ItemKind::Module, signature: None, docs: Some("Synchronization.".to_string()) },
            CrateItem { name: "time".to_string(), kind: ItemKind::Module, signature: None, docs: Some("Time utilities.".to_string()) },
            CrateItem { name: "pin".to_string(), kind: ItemKind::Module, signature: None, docs: Some("Pinning.".to_string()) },
            CrateItem { name: "future".to_string(), kind: ItemKind::Module, signature: None, docs: Some("Future utilities.".to_string()) },
            CrateItem { name: "task".to_string(), kind: ItemKind::Module, signature: None, docs: Some("Task management.".to_string()) },
            CrateItem { name: "collections".to_string(), kind: ItemKind::Module, signature: None, docs: Some("Collection types.".to_string()) },
            CrateItem { name: "hash".to_string(), kind: ItemKind::Module, signature: None, docs: Some("Hashing.".to_string()) },
            CrateItem { name: "ffi".to_string(), kind: ItemKind::Module, signature: None, docs: Some("Foreign Function Interface.".to_string()) },
            CrateItem { name: "os".to_string(), kind: ItemKind::Module, signature: None, docs: Some("OS-specific.".to_string()) },
            CrateItem { name: "path".to_string(), kind: ItemKind::Module, signature: None, docs: Some("Path manipulation.".to_string()) },
        ]
    }

    pub async fn update_project_crates(&self, source: &str) {
        let mut crates = vec![];

        if let Ok(mut parser) = Parser::new(source) {
            if let Ok(program) = parser.parse() {
                for item in &program.items {
                    if let TopLevelItem::Import(import) = item {
                        let path = &import.path;
                        if path.starts_with("crate:") {
                            let crate_name = path.trim_start_matches("crate:").split('/').next().unwrap_or("");
                            if !crate_name.is_empty() {
                                crates.push(crate_name.to_string());
                                let _ = self.crate_cache.get_crate_info(crate_name).await;
                            }
                        }
                    }
                }
            }
        }

        let mut project_crates = self.project_crates.write().await;
        *project_crates = crates;
    }

    pub async fn get_completions(&self, source: &str, line: usize, character: usize) -> Vec<CompletionItem> {
        let mut items = vec![];
        let (prefix, context) = self.get_completion_context(source, line, character);

        items.extend(self.get_keyword_completions(&prefix));

        if context == CompletionContext::Import {
            items.extend(self.get_import_completions(&prefix).await);
        } else if context == CompletionContext::MethodCall {
            items.extend(self.get_method_completions(&prefix).await);
        } else if context == CompletionContext::Type {
            items.extend(self.get_type_completions(&prefix).await);
        } else {
            items.extend(self.get_general_completions(&prefix).await);
        }

        items
    }

    fn get_completion_context(&self, source: &str, line: usize, character: usize) -> (String, CompletionContext) {
        let lines: Vec<&str> = source.lines().collect();
        if line >= lines.len() {
            return (String::new(), CompletionContext::General);
        }

        let current_line = lines[line];
        let before_cursor = &current_line[..character.min(current_line.len())];

        let prefix = before_cursor
            .split_whitespace()
            .last()
            .unwrap_or("")
            .trim_matches(|c: char| !c.is_alphanumeric() && c != '_')
            .to_string();

        let trimmed = before_cursor.trim_start();
        if trimmed.starts_with("import ") || trimmed.starts_with("use ") {
            return (prefix, CompletionContext::Import);
        }

        if before_cursor.contains('.') {
            let parts: Vec<&str> = before_cursor.split('.').collect();
            if parts.len() >= 2 {
                let obj = parts[parts.len() - 2].trim();
                if !obj.is_empty() {
                    return (obj.to_string(), CompletionContext::MethodCall);
                }
            }
        }

        let prev_line = if line > 0 { lines[line - 1] } else { "" };
        if trimmed.is_empty() && (prev_line.contains(": ") || prev_line.contains("-> ")) {
            return (prefix, CompletionContext::Type);
        }

        if before_cursor.contains(": ") || before_cursor.contains("-> ") {
            return (prefix, CompletionContext::Type);
        }

        (prefix, CompletionContext::General)
    }

    fn get_keyword_completions(&self, _prefix: &str) -> Vec<CompletionItem> {
        let keywords = vec![
            ("var", "Declare a mutable variable", Some("var ")),
            ("final", "Declare an immutable variable", Some("final ")),
            ("const", "Declare a compile-time constant", Some("const ")),
            ("async", "Mark function as asynchronous", None),
            ("await", "Await async operation", Some("await ")),
            ("class", "Define a class", Some("class ")),
            ("interface", "Define an interface", Some("interface ")),
            ("mixin", "Define a mixin", Some("mixin ")),
            ("extends", "Specify parent class", Some("extends ")),
            ("implements", "Implement interface", Some("implements ")),
            ("with", "Apply mixin", Some("with ")),
            ("if", "Conditional statement", Some("if () {\n  \n}")),
            ("else", "Else branch", Some("else {\n  \n}")),
            ("for", "For loop", Some("for (var i = 0; i < ; i++) {\n  \n}")),
            ("while", "While loop", Some("while () {\n  \n}")),
            ("do", "Do-while loop", Some("do {\n  \n} while ();")),
            ("switch", "Switch statement", Some("switch () {\n  case :\n    break;\n}")),
            ("try", "Try-catch block", Some("try {\n  \n} catch (e) {\n  \n}")),
            ("catch", "Catch block", Some("catch (e) {\n  \n}")),
            ("finally", "Finally block", Some("finally {\n  \n}")),
            ("throw", "Throw exception", Some("throw ")),
            ("return", "Return from function", Some("return ")),
            ("break", "Break from loop", Some("break;")),
            ("continue", "Continue to next iteration", Some("continue;")),
            ("import", "Import module", Some("import '';")),
            ("export", "Export module", Some("export '';")),
            ("as", "Type cast or alias", Some("as ")),
            ("is", "Type check", Some("is ")),
            ("in", "Membership check", Some("in ")),
            ("new", "Create instance", Some("new ")),
            ("this", "Current instance", Some("this.")),
            ("super", "Parent class", Some("super.")),
            ("null", "Null value", Some("null")),
            ("true", "Boolean true", Some("true")),
            ("false", "Boolean false", Some("false")),
            ("void", "No return type", None),
            ("int", "Integer type", None),
            ("double", "Double type", None),
            ("bool", "Boolean type", None),
            ("String", "String type", None),
            ("dynamic", "Dynamic type", None),
            ("never", "Never type", None),
        ];

        keywords.into_iter().map(|(label, detail, insert)| {
            let mut item = CompletionItem {
                label: label.to_string(),
                kind: Some(CompletionItemKind::KEYWORD),
                detail: Some(detail.to_string()),
                ..Default::default()
            };
            if let Some(text) = insert {
                item.insert_text = Some(text.to_string());
                if text.contains('\n') {
                    item.insert_text_format = Some(InsertTextFormat::SNIPPET);
                }
            }
            item
        }).collect()
    }

    async fn get_import_completions(&self, prefix: &str) -> Vec<CompletionItem> {
        let mut items = vec![];

        let popular_crates = vec![
            "tokio", "serde", "reqwest", "axum", "clap", "anyhow", "thiserror",
            "chrono", "regex", "lazy_static", "dashmap", "parking_lot", "tracing",
            "hyper", "actix-web", "rocket", "warp", "tonic", "prost", "sqlx",
            "diesel", "sea-orm", "mongodb", "redis", "kafka", "rabbitmq",
            "rayon", "crossbeam", "parking_lot", "async-trait", "futures",
            "pin-project", "bytes", "uuid", "rand", "sha2", "base64", "hex",
            "serde_json", "toml", "yaml-rust", "csv", "reqwest", "hyper",
            "native-tls", "rustls", "jsonwebtoken", "argon2", "bcrypt",
            "image", "plotters", "ndarray", "nalgebra", "statrs", "rustfft",
            "petgraph", "indexmap", "hashbrown", "smallvec", "arrayvec",
            "compact_str", "smartstring", "camino", "path-absolutize",
        ];

        for name in &popular_crates {
            if name.starts_with(prefix) {
                items.push(CompletionItem {
                    label: format!("crate:{}", name),
                    kind: Some(CompletionItemKind::MODULE),
                    detail: Some(format!("Crates.io crate: {}", name)),
                    insert_text: Some(format!("crate:{};", name)),
                    ..Default::default()
                });
            }
        }

        if prefix.len() >= 2 {
            if let Ok(search_results) = self.crate_cache.search_crates(prefix).await {
                for crate_info in search_results {
                    let name = &crate_info.name;
                    if !items.iter().any(|i| i.label == format!("crate:{}", name)) {
                        items.push(CompletionItem {
                            label: format!("crate:{}", name),
                            kind: Some(CompletionItemKind::MODULE),
                            detail: Some(format!(
                                "{} - {}",
                                crate_info.version,
                                crate_info.description.as_ref().unwrap_or(&"No description".to_string())
                            )),
                            insert_text: Some(format!("crate:{};", name)),
                            documentation: crate_info.description.map(|d| {
                                Documentation::MarkupContent(MarkupContent {
                                    kind: MarkupKind::Markdown,
                                    value: d,
                                })
                            }),
                            ..Default::default()
                        });
                    }
                }
            }
        }

        items
    }

    async fn get_method_completions(&self, object: &str) -> Vec<CompletionItem> {
        let mut items = vec![];

        let project_crates = self.project_crates.read().await;
        for crate_name in project_crates.iter() {
            let crate_items = self.crate_cache.get_crate_items(crate_name).await;
            for item in crate_items {
                items.push(self.crate_item_to_completion(&item, Some(crate_name)));
            }
        }

        let stdlib = self.stdlib_items.read().await;
        let object_lower = object.to_lowercase();

        for item in stdlib.iter() {
            let item_lower = item.name.to_lowercase();
            if item_lower.starts_with(&object_lower) || object_lower.contains(&item_lower) {
                items.push(self.crate_item_to_completion(item, None));
            }
        }

        if object_lower == "string" || object_lower == "str" {
            items.extend(vec![
                CompletionItem {
                    label: "len".to_string(),
                    kind: Some(CompletionItemKind::METHOD),
                    detail: Some("Returns the length of the string".to_string()),
                    ..Default::default()
                },
                CompletionItem {
                    label: "is_empty".to_string(),
                    kind: Some(CompletionItemKind::METHOD),
                    detail: Some("Returns true if the string is empty".to_string()),
                    ..Default::default()
                },
                CompletionItem {
                    label: "push".to_string(),
                    kind: Some(CompletionItemKind::METHOD),
                    detail: Some("Appends a character to the end".to_string()),
                    ..Default::default()
                },
                CompletionItem {
                    label: "push_str".to_string(),
                    kind: Some(CompletionItemKind::METHOD),
                    detail: Some("Appends a string slice".to_string()),
                    ..Default::default()
                },
                CompletionItem {
                    label: "contains".to_string(),
                    kind: Some(CompletionItemKind::METHOD),
                    detail: Some("Returns true if contains substring".to_string()),
                    ..Default::default()
                },
                CompletionItem {
                    label: "starts_with".to_string(),
                    kind: Some(CompletionItemKind::METHOD),
                    detail: Some("Returns true if starts with prefix".to_string()),
                    ..Default::default()
                },
                CompletionItem {
                    label: "ends_with".to_string(),
                    kind: Some(CompletionItemKind::METHOD),
                    detail: Some("Returns true if ends with suffix".to_string()),
                    ..Default::default()
                },
                CompletionItem {
                    label: "split".to_string(),
                    kind: Some(CompletionItemKind::METHOD),
                    detail: Some("Splits string by pattern".to_string()),
                    ..Default::default()
                },
                CompletionItem {
                    label: "trim".to_string(),
                    kind: Some(CompletionItemKind::METHOD),
                    detail: Some("Returns trimmed string".to_string()),
                    ..Default::default()
                },
                CompletionItem {
                    label: "to_string".to_string(),
                    kind: Some(CompletionItemKind::METHOD),
                    detail: Some("Converts to String".to_string()),
                    ..Default::default()
                },
                CompletionItem {
                    label: "parse".to_string(),
                    kind: Some(CompletionItemKind::METHOD),
                    detail: Some("Parses string to type".to_string()),
                    ..Default::default()
                },
            ]);
        }

        if object_lower == "vec" || object_lower == "list" {
            items.extend(vec![
                CompletionItem {
                    label: "len".to_string(),
                    kind: Some(CompletionItemKind::METHOD),
                    detail: Some("Returns the number of elements".to_string()),
                    ..Default::default()
                },
                CompletionItem {
                    label: "is_empty".to_string(),
                    kind: Some(CompletionItemKind::METHOD),
                    detail: Some("Returns true if empty".to_string()),
                    ..Default::default()
                },
                CompletionItem {
                    label: "push".to_string(),
                    kind: Some(CompletionItemKind::METHOD),
                    detail: Some("Appends element to the end".to_string()),
                    ..Default::default()
                },
                CompletionItem {
                    label: "pop".to_string(),
                    kind: Some(CompletionItemKind::METHOD),
                    detail: Some("Removes and returns last element".to_string()),
                    ..Default::default()
                },
                CompletionItem {
                    label: "get".to_string(),
                    kind: Some(CompletionItemKind::METHOD),
                    detail: Some("Returns element at index".to_string()),
                    ..Default::default()
                },
                CompletionItem {
                    label: "insert".to_string(),
                    kind: Some(CompletionItemKind::METHOD),
                    detail: Some("Inserts element at index".to_string()),
                    ..Default::default()
                },
                CompletionItem {
                    label: "remove".to_string(),
                    kind: Some(CompletionItemKind::METHOD),
                    detail: Some("Removes element at index".to_string()),
                    ..Default::default()
                },
                CompletionItem {
                    label: "clear".to_string(),
                    kind: Some(CompletionItemKind::METHOD),
                    detail: Some("Clears all elements".to_string()),
                    ..Default::default()
                },
                CompletionItem {
                    label: "iter".to_string(),
                    kind: Some(CompletionItemKind::METHOD),
                    detail: Some("Returns iterator".to_string()),
                    ..Default::default()
                },
                CompletionItem {
                    label: "contains".to_string(),
                    kind: Some(CompletionItemKind::METHOD),
                    detail: Some("Returns true if contains element".to_string()),
                    ..Default::default()
                },
            ]);
        }

        if object_lower == "option" {
            items.extend(vec![
                CompletionItem {
                    label: "is_some".to_string(),
                    kind: Some(CompletionItemKind::METHOD),
                    detail: Some("Returns true if Some".to_string()),
                    ..Default::default()
                },
                CompletionItem {
                    label: "is_none".to_string(),
                    kind: Some(CompletionItemKind::METHOD),
                    detail: Some("Returns true if None".to_string()),
                    ..Default::default()
                },
                CompletionItem {
                    label: "unwrap".to_string(),
                    kind: Some(CompletionItemKind::METHOD),
                    detail: Some("Returns contained value".to_string()),
                    ..Default::default()
                },
                CompletionItem {
                    label: "unwrap_or".to_string(),
                    kind: Some(CompletionItemKind::METHOD),
                    detail: Some("Returns value or default".to_string()),
                    ..Default::default()
                },
                CompletionItem {
                    label: "map".to_string(),
                    kind: Some(CompletionItemKind::METHOD),
                    detail: Some("Maps Option to another".to_string()),
                    ..Default::default()
                },
                CompletionItem {
                    label: "and_then".to_string(),
                    kind: Some(CompletionItemKind::METHOD),
                    detail: Some("Chaining operation".to_string()),
                    ..Default::default()
                },
                CompletionItem {
                    label: "ok_or".to_string(),
                    kind: Some(CompletionItemKind::METHOD),
                    detail: Some("Converts to Result".to_string()),
                    ..Default::default()
                },
            ]);
        }

        if object_lower == "result" {
            items.extend(vec![
                CompletionItem {
                    label: "is_ok".to_string(),
                    kind: Some(CompletionItemKind::METHOD),
                    detail: Some("Returns true if Ok".to_string()),
                    ..Default::default()
                },
                CompletionItem {
                    label: "is_err".to_string(),
                    kind: Some(CompletionItemKind::METHOD),
                    detail: Some("Returns true if Err".to_string()),
                    ..Default::default()
                },
                CompletionItem {
                    label: "unwrap".to_string(),
                    kind: Some(CompletionItemKind::METHOD),
                    detail: Some("Returns Ok value or panics".to_string()),
                    ..Default::default()
                },
                CompletionItem {
                    label: "unwrap_err".to_string(),
                    kind: Some(CompletionItemKind::METHOD),
                    detail: Some("Returns Err value".to_string()),
                    ..Default::default()
                },
                CompletionItem {
                    label: "unwrap_or".to_string(),
                    kind: Some(CompletionItemKind::METHOD),
                    detail: Some("Returns value or default".to_string()),
                    ..Default::default()
                },
                CompletionItem {
                    label: "map".to_string(),
                    kind: Some(CompletionItemKind::METHOD),
                    detail: Some("Maps Ok value".to_string()),
                    ..Default::default()
                },
                CompletionItem {
                    label: "map_err".to_string(),
                    kind: Some(CompletionItemKind::METHOD),
                    detail: Some("Maps Err value".to_string()),
                    ..Default::default()
                },
                CompletionItem {
                    label: "and_then".to_string(),
                    kind: Some(CompletionItemKind::METHOD),
                    detail: Some("Chaining for Ok".to_string()),
                    ..Default::default()
                },
                CompletionItem {
                    label: "or_else".to_string(),
                    kind: Some(CompletionItemKind::METHOD),
                    detail: Some("Chaining for Err".to_string()),
                    ..Default::default()
                },
            ]);
        }

        items
    }

    async fn get_type_completions(&self, prefix: &str) -> Vec<CompletionItem> {
        let mut items = vec![];

        let primitives = vec![
            ("int", "64-bit signed integer"),
            ("double", "64-bit floating point"),
            ("bool", "Boolean type"),
            ("String", "UTF-8 encoded string"),
            ("void", "No return type"),
            ("dynamic", "Dynamic type"),
            ("never", "Never type"),
        ];

        for (name, desc) in primitives {
            if name.to_lowercase().starts_with(&prefix.to_lowercase()) || prefix.is_empty() {
                items.push(CompletionItem {
                    label: name.to_string(),
                    kind: Some(CompletionItemKind::TYPE_PARAMETER),
                    detail: Some(desc.to_string()),
                    ..Default::default()
                });
            }
        }

        let stdlib = self.stdlib_items.read().await;
        for item in stdlib.iter() {
            if item.name.to_lowercase().starts_with(&prefix.to_lowercase()) {
                match item.kind {
                    ItemKind::Struct | ItemKind::Enum | ItemKind::Trait | ItemKind::Type => {
                        items.push(self.crate_item_to_completion(item, None));
                    }
                    _ => {}
                }
            }
        }

        items
    }

    async fn get_general_completions(&self, prefix: &str) -> Vec<CompletionItem> {
        let mut items = vec![];

        let stdlib = self.stdlib_items.read().await;
        for item in stdlib.iter() {
            if prefix.is_empty() || item.name.to_lowercase().starts_with(&prefix.to_lowercase()) {
                items.push(self.crate_item_to_completion(item, None));
            }
        }

        let project_crates = self.project_crates.read().await;
        for crate_name in project_crates.iter() {
            let crate_items = self.crate_cache.get_crate_items(crate_name).await;
            for item in crate_items {
                if prefix.is_empty() || item.name.to_lowercase().starts_with(&prefix.to_lowercase()) {
                    items.push(self.crate_item_to_completion(&item, Some(crate_name)));
                }
            }
        }

        items
    }

    fn crate_item_to_completion(&self, item: &CrateItem, crate_name: Option<&str>) -> CompletionItem {
        let kind = match item.kind {
            ItemKind::Function => CompletionItemKind::FUNCTION,
            ItemKind::Struct => CompletionItemKind::STRUCT,
            ItemKind::Enum => CompletionItemKind::ENUM,
            ItemKind::Trait => CompletionItemKind::INTERFACE,
            ItemKind::Type => CompletionItemKind::TYPE_PARAMETER,
            ItemKind::Const => CompletionItemKind::CONSTANT,
            ItemKind::Static => CompletionItemKind::CONSTANT,
            ItemKind::Macro => CompletionItemKind::FUNCTION,
            ItemKind::Module => CompletionItemKind::MODULE,
        };

        let label = if let Some(c) = crate_name {
            format!("{}::{} ({})", c, item.name, Self::kind_to_str(&item.kind))
        } else {
            item.name.clone()
        };

        CompletionItem {
            label,
            kind: Some(kind),
            detail: item.signature.clone(),
            documentation: item.docs.as_ref().map(|d| {
                Documentation::MarkupContent(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: d.clone(),
                })
            }),
            ..Default::default()
        }
    }

    fn kind_to_str(kind: &ItemKind) -> &'static str {
        match kind {
            ItemKind::Function => "fn",
            ItemKind::Struct => "struct",
            ItemKind::Enum => "enum",
            ItemKind::Trait => "trait",
            ItemKind::Type => "type",
            ItemKind::Const => "const",
            ItemKind::Static => "static",
            ItemKind::Macro => "macro",
            ItemKind::Module => "mod",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
enum CompletionContext {
    General,
    Import,
    MethodCall,
    Type,
}
