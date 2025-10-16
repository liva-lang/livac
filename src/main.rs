use clap::Parser;
use colored::*;
use std::path::PathBuf;
use std::process::Command;

mod ast;
mod codegen;
mod desugaring;
mod error;
mod ir;
mod lexer;
mod lowering;
mod parser;
mod semantic;
mod span;

use error::CompilerError;
use livac::CompilerOptions;

#[derive(Parser)]
#[command(name = "livac")]
#[command(about = "Liva → Rust compiler (v0.6)", long_about = None)]
struct Cli {
    /// Input Liva file
    input: PathBuf,

    /// Output directory (default: ./target/liva_build)
    #[arg(short, long)]
    output: Option<PathBuf>,

    /// Run after compilation
    #[arg(short, long)]
    run: bool,

    /// Show generated Rust code
    #[arg(short, long)]
    verbose: bool,

    /// Only check, don't compile
    #[arg(short, long)]
    check: bool,

    /// Output errors in JSON format for IDE integration
    #[arg(long)]
    json: bool,
}

fn main() {
    let cli = Cli::parse();

    if let Err(e) = compile(&cli) {
        // Output errors in JSON format if requested
        if cli.json {
            if let CompilerError::SemanticError(ref info) = e {
                if let Ok(json) = info.to_json() {
                    println!("{}", json);
                    std::process::exit(1);
                }
            }
            // For non-semantic errors, output simple JSON
            eprintln!(r#"{{"error": "{}"}}"#, e);
        } else {
            eprintln!("{} {}", "Error:".red().bold(), e);
        }
        std::process::exit(1);
    }
}

fn compile(cli: &Cli) -> Result<(), CompilerError> {
    let skip_cargo = std::env::var("LIVAC_SKIP_CARGO").is_ok();

    println!("{}", "🧩 Liva Compiler v0.6".cyan().bold());
    println!("{} {}", "→ Compiling".green(), cli.input.display());

    let options = CompilerOptions {
        input: cli.input.clone(),
        output: cli.output.clone(),
        verbose: false,
        check_only: cli.check,
    };

    let result = livac::compile_file(&options).map_err(|e| match e {
        livac::CompilerError::LexerError(s) => CompilerError::LexerError(s),
        livac::CompilerError::ParseError { line, col, msg } => CompilerError::ParseError { line, col, msg },
        livac::CompilerError::SemanticError(info) => {
            // Just recreate using the local error module
            CompilerError::SemanticError(crate::error::SemanticErrorInfo {
                location: info.location.map(|loc| crate::error::ErrorLocation {
                    file: loc.file,
                    line: loc.line,
                    column: loc.column,
                    source_line: loc.source_line,
                }),
                code: info.code,
                title: info.title,
                message: info.message,
                help: info.help,
            })
        },
        livac::CompilerError::TypeError(s) => CompilerError::TypeError(s),
        livac::CompilerError::CodegenError(s) => CompilerError::CodegenError(s),
        livac::CompilerError::IoError(s) => CompilerError::IoError(s),
        livac::CompilerError::RuntimeError(s) => CompilerError::RuntimeError(s),
    })?;

    if cli.check {
        println!("{}", "✓ Check passed".green().bold());
        return Ok(());
    }

    let main_rs = result.rust_code.ok_or_else(|| CompilerError::CodegenError("No Rust code generated".to_string()))?;
    let cargo_toml = result.cargo_toml.ok_or_else(|| CompilerError::CodegenError("No Cargo.toml generated".to_string()))?;

    // 7. Write output
    let output_dir = cli
        .output
        .clone()
        .unwrap_or_else(|| PathBuf::from("./target/liva_build"));
    std::fs::create_dir_all(&output_dir).map_err(|e| CompilerError::IoError(e.to_string()))?;

    let src_dir = output_dir.join("src");
    std::fs::create_dir_all(&src_dir).map_err(|e| CompilerError::IoError(e.to_string()))?;

    std::fs::write(src_dir.join("main.rs"), &main_rs)
        .map_err(|e| CompilerError::IoError(e.to_string()))?;

    std::fs::write(output_dir.join("Cargo.toml"), &cargo_toml)
        .map_err(|e| CompilerError::IoError(e.to_string()))?;

    if cli.verbose {
        println!("\n{}", "Generated Rust code:".yellow().bold());
        println!("{}", "=".repeat(60));
        println!("{}", main_rs);
        println!("{}", "=".repeat(60));
    }

    println!("{} {}", "✓ Generated at".green(), output_dir.display());

    // 8. Run cargo build
    if skip_cargo {
        println!(
            "  {} Skipping cargo build (LIVAC_SKIP_CARGO set)...",
            "→".blue()
        );
    } else {
        println!("  {} Running cargo build...", "→".blue());
        let status = Command::new("cargo")
            .arg("build")
            .current_dir(&output_dir)
            .status()
            .map_err(|e| CompilerError::IoError(e.to_string()))?;

        if !status.success() {
            return Err(CompilerError::CodegenError("Cargo build failed".into()));
        }
    }

    println!("{}", "✓ Compilation successful!".green().bold());

    // 9. Run if requested
    if cli.run {
        if skip_cargo {
            println!(
                "\n{}",
                "Skipping program run (LIVAC_SKIP_CARGO set)"
                    .yellow()
                    .bold()
            );
        } else {
            println!("\n{}", "Running program:".cyan().bold());
            println!("{}", "=".repeat(60));

            let status = Command::new("cargo")
                .arg("run")
                .current_dir(&output_dir)
                .status()
                .map_err(|e| CompilerError::IoError(e.to_string()))?;

            if !status.success() {
                return Err(CompilerError::RuntimeError(
                    "Program execution failed".into(),
                ));
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    struct EnvVarGuard {
        key: &'static str,
    }

    impl EnvVarGuard {
        fn set(key: &'static str, value: &str) -> Self {
            std::env::set_var(key, value);
            Self { key }
        }
    }

    impl Drop for EnvVarGuard {
        fn drop(&mut self) {
            std::env::remove_var(self.key);
        }
    }

    fn create_source(content: &str) -> (tempfile::TempDir, PathBuf) {
        let dir = tempdir().unwrap();
        let path = dir.path().join("main.liva");
        fs::write(&path, content).unwrap();
        (dir, path)
    }

    #[test]
    fn test_compile_check_mode() {
        let (_dir, input) = create_source(
            r#"
            main() {
                print("Hello")
            }
        "#,
        );

        let cli = Cli {
            input,
            output: None,
            run: false,
            verbose: false,
            check: true,
            json: false,
        };

        let _guard = EnvVarGuard::set("LIVAC_SKIP_CARGO", "1");
        let result = compile(&cli);
        assert!(result.is_ok());
    }

    #[test]
    fn test_compile_generates_output() {
        let (_dir, input) = create_source(
            r#"
            helper() => 42

            main() {
                print(helper())
            }
        "#,
        );
        let output_dir = tempdir().unwrap();

        let cli = Cli {
            input,
            output: Some(output_dir.path().to_path_buf()),
            run: false,
            verbose: true,
            check: false,
            json: false,
        };

        let _guard = EnvVarGuard::set("LIVAC_SKIP_CARGO", "1");
        compile(&cli).expect("compile should succeed");

        let src_main = output_dir.path().join("src/main.rs");
        let cargo_toml = output_dir.path().join("Cargo.toml");
        assert!(src_main.exists());
        assert!(cargo_toml.exists());
    }

    #[test]
    fn test_compile_missing_file_error() {
        let cli = Cli {
            input: PathBuf::from("does_not_exist.liva"),
            output: None,
            run: false,
            verbose: false,
            check: false,
            json: false,
        };

        let err = compile(&cli).expect_err("expected IO error");
        match err {
            CompilerError::IoError(msg) => {
                assert!(msg.contains("No such file"));
            }
            other => panic!("unexpected error: {other:?}"),
        }
    }
}
