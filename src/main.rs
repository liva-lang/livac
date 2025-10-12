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

use error::CompilerError;

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
}

fn main() {
    let cli = Cli::parse();

    if let Err(e) = compile(&cli) {
        eprintln!("{} {}", "Error:".red().bold(), e);
        std::process::exit(1);
    }
}

fn compile(cli: &Cli) -> Result<(), CompilerError> {
    let skip_cargo = std::env::var("LIVAC_SKIP_CARGO").is_ok();

    println!("{}", "🧩 Liva Compiler v0.6".cyan().bold());
    println!("{} {}", "→ Compiling".green(), cli.input.display());

    // 1. Read source
    let source =
        std::fs::read_to_string(&cli.input).map_err(|e| CompilerError::IoError(e.to_string()))?;

    // 2. Lexer
    println!("  {} Lexical analysis...", "→".blue());
    let tokens = lexer::tokenize(&source)?;

    // 3. Parser
    println!("  {} Parsing...", "→".blue());
    let ast = parser::parse(tokens, &source)?;

    // 4. Semantic analysis
    println!("  {} Semantic analysis...", "→".blue());
    let analyzed_ast = semantic::analyze(ast)?;

    if cli.check {
        println!("{}", "✓ Check passed".green().bold());
        return Ok(());
    }

    // 5. Desugaring (Liva → Rust concepts)
    println!("  {} Desugaring to Rust...", "→".blue());
    let desugar_ctx = desugaring::desugar(analyzed_ast.clone())?;

    // 6. Code generation
    println!("  {} Lowering to IR...", "→".blue());
    let ir_module = lowering::lower_program(&analyzed_ast);

    println!("  {} Generating Rust code...", "→".blue());
    let (main_rs, cargo_toml) = codegen::generate_from_ir(&ir_module, &analyzed_ast, desugar_ctx)?;

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
