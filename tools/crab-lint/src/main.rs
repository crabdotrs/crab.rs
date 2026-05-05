use anyhow::{Context, Result};
use clap::Parser;
use colored::Colorize;
use crab_lexer::Lexer;
use crab_parser::{ast::*, Parser as CrabParser};
use std::fs;
use std::io::{self, Read};
use std::path::{Path, PathBuf};

#[derive(Parser)]
#[command(name = "crab-lint")]
#[command(about = "Lint Crab source code")]
struct Cli {
    #[arg(help = "Input file or directory to lint")]
    input: Option<PathBuf>,

    #[arg(short, long, help = "Show all issues including warnings")]
    all: bool,

    #[arg(short, long, help = "Use stdin")]
    stdin: bool,

    #[arg(short, long, help = "Exit with error code on warnings too")]
    strict: bool,
}

#[derive(Debug, Clone)]
enum Severity {
    Error,
    Warning,
    Info,
}

#[derive(Debug, Clone)]
struct LintIssue {
    severity: Severity,
    message: String,
    line: usize,
    column: usize,
    rule: String,
}

struct Linter {
    issues: Vec<LintIssue>,
    check_style: bool,
}

impl Linter {
    fn new(check_style: bool) -> Self {
        Linter {
            issues: Vec::new(),
            check_style,
        }
    }

    fn add_issue(
        &mut self,
        severity: Severity,
        message: &str,
        line: usize,
        column: usize,
        rule: &str,
    ) {
        self.issues.push(LintIssue {
            severity,
            message: message.to_string(),
            line,
            column,
            rule: rule.to_string(),
        });
    }

    fn lint(&mut self, source: &str, program: &Program) -> Result<()> {
        self.check_empty_blocks(program);
        self.check_unused_imports(program);
        self.check_naming_conventions(program);
        self.check_null_safety(program);
        self.check_async_patterns(program);
        self.check_style_rules(source)?;
        Ok(())
    }

    fn check_empty_blocks(&mut self, program: &Program) {
        for item in &program.items {
            self.check_item_empty_blocks(item);
        }
    }

    fn check_item_empty_blocks(&mut self, item: &TopLevelItem) {
        match item {
            TopLevelItem::FunctionDecl(func) => {
                if let FunctionBody::Block(stmts) = &func.body {
                    if stmts.is_empty() && func.name != "main" {
                        self.add_issue(
                            Severity::Warning,
                            &format!("Function '{}' has an empty body", func.name),
                            0,
                            0,
                            "empty-function",
                        );
                    }
                }
            }
            TopLevelItem::ClassDecl(class) => {
                if class.fields.is_empty()
                    && class.methods.is_empty()
                    && class.constructors.is_empty()
                {
                    self.add_issue(
                        Severity::Warning,
                        &format!("Class '{}' is empty", class.name),
                        0,
                        0,
                        "empty-class",
                    );
                }
                for method in &class.methods {
                    if let FunctionBody::Block(stmts) = &method.body {
                        if stmts.is_empty() {
                            self.add_issue(
                                Severity::Warning,
                                &format!(
                                    "Method '{}' in class '{}' has an empty body",
                                    method.name, class.name
                                ),
                                0,
                                0,
                                "empty-method",
                            );
                        }
                    }
                }
            }
            _ => {}
        }
    }

    fn check_unused_imports(&mut self, program: &Program) {
        let imports: Vec<_> = program
            .items
            .iter()
            .filter_map(|item| {
                if let TopLevelItem::Import(imp) = item {
                    Some(imp.path.clone())
                } else {
                    None
                }
            })
            .collect();

        if imports.is_empty() {
            return;
        }

        let source = format!("{:?}", program).to_lowercase();

        for import in imports {
            let parts: Vec<_> = import.split('/').collect();
            let full_import = import.clone();
            let last_part = parts.last().map(|s| *s).unwrap_or(&full_import);
            let name = last_part.trim_end_matches(".crab");

            if !source.contains(&name.to_lowercase()) {
                self.add_issue(
                    Severity::Warning,
                    &format!("Potentially unused import: '{}'", import),
                    0,
                    0,
                    "unused-import",
                );
            }
        }
    }

    fn check_naming_conventions(&mut self, program: &Program) {
        for item in &program.items {
            match item {
                TopLevelItem::ClassDecl(class) => {
                    if !class
                        .name
                        .chars()
                        .next()
                        .map(|c| c.is_uppercase())
                        .unwrap_or(false)
                    {
                        self.add_issue(
                            Severity::Warning,
                            &format!("Class '{}' should use PascalCase", class.name),
                            0,
                            0,
                            "naming-class",
                        );
                    }
                    for field in &class.fields {
                        if !field.name.starts_with('_')
                            && !field
                                .name
                                .chars()
                                .next()
                                .map(|c| c.is_lowercase())
                                .unwrap_or(false)
                        {
                            self.add_issue(
                                Severity::Warning,
                                &format!(
                                    "Field '{}' in class '{}' should use camelCase",
                                    field.name, class.name
                                ),
                                0,
                                0,
                                "naming-field",
                            );
                        }
                    }
                }
                TopLevelItem::FunctionDecl(func) => {
                    if func
                        .name
                        .chars()
                        .next()
                        .map(|c| c.is_uppercase())
                        .unwrap_or(false)
                    {
                        self.add_issue(
                            Severity::Warning,
                            &format!("Function '{}' should use camelCase", func.name),
                            0,
                            0,
                            "naming-function",
                        );
                    }
                }
                _ => {}
            }
        }
    }

    fn check_null_safety(&mut self, program: &Program) {
        let source = format!("{:?}", program);
        if source.contains("null") {
            self.add_issue(
                Severity::Info,
                "Consider using nullable types (Type?) instead of null where possible",
                0,
                0,
                "null-safety",
            );
        }
    }

    fn check_async_patterns(&mut self, _program: &Program) {}

    fn check_style_rules(&mut self, source: &str) -> Result<()> {
        if !self.check_style {
            return Ok(());
        }

        let lines: Vec<_> = source.lines().collect();
        let mut line_number = 1;

        for line in &lines {
            if line.len() > 100 {
                self.add_issue(
                    Severity::Warning,
                    "Line exceeds 100 characters",
                    line_number,
                    100,
                    "line-length",
                );
            }

            if line.ends_with(' ') || line.ends_with('\t') {
                self.add_issue(
                    Severity::Warning,
                    "Trailing whitespace detected",
                    line_number,
                    line.len(),
                    "trailing-whitespace",
                );
            }

            if line.contains("  ") && !line.trim().is_empty() {
                self.add_issue(
                    Severity::Info,
                    "Multiple consecutive spaces found (consider formatting)",
                    line_number,
                    0,
                    "multiple-spaces",
                );
            }

            line_number += 1;
        }

        Ok(())
    }

    fn report(&self) {
        let mut errors = 0;
        let mut warnings = 0;
        let mut infos = 0;

        for issue in &self.issues {
            let severity_str = match &issue.severity {
                Severity::Error => {
                    errors += 1;
                    "error".red().bold()
                }
                Severity::Warning => {
                    warnings += 1;
                    "warning".yellow().bold()
                }
                Severity::Info => {
                    infos += 1;
                    "info".blue().bold()
                }
            };

            println!(
                "[{}] {} at {}:{} [{}]",
                severity_str,
                issue.message,
                issue.line,
                issue.column,
                issue.rule.dimmed()
            );
        }

        println!();
        println!(
            "Found {} {}, {} {}, and {} {}",
            errors,
            if errors == 1 { "error" } else { "errors" },
            warnings,
            if warnings == 1 { "warning" } else { "warnings" },
            infos,
            if infos == 1 { "info" } else { "infos" }
        );
    }

    fn has_errors(&self) -> bool {
        self.issues
            .iter()
            .any(|i| matches!(i.severity, Severity::Error))
    }

    fn has_warnings(&self) -> bool {
        self.issues
            .iter()
            .any(|i| matches!(i.severity, Severity::Warning))
    }
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let source = if cli.stdin {
        let mut input = String::new();
        io::stdin().read_to_string(&mut input)?;
        input
    } else {
        let input_path = cli.input.unwrap_or_else(|| PathBuf::from("."));
        if input_path.is_file() {
            fs::read_to_string(&input_path)?
        } else {
            anyhow::bail!("Directory linting not yet implemented. Please specify a file.");
        }
    };

    let mut parser = CrabParser::new(&source)?;
    let program = parser.parse()?;

    let mut linter = Linter::new(cli.all);
    linter.lint(&source, &program)?;
    linter.report();

    if linter.has_errors() || (cli.strict && linter.has_warnings()) {
        std::process::exit(1);
    }

    Ok(())
}
