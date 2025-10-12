/// Liva Compiler Library
///
/// This library provides the core compilation pipeline for the Liva programming language.
/// Liva combines the simplicity of TypeScript, the expressiveness of Python, and the
/// safety and performance of Rust.
///
/// # Architecture
///
/// The compiler follows a traditional pipeline:
///
/// 1. **Lexer** - Tokenizes source code
/// 2. **Parser** - Builds an Abstract Syntax Tree (AST)
/// 3. **Semantic Analysis** - Type checking, async inference, visibility validation
/// 4. **Desugaring** - Transforms Liva constructs to Rust concepts
/// 5. **Code Generation** - Emits Rust code and Cargo.toml
///
/// # Example
///
/// ```rust,no_run
/// use livac::{compile_file, CompilerOptions};
///
/// let options = CompilerOptions {
///     input: "hello.liva".into(),
///     output: Some("./build".into()),
///     verbose: true,
///     check_only: false,
/// };
///
/// match compile_file(&options) {
///     Ok(_) => println!("Compilation successful!"),
///     Err(e) => eprintln!("Compilation failed: {}", e),
/// }
/// ```
pub mod ast;
pub mod codegen;
pub mod desugaring;
pub mod error;
pub mod ir;
pub mod lexer;
pub mod lowering;
pub mod parser;
pub mod semantic;

pub use error::{CompilerError, Result};

use std::path::{Path, PathBuf};

/// Compiler options for configuring the compilation process
#[derive(Debug, Clone)]
pub struct CompilerOptions {
    /// Path to the input Liva source file
    pub input: PathBuf,

    /// Optional output directory (default: ./target/liva_build)
    pub output: Option<PathBuf>,

    /// Show generated Rust code
    pub verbose: bool,

    /// Only check syntax, don't generate code
    pub check_only: bool,
}

impl Default for CompilerOptions {
    fn default() -> Self {
        Self {
            input: PathBuf::from("main.liva"),
            output: None,
            verbose: false,
            check_only: false,
        }
    }
}

/// Compile a Liva source file with the given options
///
/// # Arguments
///
/// * `options` - Compiler options specifying input file, output directory, etc.
///
/// # Returns
///
/// * `Ok(CompilationResult)` - On successful compilation
/// * `Err(CompilerError)` - On compilation failure
///
/// # Example
///
/// ```rust,no_run
/// use livac::{compile_file, CompilerOptions};
/// use std::path::PathBuf;
///
/// let options = CompilerOptions {
///     input: PathBuf::from("example.liva"),
///     output: Some(PathBuf::from("./output")),
///     verbose: false,
///     check_only: false,
/// };
///
/// compile_file(&options)?;
/// # Ok::<(), livac::CompilerError>(())
/// ```
pub fn compile_file(options: &CompilerOptions) -> Result<CompilationResult> {
    // Read source file
    let source = std::fs::read_to_string(&options.input)
        .map_err(|e| CompilerError::IoError(format!("Failed to read input file: {}", e)))?;

    compile_source(&source, options)
}

/// Compile Liva source code from a string
///
/// # Arguments
///
/// * `source` - The Liva source code as a string
/// * `options` - Compiler options
///
/// # Returns
///
/// * `Ok(CompilationResult)` - On successful compilation
/// * `Err(CompilerError)` - On compilation failure
pub fn compile_source(source: &str, options: &CompilerOptions) -> Result<CompilationResult> {
    // 1. Lexer - tokenize source
    let tokens = lexer::tokenize(source)?;

    // 2. Parser - build AST
    let ast = parser::parse(tokens, source)?;

    // 3. Semantic analysis
    let analyzed_ast = semantic::analyze(ast)?;

    // If check-only mode, stop here
    if options.check_only {
        return Ok(CompilationResult {
            rust_code: None,
            cargo_toml: None,
            output_dir: None,
        });
    }

    // 4. Desugaring
    let desugar_ctx = desugaring::desugar(analyzed_ast.clone())?;

    // 5. Code generation
    let ir_module = lowering::lower_program(&analyzed_ast);

    let (rust_code, cargo_toml) =
        codegen::generate_from_ir(&ir_module, &analyzed_ast, desugar_ctx)?;

    // 6. Write output files if output directory specified
    let output_dir = if let Some(out_dir) = &options.output {
        Some(write_output_files(&rust_code, &cargo_toml, out_dir)?)
    } else {
        None
    };

    Ok(CompilationResult {
        rust_code: Some(rust_code.to_string()),
        cargo_toml: Some(cargo_toml.to_string()),
        output_dir,
    })
}

/// Result of a successful compilation
#[derive(Debug, Clone)]
pub struct CompilationResult {
    /// Generated Rust code (None if check-only mode)
    pub rust_code: Option<String>,

    /// Generated Cargo.toml (None if check-only mode)
    pub cargo_toml: Option<String>,

    /// Output directory where files were written
    pub output_dir: Option<PathBuf>,
}

/// Write generated code to the filesystem
fn write_output_files(rust_code: &str, cargo_toml: &str, output_dir: &Path) -> Result<PathBuf> {
    // Create output directory
    std::fs::create_dir_all(output_dir)
        .map_err(|e| CompilerError::IoError(format!("Failed to create output directory: {}", e)))?;

    // Create src directory
    let src_dir = output_dir.join("src");
    std::fs::create_dir_all(&src_dir)
        .map_err(|e| CompilerError::IoError(format!("Failed to create src directory: {}", e)))?;

    // Write main.rs
    let main_rs_path = src_dir.join("main.rs");
    std::fs::write(&main_rs_path, rust_code)
        .map_err(|e| CompilerError::IoError(format!("Failed to write main.rs: {}", e)))?;

    // Write Cargo.toml
    let cargo_toml_path = output_dir.join("Cargo.toml");
    std::fs::write(&cargo_toml_path, cargo_toml)
        .map_err(|e| CompilerError::IoError(format!("Failed to write Cargo.toml: {}", e)))?;

    Ok(output_dir.to_path_buf())
}

/// Parse and validate Liva source code without generating output
///
/// This is useful for IDE integration, linters, and syntax checkers.
///
/// # Arguments
///
/// * `source` - The Liva source code to check
///
/// # Returns
///
/// * `Ok(())` - If the source is valid
/// * `Err(CompilerError)` - If there are syntax or semantic errors
///
/// # Example
///
/// ```rust
/// use livac::check_syntax;
///
/// let source = r#"
///     sum(a: number, b: number): number = a + b
/// "#;
///
/// match check_syntax(source) {
///     Ok(_) => println!("Syntax is valid"),
///     Err(e) => eprintln!("Syntax error: {}", e),
/// }
/// # Ok::<(), livac::CompilerError>(())
/// ```
pub fn check_syntax(source: &str) -> Result<()> {
    let tokens = lexer::tokenize(source)?;
    let ast = parser::parse(tokens, source)?;
    semantic::analyze(ast)?;
    Ok(())
}

/// Get version information about the compiler
pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

/// Get the full version string with additional information
pub fn version_info() -> String {
    format!(
        "Liva Compiler v{}\nRust Backend: rustc {}\nTarget: {}",
        version(),
        rustc_version::version().unwrap_or_else(|_| "unknown".parse().unwrap()),
        std::env::consts::ARCH
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_compile_simple_function() {
        let source = "sum(a: number, b: number): number = a + b";

        let options = CompilerOptions {
            input: PathBuf::from("test.liva"),
            output: None,
            verbose: false,
            check_only: false,
        };

        let result = compile_source(source, &options);
        assert!(result.is_ok());
    }

    #[test]
    fn test_check_syntax_valid() {
        let source = r#"
            main() {
                print("Hello, Liva!")
            }
        "#;

        assert!(check_syntax(source).is_ok());
    }

    #[test]
    fn test_check_syntax_invalid() {
        let source = "let x = ";

        assert!(check_syntax(source).is_err());
    }

    #[test]
    fn test_version() {
        let ver = version();
        assert!(!ver.is_empty());
        assert!(ver.contains('.'));
    }

    #[test]
    fn test_compile_source_check_only_returns_none() {
        let options = CompilerOptions {
            input: PathBuf::from("virtual.liva"),
            output: None,
            verbose: false,
            check_only: true,
        };

        let result = compile_source(
            r#"
            main() {
                let value = 42
                return value
            }
        "#,
            &options,
        )
        .expect("check should succeed");

        assert!(result.rust_code.is_none());
        assert!(result.output_dir.is_none());
    }

    #[test]
    fn test_compile_source_writes_output_files() {
        let tmp = tempdir().unwrap();
        let out_dir = tmp.path().join("build");
        let options = CompilerOptions {
            input: PathBuf::from("virtual.liva"),
            output: Some(out_dir.clone()),
            verbose: true,
            check_only: false,
        };

        let result = compile_source(
            r#"
            helper() = 1

            main() {
                print(helper())
            }
        "#,
            &options,
        )
        .expect("compilation should succeed");

        let rust_code = result.rust_code.expect("rust code expected");
        assert!(rust_code.contains("fn main"));
        let output_dir = result.output_dir.expect("output dir expected");
        assert!(output_dir.join("src/main.rs").exists());
        assert!(output_dir.join("Cargo.toml").exists());
    }

    #[test]
    fn test_version_info_contains_components() {
        let info = version_info();
        assert!(info.contains("Liva Compiler"));
        assert!(info.contains("Rust Backend"));
    }
}
