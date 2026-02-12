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

    /// Format source file(s) in place
    #[arg(long)]
    fmt: bool,

    /// Check if files are formatted (exit 1 if not)
    #[arg(long)]
    fmt_check: bool,

    /// Run tests: discover and execute *.test.liva files
    #[arg(long)]
    test: bool,

    /// Filter tests by name (substring match)
    #[arg(long)]
    filter: Option<String>,
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

    // Format mode
    if cli.fmt || cli.fmt_check {
        let input = cli.input.as_ref().expect("input file required for formatting");
        if let Err(e) = run_format(&cli, input) {
            if cli.json {
                eprintln!(r#"{{"error": "{}"}}"#, e);
            } else {
                eprintln!("{} {}", "Error:".red().bold(), e);
            }
            std::process::exit(1);
        }
        return;
    }

    // Test mode
    if cli.test {
        let exit_code = run_tests(&cli);
        std::process::exit(exit_code);
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

fn run_format(cli: &Cli, input: &PathBuf) -> Result<(), CompilerError> {
    use livac::formatter::{format_source, check_format, FormatOptions};

    let options = FormatOptions::default();
    let source = std::fs::read_to_string(input)
        .map_err(|e| CompilerError::IoError(format!("Failed to read file: {}", e)))?;

    if cli.fmt_check {
        // Check mode: report whether file is formatted
        let is_formatted = check_format(&source, &options)?;
        if is_formatted {
            println!("{} {}", "✓".green(), input.display());
        } else {
            let formatted = format_source(&source, &options)?;
            println!("{} {} (needs formatting)", "✗".red(), input.display());
            
            // Show a simple diff
            if cli.verbose {
                println!();
                for diff in simple_diff(&source, &formatted) {
                    println!("{}", diff);
                }
            }
            
            std::process::exit(1);
        }
    } else {
        // Format in place
        let formatted = format_source(&source, &options)?;
        if formatted == source {
            println!("{} {} (already formatted)", "✓".green(), input.display());
        } else {
            std::fs::write(input, &formatted)
                .map_err(|e| CompilerError::IoError(format!("Failed to write file: {}", e)))?;
            println!("{} {} (formatted)", "✓".green().bold(), input.display());
        }
    }

    Ok(())
}

/// Simple line-by-line diff for format check output
fn simple_diff(original: &str, formatted: &str) -> Vec<String> {
    let orig_lines: Vec<&str> = original.lines().collect();
    let fmt_lines: Vec<&str> = formatted.lines().collect();
    let mut diffs = Vec::new();
    let max_lines = orig_lines.len().max(fmt_lines.len());
    
    for i in 0..max_lines {
        let orig = orig_lines.get(i).unwrap_or(&"");
        let fmt = fmt_lines.get(i).unwrap_or(&"");
        if orig != fmt {
            diffs.push(format!("  L{}: {} → {}", i + 1, 
                format!("- {}", orig).red(),
                format!("+ {}", fmt).green()
            ));
        }
    }
    
    diffs
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

fn run_tests(cli: &Cli) -> i32 {
    use walkdir::WalkDir;
    use std::time::Instant;

    let skip_cargo = std::env::var("LIVAC_SKIP_CARGO").is_ok();

    println!("{}", "🧪 Liva Test Runner".cyan().bold());
    println!();

    let total_start = Instant::now();

    // Discover test files
    let test_files: Vec<PathBuf> = if let Some(input) = &cli.input {
        // Specific file given
        if !input.exists() {
            eprintln!("{} File not found: {}", "Error:".red().bold(), input.display());
            return 1;
        }
        vec![input.clone()]
    } else {
        // Discover *.test.liva files recursively from current directory
        let search_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        let mut files: Vec<PathBuf> = WalkDir::new(&search_dir)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| {
                let path = e.path();
                path.extension().map_or(false, |ext| ext == "liva")
                    && path.file_stem().map_or(false, |stem| {
                        stem.to_str().map_or(false, |s| s.ends_with(".test"))
                    })
            })
            .map(|e| e.into_path())
            .collect();
        files.sort();
        files
    };

    if test_files.is_empty() {
        println!("{}", "No test files found (*.test.liva)".yellow());
        println!();
        println!("Create a test file like:");
        println!("  tests/math.test.liva");
        println!();
        println!("With test blocks:");
        println!("  test \"addition works\" {{");
        println!("      let result = 2 + 3");
        println!("      if result != 5 {{");
        println!("          throw \"expected 5\"");
        println!("      }}");
        println!("  }}");
        return 0;
    }

    println!(
        "  {} Found {} test file{}",
        "→".blue(),
        test_files.len(),
        if test_files.len() == 1 { "" } else { "s" }
    );
    println!();

    let mut total_passed = 0;
    let mut total_failed = 0;
    let mut total_files_passed = 0;
    let mut total_files_failed = 0;
    let mut failed_files: Vec<(PathBuf, String)> = Vec::new();

    for test_file in &test_files {
        let file_start = Instant::now();
        let relative_path = test_file
            .strip_prefix(std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")))
            .unwrap_or(test_file);

        // 1. Compile the test file
        let options = CompilerOptions {
            input: test_file.clone(),
            output: None,
            verbose: false,
            check_only: false,
        };

        let result = match livac::compile_file(&options) {
            Ok(r) => r,
            Err(e) => {
                println!(
                    " {} {}",
                    "FAIL".white().on_red().bold(),
                    relative_path.display()
                );
                println!("       Compilation error: {}", e);
                println!();
                total_files_failed += 1;
                failed_files.push((test_file.clone(), format!("Compilation error: {}", e)));
                continue;
            }
        };

        let main_rs = match &result.rust_code {
            Some(code) => code.clone(),
            None => {
                println!(
                    " {} {}",
                    "FAIL".white().on_red().bold(),
                    relative_path.display()
                );
                println!("       No code generated");
                println!();
                total_files_failed += 1;
                failed_files.push((test_file.clone(), "No code generated".to_string()));
                continue;
            }
        };

        let cargo_toml = result.cargo_toml.unwrap_or_default();

        // Check if the generated code actually contains test functions
        let test_count = main_rs.matches("#[test]").count()
            + main_rs.matches("#[tokio::test]").count();
        if test_count == 0 {
            println!(
                " {} {} (no test blocks found)",
                "SKIP".black().on_yellow().bold(),
                relative_path.display()
            );
            println!();
            continue;
        }

        // 2. Write to build directory
        let build_dir = test_file
            .parent()
            .unwrap_or_else(|| std::path::Path::new("."))
            .join(".liva_test_build")
            .join(
                test_file
                    .file_stem()
                    .unwrap_or_default()
                    .to_str()
                    .unwrap_or("test"),
            );

        if let Err(e) = std::fs::create_dir_all(build_dir.join("src")) {
            eprintln!("  Error creating build dir: {}", e);
            total_files_failed += 1;
            failed_files.push((test_file.clone(), format!("IO error: {}", e)));
            continue;
        }

        if let Err(e) = std::fs::write(build_dir.join("src/main.rs"), &main_rs) {
            eprintln!("  Error writing main.rs: {}", e);
            total_files_failed += 1;
            failed_files.push((test_file.clone(), format!("IO error: {}", e)));
            continue;
        }

        // Write module files if present
        if let Some(module_files) = &result.module_files {
            for (rel_path, content) in module_files {
                let file_path = build_dir.join(rel_path);
                if let Some(parent) = file_path.parent() {
                    let _ = std::fs::create_dir_all(parent);
                }
                let _ = std::fs::write(&file_path, content);
            }
        }

        if let Err(e) = std::fs::write(build_dir.join("Cargo.toml"), &cargo_toml) {
            eprintln!("  Error writing Cargo.toml: {}", e);
            total_files_failed += 1;
            failed_files.push((test_file.clone(), format!("IO error: {}", e)));
            continue;
        }

        if skip_cargo {
            println!(
                " {} {} ({} test{}, skipped cargo)",
                "SKIP".black().on_yellow().bold(),
                relative_path.display(),
                test_count,
                if test_count == 1 { "" } else { "s" }
            );
            continue;
        }

        // 3. Run cargo test
        let mut cmd = Command::new("cargo");
        cmd.arg("test")
            .arg("--color=always")
            .current_dir(&build_dir);

        // Add filter if specified
        if let Some(filter) = &cli.filter {
            cmd.arg("--").arg(filter);
        }

        let output = match cmd.output() {
            Ok(o) => o,
            Err(e) => {
                println!(
                    " {} {}",
                    "FAIL".white().on_red().bold(),
                    relative_path.display()
                );
                println!("       Failed to run cargo test: {}", e);
                println!();
                total_files_failed += 1;
                failed_files.push((test_file.clone(), format!("cargo test error: {}", e)));
                continue;
            }
        };

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        // Parse test results from cargo test output
        let (passed, failed) = parse_cargo_test_output(&stdout);
        let file_elapsed = file_start.elapsed();

        if output.status.success() {
            println!(
                " {} {} ({} test{}, {:.0?})",
                "PASS".white().on_green().bold(),
                relative_path.display(),
                passed,
                if passed == 1 { "" } else { "s" },
                file_elapsed
            );

            // Show individual test results if verbose
            if cli.verbose {
                for line in stdout.lines() {
                    if line.contains("test test_") {
                        let display_line = line
                            .replace("test test_", "    ✓ ")
                            .replace(" ... ok", "");
                        println!("{}", display_line.green());
                    }
                }
            }

            total_passed += passed;
            total_files_passed += 1;
        } else {
            println!(
                " {} {} ({} passed, {} failed, {:.0?})",
                "FAIL".white().on_red().bold(),
                relative_path.display(),
                passed,
                failed,
                file_elapsed
            );

            // Show failing test details
            let mut in_failure = false;
            for line in stdout.lines() {
                if line.contains("test test_") && line.contains("FAILED") {
                    let display_line = line
                        .replace("test test_", "    ✗ ")
                        .replace(" ... FAILED", "");
                    println!("{}", display_line.red());
                    in_failure = true;
                } else if line.contains("test test_") && line.contains("ok") {
                    let display_line = line
                        .replace("test test_", "    ✓ ")
                        .replace(" ... ok", "");
                    println!("{}", display_line.green());
                }
            }

            // Show error output
            if !in_failure {
                for line in stderr.lines() {
                    if line.contains("panicked") || line.contains("Error") {
                        println!("       {}", line.red());
                    }
                }
            }

            println!();
            total_passed += passed;
            total_failed += failed;
            total_files_failed += 1;
            failed_files.push((
                test_file.clone(),
                format!("{} test{} failed", failed, if failed == 1 { "" } else { "s" }),
            ));
        }
    }

    // Summary
    let total_elapsed = total_start.elapsed();
    println!();
    println!("{}", "─".repeat(50));

    let total_tests = total_passed + total_failed;
    let summary = format!(
        "Tests:  {} passed, {} failed, {} total",
        total_passed, total_failed, total_tests
    );
    let files_summary = format!(
        "Files:  {} passed, {} failed, {} total",
        total_files_passed,
        total_files_failed,
        total_files_passed + total_files_failed
    );
    let time_summary = format!("Time:   {:.2?}", total_elapsed);

    if total_failed == 0 && total_files_failed == 0 {
        println!("{}", summary.green().bold());
        println!("{}", files_summary.green());
    } else {
        println!("{}", summary.red().bold());
        println!("{}", files_summary.red());
    }
    println!("{}", time_summary);

    if !failed_files.is_empty() {
        println!();
        println!("{}", "Failed files:".red().bold());
        for (path, reason) in &failed_files {
            let rel = path
                .strip_prefix(std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")))
                .unwrap_or(path);
            println!("  {} {} — {}", "✗".red(), rel.display(), reason);
        }
    }

    println!();

    // Clean up build directories
    for test_file in &test_files {
        let build_dir = test_file
            .parent()
            .unwrap_or_else(|| std::path::Path::new("."))
            .join(".liva_test_build");
        let _ = std::fs::remove_dir_all(&build_dir);
    }

    if total_failed > 0 || total_files_failed > 0 {
        1
    } else {
        0
    }
}

/// Parse the output of `cargo test` to extract pass/fail counts
fn parse_cargo_test_output(output: &str) -> (usize, usize) {
    // Look for line like: "test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out"
    for line in output.lines() {
        if line.contains("test result:") {
            let passed = extract_number(line, "passed");
            let failed = extract_number(line, "failed");
            return (passed, failed);
        }
    }
    // Fallback: count individual test lines
    let passed = output.lines().filter(|l| l.contains("... ok")).count();
    let failed = output.lines().filter(|l| l.contains("... FAILED")).count();
    (passed, failed)
}

/// Extract a number before a word in a string, e.g., "3 passed" → 3
fn extract_number(s: &str, word: &str) -> usize {
    if let Some(idx) = s.find(word) {
        let before = &s[..idx].trim_end();
        if let Some(last_word) = before.rsplit_once(|c: char| !c.is_ascii_digit()) {
            last_word.1.parse().unwrap_or(0)
        } else {
            before.parse().unwrap_or(0)
        }
    } else {
        0
    }
}

fn compile(cli: &Cli, input: &PathBuf) -> Result<(), CompilerError> {
    let skip_cargo = std::env::var("LIVAC_SKIP_CARGO").is_ok();

    if !cli.json {
        println!("{}", "🧩 Liva Compiler v1.0.0".cyan().bold());
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
            fmt: false,
            fmt_check: false,
            test: false,
            filter: None,
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
            fmt: false,
            fmt_check: false,
            test: false,
            filter: None,
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
            fmt: false,
            fmt_check: false,
            test: false,
            filter: None,
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
