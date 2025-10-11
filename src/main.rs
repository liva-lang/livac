use clap::Parser;
use colored::*;
use std::path::PathBuf;
use std::process::Command;

mod ast;
mod codegen;
mod error;
mod lexer;
mod parser;
mod semantic;
mod desugaring;

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
    println!("{}", "🧩 Liva Compiler v0.6".cyan().bold());
    println!("{} {}", "→ Compiling".green(), cli.input.display());

    // 1. Read source
    let source = std::fs::read_to_string(&cli.input)
        .map_err(|e| CompilerError::IoError(e.to_string()))?;

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
    println!("  {} Generating Rust code...", "→".blue());
    let (main_rs, cargo_toml) = codegen::generate_with_ast(&analyzed_ast, desugar_ctx)?;

    // 7. Write output
    let output_dir = cli.output.clone().unwrap_or_else(|| PathBuf::from("./target/liva_build"));
    std::fs::create_dir_all(&output_dir)
        .map_err(|e| CompilerError::IoError(e.to_string()))?;

    let src_dir = output_dir.join("src");
    std::fs::create_dir_all(&src_dir)
        .map_err(|e| CompilerError::IoError(e.to_string()))?;

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
    println!("  {} Running cargo build...", "→".blue());
    let status = Command::new("cargo")
        .arg("build")
        .current_dir(&output_dir)
        .status()
        .map_err(|e| CompilerError::IoError(e.to_string()))?;

    if !status.success() {
        return Err(CompilerError::CodegenError("Cargo build failed".into()));
    }

    println!("{}", "✓ Compilation successful!".green().bold());

    // 9. Run if requested
    if cli.run {
        println!("\n{}", "Running program:".cyan().bold());
        println!("{}", "=".repeat(60));
        
        let status = Command::new("cargo")
            .arg("run")
            .current_dir(&output_dir)
            .status()
            .map_err(|e| CompilerError::IoError(e.to_string()))?;

        if !status.success() {
            return Err(CompilerError::RuntimeError("Program execution failed".into()));
        }
    }

    Ok(())
}
