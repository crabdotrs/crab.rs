use anyhow::{Context, Result};
use clap::{Parser as ClapParser, Subcommand};
use crab_lexer::Lexer;
use std::fs;
use std::io::{self, Read};
use std::path::{Path, PathBuf};

#[derive(ClapParser)]
#[command(name = "crab-fmt")]
#[command(about = "Format Crab source code")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    #[arg(help = "Input file or directory to format")]
    input: Option<PathBuf>,

    #[arg(short, long, help = "Write changes to files in place")]
    write: bool,

    #[arg(short, long, help = "Check if files are formatted")]
    check: bool,

    #[arg(short, long, help = "Use stdin/stdout")]
    stdin: bool,
}

#[derive(Subcommand)]
enum Commands {
    #[command(about = "Format a file or directory")]
    Format {
        #[arg(help = "Input file or directory")]
        input: PathBuf,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    if cli.stdin {
        let mut input = String::new();
        io::stdin().read_to_string(&mut input)?;
        let formatted = format_code(&input)?;
        print!("{}", formatted);
        return Ok(());
    }

    let input_path = cli.input.unwrap_or_else(|| PathBuf::from("."));

    if input_path.is_file() {
        process_file(&input_path, cli.write, cli.check)?;
    } else if input_path.is_dir() {
        process_directory(&input_path, cli.write, cli.check)?;
    } else {
        anyhow::bail!("Input path does not exist: {}", input_path.display());
    }

    Ok(())
}

fn process_file(path: &Path, write: bool, check: bool) -> Result<()> {
    let content =
        fs::read_to_string(path).with_context(|| format!("Failed to read {}", path.display()))?;

    let formatted = format_code(&content)?;

    if check {
        if content != formatted {
            println!("Would reformat {}", path.display());
            std::process::exit(1);
        }
    } else if write {
        if content != formatted {
            fs::write(path, formatted)
                .with_context(|| format!("Failed to write {}", path.display()))?;
            println!("Formatted {}", path.display());
        }
    } else {
        print!("{}", formatted);
    }

    Ok(())
}

fn process_directory(dir: &Path, write: bool, check: bool) -> Result<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() && path.extension().map(|e| e == "crab").unwrap_or(false) {
            process_file(&path, write, check)?;
        } else if path.is_dir() {
            process_directory(&path, write, check)?;
        }
    }

    Ok(())
}

struct Formatter {
    indent_level: usize,
    indent_size: usize,
    output: String,
}

impl Formatter {
    fn new() -> Self {
        Formatter {
            indent_level: 0,
            indent_size: 2,
            output: String::new(),
        }
    }

    fn indent(&self) -> String {
        " ".repeat(self.indent_level * self.indent_size)
    }

    fn write_indent(&mut self) {
        self.output.push_str(&self.indent());
    }

    fn write(&mut self, s: &str) {
        self.output.push_str(s);
    }

    fn writeln(&mut self, s: &str) {
        self.write(s);
        self.output.push('\n');
    }
}

fn format_code(source: &str) -> Result<String> {
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize()?;

    let mut formatter = Formatter::new();
    let mut i = 0;
    let mut prev_token = None;

    while i < tokens.len() {
        let token = &tokens[i];

        match token {
            crab_lexer::Token::LeftBrace => {
                formatter.write(" {");
                formatter.indent_level += 1;
                formatter.writeln("");
            }
            crab_lexer::Token::RightBrace => {
                if formatter.indent_level > 0 {
                    formatter.indent_level -= 1;
                }
                formatter.write_indent();
                formatter.write("}");
                if i + 1 < tokens.len() && !matches!(tokens[i + 1], crab_lexer::Token::Semicolon) {
                    formatter.writeln("");
                }
            }
            crab_lexer::Token::Semicolon => {
                formatter.write(";");
                formatter.writeln("");
            }
            crab_lexer::Token::Comma => {
                formatter.write(", ");
            }
            crab_lexer::Token::Var
            | crab_lexer::Token::Final
            | crab_lexer::Token::Const
            | crab_lexer::Token::Class
            | crab_lexer::Token::Abstract
            | crab_lexer::Token::Sealed
            | crab_lexer::Token::Base
            | crab_lexer::Token::Interface
            | crab_lexer::Token::Mixin
            | crab_lexer::Token::Extension
            | crab_lexer::Token::Import
            | crab_lexer::Token::Export
            | crab_lexer::Token::Async
            | crab_lexer::Token::Sync
            | crab_lexer::Token::Static
            | crab_lexer::Token::If
            | crab_lexer::Token::Else
            | crab_lexer::Token::For
            | crab_lexer::Token::While
            | crab_lexer::Token::Do
            | crab_lexer::Token::Switch
            | crab_lexer::Token::Try
            | crab_lexer::Token::Catch
            | crab_lexer::Token::Finally
            | crab_lexer::Token::Return
            | crab_lexer::Token::Break
            | crab_lexer::Token::Continue
            | crab_lexer::Token::Throw
            | crab_lexer::Token::New
            | crab_lexer::Token::This
            | crab_lexer::Token::Super
            | crab_lexer::Token::Get
            | crab_lexer::Token::Set
            | crab_lexer::Token::Factory
            | crab_lexer::Token::Required
            | crab_lexer::Token::Override
            | crab_lexer::Token::Typedef
            | crab_lexer::Token::Is
            | crab_lexer::Token::As
            | crab_lexer::Token::In
            | crab_lexer::Token::On
            | crab_lexer::Token::With
            | crab_lexer::Token::Show
            | crab_lexer::Token::Hide
            | crab_lexer::Token::Deferred
            | crab_lexer::Token::Defer
            | crab_lexer::Token::CBlock
            | crab_lexer::Token::Unsafe
            | crab_lexer::Token::Enum
            | crab_lexer::Token::Part
            | crab_lexer::Token::PartOf
            | crab_lexer::Token::Library
            | crab_lexer::Token::Yield
            | crab_lexer::Token::Await => {
                formatter.write_indent();
                let kw = format!("{:?}", token).to_lowercase();
                let kw = match kw.as_str() {
                    "var" => "var",
                    "final" => "final",
                    "const" => "const",
                    "class" => "class",
                    "abstract" => "abstract",
                    "sealed" => "sealed",
                    "base" => "base",
                    "interface" => "interface",
                    "mixin" => "mixin",
                    "extension" => "extension",
                    "import" => "import",
                    "export" => "export",
                    "async" => "async",
                    "sync" => "sync",
                    "static" => "static",
                    "if" => "if",
                    "else" => "else",
                    "for" => "for",
                    "while" => "while",
                    "do" => "do",
                    "switch" => "switch",
                    "try" => "try",
                    "catch" => "catch",
                    "finally" => "finally",
                    "return" => "return",
                    "break" => "break",
                    "continue" => "continue",
                    "throw" => "throw",
                    "new" => "new",
                    "this" => "this",
                    "super" => "super",
                    "get" => "get",
                    "set" => "set",
                    "factory" => "factory",
                    "required" => "required",
                    "override" => "override",
                    "typedef" => "typedef",
                    "is" => "is",
                    "as" => "as",
                    "in" => "in",
                    "on" => "on",
                    "with" => "with",
                    "show" => "show",
                    "hide" => "hide",
                    "deferred" => "deferred",
                    "defer" => "defer",
                    "cblock" => "CBlock",
                    "unsafe" => "unsafe",
                    "enum" => "enum",
                    "part" => "part",
                    "partof" => "partOf",
                    "library" => "library",
                    "yield" => "yield",
                    "await" => "await",
                    _ => &kw,
                };
                formatter.write(kw);
                formatter.write(" ");
            }
            crab_lexer::Token::Identifier(s) => {
                if matches!(
                    prev_token,
                    Some(crab_lexer::Token::Semicolon) | Some(crab_lexer::Token::RightBrace) | None
                ) {
                    formatter.write_indent();
                }
                formatter.write(s);
            }
            crab_lexer::Token::IntLiteral(n) => {
                formatter.write(&n.to_string());
            }
            crab_lexer::Token::DoubleLiteral(n) => {
                formatter.write(&n.to_string());
            }
            crab_lexer::Token::StringLiteral(s) => {
                formatter.write("\"");
                formatter.write(s);
                formatter.write("\"");
            }
            crab_lexer::Token::BoolLiteral(b) => {
                formatter.write(if *b { "true" } else { "false" });
            }
            crab_lexer::Token::NullLiteral => {
                formatter.write("null");
            }
            crab_lexer::Token::Null => {
                formatter.write("null");
            }
            crab_lexer::Token::True => formatter.write("true"),
            crab_lexer::Token::False => formatter.write("false"),
            crab_lexer::Token::Int => formatter.write("int"),
            crab_lexer::Token::Double => formatter.write("double"),
            crab_lexer::Token::Bool => formatter.write("bool"),
            crab_lexer::Token::String => formatter.write("String"),
            crab_lexer::Token::Void => formatter.write("void"),
            crab_lexer::Token::Plus => formatter.write(" + "),
            crab_lexer::Token::Minus => formatter.write(" - "),
            crab_lexer::Token::Star => formatter.write(" * "),
            crab_lexer::Token::Slash => formatter.write(" / "),
            crab_lexer::Token::Percent => formatter.write(" % "),
            crab_lexer::Token::Equal => formatter.write(" = "),
            crab_lexer::Token::EqualEqual => formatter.write(" == "),
            crab_lexer::Token::BangEqual => formatter.write(" != "),
            crab_lexer::Token::Less => formatter.write(" < "),
            crab_lexer::Token::Greater => formatter.write(" > "),
            crab_lexer::Token::LessEqual => formatter.write(" <= "),
            crab_lexer::Token::GreaterEqual => formatter.write(" >= "),
            crab_lexer::Token::AmpersandAmpersand => formatter.write(" && "),
            crab_lexer::Token::PipePipe => formatter.write(" || "),
            crab_lexer::Token::Bang => formatter.write("!"),
            crab_lexer::Token::Tilde => formatter.write("~"),
            crab_lexer::Token::Ampersand => formatter.write(" & "),
            crab_lexer::Token::Pipe => formatter.write(" | "),
            crab_lexer::Token::Caret => formatter.write(" ^ "),
            crab_lexer::Token::LeftShift => formatter.write(" << "),
            crab_lexer::Token::RightShift => formatter.write(" >> "),
            crab_lexer::Token::PlusEqual => formatter.write(" += "),
            crab_lexer::Token::MinusEqual => formatter.write(" -= "),
            crab_lexer::Token::StarEqual => formatter.write(" *= "),
            crab_lexer::Token::SlashEqual => formatter.write(" /= "),
            crab_lexer::Token::PercentEqual => formatter.write(" %= "),
            crab_lexer::Token::Question => formatter.write("?"),
            crab_lexer::Token::QuestionDot => formatter.write("?."),
            crab_lexer::Token::QuestionQuestion => formatter.write(" ?? "),
            crab_lexer::Token::QuestionQuestionEqual => formatter.write(" ??= "),
            crab_lexer::Token::DoubleSlash => formatter.write(" ~/ "),
            crab_lexer::Token::Arrow => formatter.write(" => "),
            crab_lexer::Token::DoubleArrow => formatter.write(" => "),
            crab_lexer::Token::Ellipsis => formatter.write("..."),
            crab_lexer::Token::LeftParen => formatter.write("("),
            crab_lexer::Token::RightParen => formatter.write(")"),
            crab_lexer::Token::LeftBracket => formatter.write("["),
            crab_lexer::Token::RightBracket => formatter.write("]"),
            crab_lexer::Token::Colon => formatter.write(":"),
            crab_lexer::Token::Dot => formatter.write("."),
            crab_lexer::Token::At => formatter.write("@"),
            crab_lexer::Token::Hash => formatter.write("#"),
            crab_lexer::Token::DoubleStar => formatter.write(" ** "),
            crab_lexer::Token::Case => formatter.write("case "),
            crab_lexer::Token::Default => formatter.write("default "),
            crab_lexer::Token::Extends => formatter.write("extends "),
            crab_lexer::Token::Implements => formatter.write("implements "),
            crab_lexer::Token::Function => formatter.write("Function"),
            crab_lexer::Token::Eof => {}
        }

        prev_token = Some(token.clone());
        i += 1;
    }

    Ok(formatter.output)
}
