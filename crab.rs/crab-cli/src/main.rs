use clap::{Parser, Subcommand};

mod c_compiler;
mod commands;
mod utils;

use commands::*;

#[derive(Parser)]
#[command(name = "crab")]
#[command(version = "0.1.0")]
#[command(about = "Crab: Dart-like syntax that transpiles to Rust", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    New {
        name: String,
    },
    Build,
    Run,
    Check,
    Clean,
    Test {
        #[arg(long)]
        release: bool,
    },
    Fmt,
    Lint,
    Add {
        package: String,
    },
    Remove {
        package: String,
    },
    Doc,
    Publish,
}

fn main() {
    let cli = Cli::parse();
    match cli.command {
        Commands::New { name } => cmd_new(&name),
        Commands::Build => cmd_build(),
        Commands::Run => cmd_run(),
        Commands::Check => cmd_check(),
        Commands::Clean => cmd_clean(),
        Commands::Test { release } => cmd_test(release),
        Commands::Fmt => cmd_fmt(),
        Commands::Lint => cmd_lint(),
        Commands::Add { package } => cmd_add(&package),
        Commands::Remove { package } => cmd_remove(&package),
        Commands::Doc => cmd_doc(),
        Commands::Publish => cmd_publish(),
    }
}
