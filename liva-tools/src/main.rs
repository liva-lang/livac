//! `liva-tools` — CLI dispatcher for Liva developer tooling.
//!
//! Subcommands:
//!   - `fmt <file> [--check] [--verbose]`
//!   - `lint <file> [--json]`
//!   - `lsp`
//!
//! Called as a subprocess by the `livac` binary so the (eventually frozen)
//! compiler crate has no dependency on developer tools.

use clap::{Parser, Subcommand};
use colored::Colorize;
use std::path::PathBuf;
use std::process::ExitCode;

use liva_tools::{formatter, linter, lsp};

#[derive(Parser)]
#[command(name = "liva-tools", version, about = "Liva developer tools")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Format a Liva source file
    Fmt {
        input: PathBuf,
        #[arg(long)]
        check: bool,
        #[arg(short, long)]
        verbose: bool,
    },
    /// Lint a Liva source file
    Lint {
        input: PathBuf,
        #[arg(long)]
        json: bool,
    },
    /// Run the Liva language server (stdio)
    Lsp,
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    match cli.command {
        Commands::Fmt {
            input,
            check,
            verbose,
        } => match run_format(&input, check, verbose) {
            Ok(()) => ExitCode::SUCCESS,
            Err(e) => {
                eprintln!("{} {}", "Error:".red().bold(), e);
                ExitCode::from(1)
            }
        },
        Commands::Lint { input, json } => ExitCode::from(run_lint(&input, json) as u8),
        Commands::Lsp => match run_lsp_server() {
            Ok(()) => ExitCode::SUCCESS,
            Err(e) => {
                eprintln!("{} {}", "Error:".red().bold(), e);
                ExitCode::from(1)
            }
        },
    }
}

fn run_lint(input: &PathBuf, json: bool) -> i32 {
    let source = match std::fs::read_to_string(input) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("{} Failed to read file: {}", "Error:".red().bold(), e);
            return 1;
        }
    };

    let filename = input.to_str().unwrap_or("unknown");

    let tokens = match livac::lexer::tokenize(&source) {
        Ok(t) => t,
        Err(e) => {
            if json {
                if let Some(json_str) = e.to_json() {
                    println!("{}", json_str);
                }
            } else {
                eprintln!("{} {}", "Error:".red().bold(), e);
            }
            return 1;
        }
    };

    let ast = match livac::parser::parse(tokens, &source) {
        Ok(a) => a,
        Err(e) => {
            if json {
                if let Some(json_str) = e.to_json() {
                    println!("{}", json_str);
                }
            } else {
                eprintln!("{} {}", "Error:".red().bold(), e);
            }
            return 1;
        }
    };

    let warnings = linter::lint(&ast, filename, &source);

    if json {
        println!("{}", linter::format_warnings_json(&warnings));
    } else if warnings.is_empty() {
        println!("{} {} — no warnings", "✓".green().bold(), input.display());
    } else {
        eprint!("{}", linter::format_warnings(&warnings));
    }

    0
}

fn run_format(
    input: &PathBuf,
    check_only: bool,
    verbose: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let options = formatter::FormatOptions::default();
    let source = std::fs::read_to_string(input)?;

    if check_only {
        let is_formatted = formatter::check_format(&source, &options)?;
        if is_formatted {
            println!("{} {}", "✓".green(), input.display());
        } else {
            let formatted = formatter::format_source(&source, &options)?;
            println!("{} {} (needs formatting)", "✗".red(), input.display());
            if verbose {
                println!();
                for diff in simple_diff(&source, &formatted) {
                    println!("{}", diff);
                }
            }
            std::process::exit(1);
        }
    } else {
        let formatted = formatter::format_source(&source, &options)?;
        if formatted == source {
            println!("{} {} (already formatted)", "✓".green(), input.display());
        } else {
            std::fs::write(input, &formatted)?;
            println!("{} {} (formatted)", "✓".green().bold(), input.display());
        }
    }

    Ok(())
}

fn run_lsp_server() -> Result<(), Box<dyn std::error::Error>> {
    let runtime = tokio::runtime::Runtime::new()?;
    runtime.block_on(async {
        use tower_lsp::{LspService, Server};
        let stdin = tokio::io::stdin();
        let stdout = tokio::io::stdout();
        let (service, socket) =
            LspService::build(|client| lsp::LivaLanguageServer::new(client)).finish();
        Server::new(stdin, stdout, socket).serve(service).await;
    });
    Ok(())
}

fn simple_diff(original: &str, formatted: &str) -> Vec<String> {
    let orig_lines: Vec<&str> = original.lines().collect();
    let fmt_lines: Vec<&str> = formatted.lines().collect();
    let max_lines = orig_lines.len().max(fmt_lines.len());
    let mut diffs = Vec::new();
    for i in 0..max_lines {
        let orig = orig_lines.get(i).unwrap_or(&"");
        let fmt = fmt_lines.get(i).unwrap_or(&"");
        if orig != fmt {
            diffs.push(format!(
                "  L{}: {} → {}",
                i + 1,
                format!("- {}", orig).red(),
                format!("+ {}", fmt).green()
            ));
        }
    }
    diffs
}
