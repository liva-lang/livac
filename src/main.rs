use clap::Parser;
use colored::*;
use std::path::PathBuf;
use std::process::Command;

use livac::{CompilerError, CompilerOptions};

#[derive(Parser)]
#[command(name = "livac")]
#[command(about = "Liva → Rust compiler (v0.12)", long_about = None)]
struct Cli {
    /// Input Liva file
    input: Option<PathBuf>,

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

    /// Start Language Server Protocol mode
    #[arg(long)]
    lsp: bool,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    // LSP mode
    if cli.lsp {
        if let Err(e) = run_lsp_server().await {
            eprintln!("LSP server error: {}", e);
            std::process::exit(1);
        }
        return;
    }

    // Regular compilation mode
    let input = cli.input.as_ref().expect("input file required for compilation");

    if let Err(e) = compile(&cli, input) {
        // Output errors in JSON format if requested
        if cli.json {
            if let Some(json) = e.to_json() {
                println!("{}", json);
                std::process::exit(1);
            }
            // For non-structured errors, output simple JSON
            eprintln!(r#"{{"error": "{}"}}"#, e);
        } else {
            eprintln!("{} {}", "Error:".red().bold(), e);
        }
        std::process::exit(1);
    }
}

async fn run_lsp_server() -> Result<(), Box<dyn std::error::Error>> {
    use tower_lsp::{LspService, Server};
    use livac::lsp::LivaLanguageServer;

    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::build(|client| LivaLanguageServer::new(client))
        .finish();

    Server::new(stdin, stdout, socket).serve(service).await;

    Ok(())
}

fn compile(cli: &Cli, input: &PathBuf) -> Result<(), CompilerError> {
    let skip_cargo = std::env::var("LIVAC_SKIP_CARGO").is_ok();

    if !cli.json {
        println!("{}", "🧩 Liva Compiler v0.11.3".cyan().bold());
        println!("{} {}", "→ Compiling".green(), input.display());
    }

    let options = CompilerOptions {
        input: input.clone(),
        output: cli.output.clone(),
        verbose: false,
        check_only: cli.check,
    };

    let result = livac::compile_file(&options)?;

    if cli.check {
        if !cli.json {
            println!("{}", "✓ Check passed".green().bold());
        }
        return Ok(());
    }

    let main_rs = result.rust_code.ok_or_else(|| {
        CompilerError::CodegenError(livac::SemanticErrorInfo::new(
            "E3001",
            "Code generation failed",
            "No Rust code generated",
        ))
    })?;
    let cargo_toml = result.cargo_toml.ok_or_else(|| {
        CompilerError::CodegenError(livac::SemanticErrorInfo::new(
            "E3002",
            "Code generation failed",
            "No Cargo.toml generated",
        ))
    })?;

    // Determine output directory
    let output_dir = if let Some(output) = &cli.output {
        // User specified --output, use that
        output.clone()
    } else if cli.run && result.has_imports {
        // --run with imports: use .liva_build/ next to source file
        let input_dir = input.parent().unwrap_or_else(|| std::path::Path::new("."));
        input_dir.join(".liva_build")
    } else {
        // Default: ./target/liva_build
        PathBuf::from("./target/liva_build")
    };
    
    std::fs::create_dir_all(&output_dir).map_err(|e| CompilerError::IoError(e.to_string()))?;

    let src_dir = output_dir.join("src");
    std::fs::create_dir_all(&src_dir).map_err(|e| CompilerError::IoError(e.to_string()))?;

    std::fs::write(src_dir.join("main.rs"), &main_rs)
        .map_err(|e| CompilerError::IoError(e.to_string()))?;

    // Write module files if present
    if let Some(module_files) = &result.module_files {
        for (rel_path, content) in module_files {
            let file_path = output_dir.join(rel_path);
            // Create parent directories if needed
            if let Some(parent) = file_path.parent() {
                std::fs::create_dir_all(parent)
                    .map_err(|e| CompilerError::IoError(e.to_string()))?;
            }
            std::fs::write(&file_path, content)
                .map_err(|e| CompilerError::IoError(e.to_string()))?;
        }
    }

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
        let output = Command::new("cargo")
            .arg("build")
            .arg("--color=always")
            .current_dir(&output_dir)
            .output()
            .map_err(|e| CompilerError::IoError(e.to_string()))?;

        if !output.status.success() {
            // Show the actual Rust compiler error
            eprintln!("\n{}", "Rust Compilation Error:".red().bold());
            eprintln!("{}", "=".repeat(80));
            
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            
            // Print stdout (cargo messages)
            if !stdout.is_empty() {
                eprint!("{}", stdout);
            }
            
            // Print stderr (error messages)
            if !stderr.is_empty() {
                eprint!("{}", stderr);
            }
            
            eprintln!("{}", "=".repeat(80));
            eprintln!("\n{}", "💡 Tip: This is a Rust type error in the generated code.".yellow());
            eprintln!("   Check the Liva code for type mismatches or incompatible operations.\n");
            
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
            input: Some(input.clone()),
            output: None,
            run: false,
            verbose: false,
            check: true,
            json: false,
            lsp: false,
        };

        let _guard = EnvVarGuard::set("LIVAC_SKIP_CARGO", "1");
        let result = compile(&cli, &input);
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
            input: Some(input.clone()),
            output: Some(output_dir.path().to_path_buf()),
            run: false,
            verbose: true,
            check: false,
            json: false,
            lsp: false,
        };

        let _guard = EnvVarGuard::set("LIVAC_SKIP_CARGO", "1");
        compile(&cli, &input).expect("compile should succeed");

        let src_main = output_dir.path().join("src/main.rs");
        let cargo_toml = output_dir.path().join("Cargo.toml");
        assert!(src_main.exists());
        assert!(cargo_toml.exists());
    }

    #[test]
    fn test_compile_missing_file_error() {
        let input = PathBuf::from("does_not_exist.liva");
        let cli = Cli {
            input: Some(input.clone()),
            output: None,
            run: false,
            verbose: false,
            check: false,
            json: false,
            lsp: false,
        };

        let err = compile(&cli, &input).expect_err("expected IO error");
        match err {
            CompilerError::IoError(msg) => {
                assert!(msg.contains("No such file"));
            }
            other => panic!("unexpected error: {other:?}"),
        }
    }
}
