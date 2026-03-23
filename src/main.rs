use clap::{Parser, Subcommand};
use colored::*;
use std::path::PathBuf;
use std::process::Command;

use livac::{CompilerError, CompilerOptions};

const GITHUB_REPO: &str = "liva-lang/livac";

#[derive(Parser)]
#[command(name = "livac")]
#[command(version = env!("CARGO_PKG_VERSION"))]
#[command(about = "Liva → Rust compiler", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Compile a Liva file to a native binary
    Build {
        /// Input Liva file
        input: PathBuf,

        /// Output directory (default: ./target/liva_build)
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Show generated Rust code
        #[arg(short, long)]
        verbose: bool,

        /// Compile with optimizations (cargo build --release)
        #[arg(long)]
        release: bool,

        /// Output errors in JSON format for IDE integration
        #[arg(long)]
        json: bool,
    },

    /// Compile and run a Liva file
    Run {
        /// Input Liva file
        input: PathBuf,

        /// Output directory (default: ./target/liva_build)
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Show generated Rust code
        #[arg(short, long)]
        verbose: bool,

        /// Compile with optimizations (cargo build --release)
        #[arg(long)]
        release: bool,

        /// Output errors in JSON format for IDE integration
        #[arg(long)]
        json: bool,

        /// Arguments to pass to the compiled program (after --)
        #[arg(last = true)]
        program_args: Vec<String>,
    },

    /// Check a Liva file for errors without compiling
    Check {
        /// Input Liva file
        input: PathBuf,

        /// Output errors in JSON format for IDE integration
        #[arg(long)]
        json: bool,
    },

    /// Format Liva source files
    Fmt {
        /// Input Liva file
        input: PathBuf,

        /// Check formatting without modifying (exit 1 if not formatted)
        #[arg(long)]
        check: bool,

        /// Show diff details
        #[arg(short, long)]
        verbose: bool,

        /// Output errors in JSON format for IDE integration
        #[arg(long)]
        json: bool,
    },

    /// Run tests (discover and execute *.test.liva files)
    Test {
        /// Specific test file (default: discover *.test.liva recursively)
        input: Option<PathBuf>,

        /// Filter tests by name (substring match)
        #[arg(long)]
        filter: Option<String>,

        /// Show detailed output
        #[arg(short, long)]
        verbose: bool,
    },

    /// Start Language Server Protocol mode
    Lsp,

    /// Lint a Liva file for warnings (unused variables, unreachable code, etc.)
    Lint {
        /// Input Liva file
        input: PathBuf,

        /// Output warnings in JSON format for IDE integration
        #[arg(long)]
        json: bool,
    },

    /// Update livac to the latest version
    Update,

    /// Initialize a new Liva project
    Init {
        /// Project name or "." for current directory
        #[arg(default_value = ".")]
        name: String,
    },
}

/// Internal struct passed to compile() with resolved options
struct CompileArgs {
    output: Option<PathBuf>,
    run: bool,
    verbose: bool,
    check: bool,
    json: bool,
    release: bool,
    program_args: Vec<String>,
}

fn handle_compile_error(json: bool, e: CompilerError) -> ! {
    if json {
        if let Some(json_str) = e.to_json() {
            println!("{}", json_str);
            std::process::exit(1);
        }
        eprintln!(r#"{{"error": "{}"}}"#, e);
    } else {
        eprintln!("{} {}", "Error:".red().bold(), e);
    }
    std::process::exit(1);
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Update => {
            if let Err(e) = self_update().await {
                eprintln!("{} {}", "Error:".red().bold(), e);
                std::process::exit(1);
            }
        }
        Commands::Lsp => {
            if let Err(e) = run_lsp_server().await {
                eprintln!("LSP server error: {}", e);
                std::process::exit(1);
            }
        }
        Commands::Fmt { input, check, verbose, json } => {
            if let Err(e) = run_format(&input, check, verbose) {
                if json {
                    eprintln!(r#"{{"error": "{}"}}"#, e);
                } else {
                    eprintln!("{} {}", "Error:".red().bold(), e);
                }
                std::process::exit(1);
            }
        }
        Commands::Test { input, filter, verbose } => {
            let exit_code = run_tests(input.as_ref(), filter.as_deref(), verbose);
            std::process::exit(exit_code);
        }
        Commands::Check { input, json } => {
            let args = CompileArgs {
                output: None,
                run: false,
                verbose: false,
                check: true,
                json,
                release: false,
                program_args: vec![],
            };
            if let Err(e) = compile(&args, &input) {
                handle_compile_error(args.json, e);
            }
        }
        Commands::Build { input, output, verbose, release, json } => {
            let args = CompileArgs {
                output,
                run: false,
                verbose,
                check: false,
                json,
                release,
                program_args: vec![],
            };
            if let Err(e) = compile(&args, &input) {
                handle_compile_error(args.json, e);
            }
        }
        Commands::Run { input, output, verbose, release, json, program_args } => {
            let args = CompileArgs {
                output,
                run: true,
                verbose,
                check: false,
                json,
                release,
                program_args,
            };
            if let Err(e) = compile(&args, &input) {
                handle_compile_error(args.json, e);
            }
        }
        Commands::Init { name } => {
            if let Err(e) = run_init(&name) {
                eprintln!("{} {}", "Error:".red().bold(), e);
                std::process::exit(1);
            }
        }
        Commands::Lint { input, json } => {
            let exit_code = run_lint(&input, json);
            std::process::exit(exit_code);
        }
    }
}

fn run_lint(input: &PathBuf, json: bool) -> i32 {
    use livac::linter;

    let source = match std::fs::read_to_string(input) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("{} Failed to read file: {}", "Error:".red().bold(), e);
            return 1;
        }
    };

    let filename = input.to_str().unwrap_or("unknown");

    // Parse the file (lexer + parser)
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

    // Run linter
    let warnings = linter::lint(&ast, filename, &source);

    if json {
        println!("{}", linter::format_warnings_json(&warnings));
    } else {
        if warnings.is_empty() {
            println!("{} {} — no warnings", "✓".green().bold(), input.display());
        } else {
            eprint!("{}", linter::format_warnings(&warnings));
        }
    }

    0
}

fn run_format(input: &PathBuf, check_only: bool, verbose: bool) -> Result<(), CompilerError> {
    use livac::formatter::{check_format, format_source, FormatOptions};

    let options = FormatOptions::default();
    let source = std::fs::read_to_string(input)
        .map_err(|e| CompilerError::IoError(format!("Failed to read file: {}", e)))?;

    if check_only {
        // Check mode: report whether file is formatted
        let is_formatted = check_format(&source, &options)?;
        if is_formatted {
            println!("{} {}", "✓".green(), input.display());
        } else {
            let formatted = format_source(&source, &options)?;
            println!("{} {} (needs formatting)", "✗".red(), input.display());

            // Show a simple diff
            if verbose {
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

/// Initialize a new Liva project with scaffolding
fn run_init(name: &str) -> Result<(), Box<dyn std::error::Error>> {
    // Resolve "." to current directory
    let (project_dir, display_name) = if name == "." {
        let cwd = std::env::current_dir()?;
        let dir_name = cwd
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("my-project")
            .to_string();
        (cwd, dir_name)
    } else {
        // Validate project name
        if name.is_empty() {
            return Err("Project name cannot be empty".into());
        }
        if name.contains(std::path::MAIN_SEPARATOR) || name.contains('/') || name.contains('\\') {
            return Err("Project name cannot contain path separators".into());
        }
        if !name.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_') {
            return Err("Project name can only contain letters, numbers, hyphens, and underscores".into());
        }
        let dir = PathBuf::from(name);
        if dir.exists() {
            return Err(format!("Directory '{}' already exists", name).into());
        }
        (dir, name.to_string())
    };

    // Check if target files already exist (for "." mode)
    if name == "." {
        if project_dir.join("main.liva").exists() {
            return Err("main.liva already exists in current directory".into());
        }
    }

    // Create directory structure
    std::fs::create_dir_all(project_dir.join("tests"))?;

    println!(
        "{} Creating project '{}'...",
        "→".blue(),
        display_name.bold()
    );

    // Write source files
    std::fs::write(project_dir.join("main.liva"), template_main(&display_name))?;
    std::fs::write(project_dir.join("math.liva"), template_math())?;
    std::fs::write(project_dir.join("models.liva"), template_models())?;
    std::fs::write(
        project_dir.join("tests").join("main.test.liva"),
        template_test(),
    )?;
    std::fs::write(project_dir.join(".gitignore"), GITIGNORE_TEMPLATE)?;

    println!("{} Created project structure:", "✓".green().bold());
    if name == "." {
        println!("    ./ ({})", display_name);
    } else {
        println!("    {}/", display_name);
    }
    println!("    ├── main.liva");
    println!("    ├── math.liva");
    println!("    ├── models.liva");
    println!("    ├── tests/");
    println!("    │   └── main.test.liva");
    println!("    └── .gitignore");
    println!();
    println!("  Get started:");
    if name == "." {
        println!("    {} main.liva", "livac run".cyan());
        println!("    {} tests/main.test.liva", "livac test".cyan());
    } else {
        println!("    {} {}/main.liva", "livac run".cyan(), display_name);
        println!("    {} {}/tests/main.test.liva", "livac test".cyan(), display_name);
    }

    Ok(())
}

// ── Templates ──────────────────────────────────────────────

const GITIGNORE_TEMPLATE: &str = "\
# Liva build output
target/

# Environment files with secrets
.env
*.env.local

# OS files
.DS_Store
Thumbs.db
";

fn template_main(name: &str) -> String {
    format!(
        r#"// {name} - Liva Language Tour
// A multi-file showcase of the language
// Run:  livac run main.liva
// Test: livac test tests/main.test.liva

import {{ add, square, isEven, factorial, divide, describeScore, greet }} from "./math.liva"
import {{ Point, Pet }} from "./models.liva"

const VERSION = "0.1.0"

// -- Enums --

enum Color {{ Red, Green, Blue }}

enum Shape {{
    Circle(radius: float),
    Rect(width: float, height: float),
    Dot
}}

// -- Enum helpers (switch expressions) --

shapeInfo(s: Shape): string {{
    let info = switch s {{
        Shape.Circle(r)  => $"Circle r={{r}}",
        Shape.Rect(w, h) => $"Rect {{w}}x{{h}}",
        Shape.Dot        => "Dot"
    }}
    return info
}}

colorName(c: Color): string {{
    let result = switch c {{
        Color.Red   => "Red",
        Color.Green => "Green",
        Color.Blue  => "Blue"
    }}
    return result
}}

// -- Main --

main() {{
    print($"Welcome to {name}! (v{{VERSION}})")
    print("")

    // -- Variables & Types --
    print("-- Variables & Types --")
    let count = 0
    let pi: float = 3.14159
    let words: [string] = ["Liva", "compiles", "to", "Rust"]

    print($"pi = {{pi}}")
    let motto = words.join(" ")
    print($"Motto: {{motto}}")
    print("")

    // -- Functions (imported from math.liva) --
    print("-- Functions --")
    print($"add(2, 3)    = {{add(2, 3)}}")
    print($"square(7)    = {{square(7)}}")
    print($"factorial(6) = {{factorial(6)}}")
    print(greet("World"))
    print(greet("Liva"))
    print("")

    // -- Error Handling --
    print("-- Error Handling --")

    // Error binding: two-variable pattern
    let result, e1 = divide(10.0, 3.0)
    if e1 {{
        print($"Error: {{e1}}")
    }} else {{
        print($"10 / 3 = {{result}}")
    }}

    // Catching a failure
    let bad, e2 = divide(5.0, 0.0)
    if e2 {{
        print($"Caught: {{e2}}")
    }}
    print("")

    // -- Control Flow --
    print("-- Control Flow --")
    for i in 1..=5 {{
        count = count + 1
    }}
    print($"Counted to {{count}}")

    for word in words {{
        print($"  -> {{word}}")
    }}
    print("")

    // -- Arrays (functional pipeline) --
    print("-- Arrays --")
    let numbers = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]
    let evens = numbers.filter(n => isEven(n))
    let doubled = evens.map(n => n * 2)
    let total = doubled.reduce(0, (acc, n) => acc + n)

    print($"Numbers: {{numbers}}")
    print($"Evens:   {{evens}}")
    print($"Doubled: {{doubled}}")
    print($"Sum:     {{total}}")

    let hasLarge = numbers.some(n => n > 8)
    let allPos = numbers.every(n => n > 0)
    print($"Has > 8: {{hasLarge}}")
    print($"All > 0: {{allPos}}")
    print("")

    // -- Maps --
    print("-- Maps --")
    let scores = Map {{ "Alice": 95, "Bob": 82, "Carol": 91 }}
    let aliceScore = scores.get("Alice") or 0
    print($"Alice: {{aliceScore}}")
    scores.set("Dave", 78)

    let hasDave = scores.has("Dave")
    print($"Has Dave: {{hasDave}}")

    for student, score in scores {{
        print($"  {{student}}: {{describeScore(score)}}")
    }}
    print("")

    // -- Sets --
    print("-- Sets --")
    let tags = Set {{ "rust", "fast", "safe" }}
    tags.add("compiled")
    let hasRust = tags.has("rust")
    let hasGo = tags.has("go")
    print($"has rust: {{hasRust}}")
    print($"has go:   {{hasGo}}")
    print("")

    // -- Data Classes (imported from models.liva) --
    print("-- Data Classes --")
    let p1 = Point(10, 20)
    let p2 = Point(10, 20)
    let p3 = Point(99, 1)
    print($"p1:        {{p1}}")
    print($"p1 == p2:  {{p1 == p2}}")
    print($"p1 == p3:  {{p1 == p3}}")
    print("")

    // -- Enums & Pattern Matching --
    print("-- Enums & Pattern Matching --")
    print($"Color: {{colorName(Color.Red)}}")
    print($"  {{shapeInfo(Shape.Circle(5.0))}}")
    print($"  {{shapeInfo(Shape.Rect(4.0, 6.0))}}")
    print($"  {{shapeInfo(Shape.Dot)}}")
    print("")

    // -- Classes & Interfaces (imported from models.liva) --
    print("-- Classes & Interfaces --")
    let pet = Pet("Luna", "cat")
    print(pet.describe())
    print("")

    // -- Grade Report (switch with ranges) --
    print("-- Grade Report --")
    let testScores = [95, 82, 73, 45]
    for score in testScores {{
        print($"  Score {{score}}: {{describeScore(score)}}")
    }}
    print("")

    // -- Math stdlib --
    print("-- Math --")
    print($"sqrt(16) = {{Math.sqrt(16.0)}}")
    print($"pow(2,3) = {{Math.pow(2.0, 3.0)}}")
    print($"PI       = {{Math.PI}}")
    print($"abs(-5)  = {{Math.abs(-5.0)}}")
    print("")

    print("Tour complete!")
}}
"#,
        name = name
    )
}

fn template_math() -> String {
    r#"// math.liva - Pure functions

// -- Arrow functions (implicit return) --

add(a: number, b: number): number => a + b
square(n: number): number => n * n
isEven(n: number): bool => n % 2 == 0
greet(name: string): string => $"Hello, {name}!"

// -- Block function (multi-line, recursion) --

factorial(n: number): number {
    if n <= 1 { return 1 }
    return n * factorial(n - 1)
}

// -- Fallible function (fail keyword) --

divide(a: float, b: float): float {
    if b == 0.0 { fail "Division by zero" }
    return a / b
}

// -- Pattern matching: switch with ranges --

describeScore(score: number): string {
    let label = switch score {
        90..=100 => "A - Excellent",
        80..=89  => "B - Good",
        70..=79  => "C - Average",
        _        => "F - Needs work"
    }
    return label
}
"#
    .to_string()
}

fn template_models() -> String {
    r#"// models.liva - Data classes, classes, interfaces

// -- Data Class (auto constructor, Display, PartialEq) --

Point { x: number; y: number }

// -- Interface --

Describable { describe(): string }

// -- Class implementing interface --

Pet : Describable {
    name: string
    kind: string
    constructor(name: string, kind: string) {
        this.name = name
        this.kind = kind
    }
    describe(): string => $"{this.name} the {this.kind}"
}
"#
    .to_string()
}

fn template_test() -> String {
    r#"// Tests for the project
// Run: livac test tests/main.test.liva

import { describe, test, expect } from "liva/test"

// -- Functions under test --

add(a: number, b: number): number => a + b
square(n: number): number => n * n
isEven(n: number): bool => n % 2 == 0

factorial(n: number): number {
    if n <= 1 { return 1 }
    return n * factorial(n - 1)
}

describeScore(score: number): string {
    let label = switch score {
        90..=100 => "A - Excellent",
        80..=89  => "B - Good",
        70..=79  => "C - Average",
        _        => "F - Needs work"
    }
    return label
}

// -- Tests --

describe("Math", () => {
    test("add works", () => {
        expect(add(2, 3)).toBe(5)
        expect(add(-1, 1)).toBe(0)
        expect(add(0, 0)).toBe(0)
    })

    test("square computes squares", () => {
        expect(square(7)).toBe(49)
        expect(square(0)).toBe(0)
    })

    test("isEven detects parity", () => {
        expect(isEven(4)).toBeTruthy()
        expect(isEven(3)).toBeFalsy()
        expect(isEven(0)).toBeTruthy()
    })

    test("factorial recurses correctly", () => {
        expect(factorial(0)).toBe(1)
        expect(factorial(1)).toBe(1)
        expect(factorial(6)).toBe(720)
    })
})

describe("Score Grading", () => {
    test("describeScore grades correctly", () => {
        expect(describeScore(95)).toBe("A - Excellent")
        expect(describeScore(82)).toBe("B - Good")
        expect(describeScore(73)).toBe("C - Average")
        expect(describeScore(50)).toBe("F - Needs work")
    })
})
"#
    .to_string()
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

async fn run_lsp_server() -> Result<(), Box<dyn std::error::Error>> {
    use livac::lsp::LivaLanguageServer;
    use tower_lsp::{LspService, Server};

    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::build(|client| LivaLanguageServer::new(client)).finish();

    Server::new(stdin, stdout, socket).serve(service).await;

    Ok(())
}

/// Self-update: download the latest release from GitHub and replace the current binary
async fn self_update() -> Result<(), Box<dyn std::error::Error>> {
    let current_version = env!("CARGO_PKG_VERSION");
    println!(
        "{} Checking for updates (current: v{})...",
        "🧩".to_string(),
        current_version
    );

    // 1. Fetch latest release info from GitHub
    let client = reqwest::Client::builder()
        .user_agent("livac-self-update")
        .build()?;

    // Try /releases/latest first, fall back to /releases[0]
    let release: serde_json::Value = match client
        .get(format!(
            "https://api.github.com/repos/{}/releases/latest",
            GITHUB_REPO
        ))
        .send()
        .await
    {
        Ok(resp) if resp.status().is_success() => resp.json().await?,
        _ => {
            // Fallback: get all releases and pick the first one
            let releases: Vec<serde_json::Value> = client
                .get(format!(
                    "https://api.github.com/repos/{}/releases",
                    GITHUB_REPO
                ))
                .send()
                .await?
                .json()
                .await?;

            releases
                .into_iter()
                .next()
                .ok_or("No releases found on GitHub")?
        }
    };

    let latest_tag = release["tag_name"]
        .as_str()
        .ok_or("Could not parse tag_name from release")?;
    let latest_version = latest_tag.trim_start_matches('v');

    if latest_version == current_version {
        println!(
            "{} Already up to date (v{})",
            "✓".green().bold(),
            current_version
        );
        return Ok(());
    }

    println!(
        "{} New version available: v{} → {}",
        "→".blue(),
        current_version,
        latest_tag.bold()
    );

    // 2. Detect platform
    let (os_name, arch) = detect_platform()?;
    let artifact_name = format!("livac-{}-{}", os_name, arch);
    let ext = if os_name == "windows" { "zip" } else { "tar.gz" };
    let asset_name = format!("{}.{}", artifact_name, ext);

    // 3. Find the download URL
    let assets = release["assets"]
        .as_array()
        .ok_or("Could not parse assets from release")?;

    let download_url = assets
        .iter()
        .find(|a| a["name"].as_str() == Some(&asset_name))
        .and_then(|a| a["browser_download_url"].as_str())
        .ok_or_else(|| {
            format!(
                "Asset '{}' not found in release {}. Available: {}",
                asset_name,
                latest_tag,
                assets
                    .iter()
                    .filter_map(|a| a["name"].as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        })?;

    println!(
        "{} Downloading {}...",
        "→".blue(),
        asset_name
    );

    // 4. Download to temp file
    let tmp_dir = std::env::temp_dir().join("livac-update");
    std::fs::create_dir_all(&tmp_dir)?;
    let archive_path = tmp_dir.join(&asset_name);

    let response = client.get(download_url).send().await?;
    if !response.status().is_success() {
        return Err(format!("Download failed: HTTP {}", response.status()).into());
    }
    let bytes = response.bytes().await?;
    std::fs::write(&archive_path, &bytes)?;

    println!(
        "{} Downloaded ({:.1} MB)",
        "✓".green(),
        bytes.len() as f64 / 1_048_576.0
    );

    // 5. Extract
    println!("{} Extracting...", "→".blue());
    let extract_dir = tmp_dir.join("extracted");
    let _ = std::fs::remove_dir_all(&extract_dir);
    std::fs::create_dir_all(&extract_dir)?;

    let status = Command::new("tar")
        .args(["xzf", archive_path.to_str().unwrap(), "-C", extract_dir.to_str().unwrap()])
        .status()?;

    if !status.success() {
        return Err("Failed to extract archive".into());
    }

    // 6. Find the new binary
    let binary_name = if cfg!(windows) { "livac.exe" } else { "livac" };
    let new_binary = find_file_recursive(&extract_dir, binary_name)
        .ok_or("Could not find livac binary in archive")?;

    // 7. Replace current binary
    let current_exe = std::env::current_exe()?;
    println!(
        "{} Updating {}...",
        "→".blue(),
        current_exe.display()
    );

    // On Unix, we can't overwrite a running binary directly.
    // Strategy: rename old → .bak, copy new, delete .bak
    let backup_path = current_exe.with_extension("bak");
    let _ = std::fs::remove_file(&backup_path); // remove any old backup

    // Try rename first (works if same filesystem)
    if std::fs::rename(&current_exe, &backup_path).is_err() {
        // If rename fails (cross-device), try remove + copy
        std::fs::remove_file(&current_exe)?;
    }

    std::fs::copy(&new_binary, &current_exe)?;

    // Make executable on Unix
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&current_exe, std::fs::Permissions::from_mode(0o755))?;
    }

    // Remove backup
    let _ = std::fs::remove_file(&backup_path);

    // 8. Save version to ~/.liva/version
    let liva_dir = dirs_or_home().join(".liva");
    let _ = std::fs::create_dir_all(&liva_dir);
    let _ = std::fs::write(liva_dir.join("version"), latest_tag);

    // 9. Also update ~/.liva/bin/livac if it exists and is different from current_exe
    let liva_bin = liva_dir.join("bin").join(binary_name);
    if liva_bin.exists() && liva_bin.canonicalize().ok() != current_exe.canonicalize().ok() {
        let _ = std::fs::remove_file(&liva_bin);
        let _ = std::fs::copy(&new_binary, &liva_bin);
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = std::fs::set_permissions(&liva_bin, std::fs::Permissions::from_mode(0o755));
        }
        println!("{} Also updated {}", "✓".green(), liva_bin.display());
    }

    // Cleanup
    let _ = std::fs::remove_dir_all(&tmp_dir);

    println!();
    println!(
        "{} Updated to {} successfully!",
        "✓".green().bold(),
        latest_tag.bold()
    );
    println!();

    // Verify
    let output = Command::new(&current_exe).arg("--version").output();
    if let Ok(out) = output {
        let ver = String::from_utf8_lossy(&out.stdout);
        println!("{} {}", "✓".green(), ver.trim());
    }

    Ok(())
}

/// Detect the current platform for self-update
fn detect_platform() -> Result<(&'static str, &'static str), Box<dyn std::error::Error>> {
    let os_name = if cfg!(target_os = "linux") {
        "linux"
    } else if cfg!(target_os = "macos") {
        "darwin"
    } else if cfg!(target_os = "windows") {
        "windows"
    } else {
        return Err("Unsupported OS".into());
    };

    let arch = if cfg!(target_arch = "x86_64") {
        "x64"
    } else if cfg!(target_arch = "aarch64") {
        "arm64"
    } else {
        return Err("Unsupported architecture".into());
    };

    Ok((os_name, arch))
}

/// Recursively find a file by name
fn find_file_recursive(dir: &std::path::Path, name: &str) -> Option<PathBuf> {
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() && path.file_name().map(|n| n == name).unwrap_or(false) {
                return Some(path);
            }
            if path.is_dir() {
                if let Some(found) = find_file_recursive(&path, name) {
                    return Some(found);
                }
            }
        }
    }
    None
}

/// Get home directory
fn dirs_or_home() -> PathBuf {
    std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("."))
}

fn run_tests(input: Option<&PathBuf>, filter: Option<&str>, verbose: bool) -> i32 {
    use std::time::Instant;
    use walkdir::WalkDir;

    let skip_cargo = std::env::var("LIVAC_SKIP_CARGO").is_ok();

    // Check Rust toolchain early
    if !skip_cargo {
        if let Err(e) = check_cargo_available() {
            eprintln!("{} {}", "Error:".red().bold(), e);
            return 1;
        }
    }

    println!("{}", "🧪 Liva Test Runner".cyan().bold());
    println!();

    let total_start = Instant::now();

    // Discover test files
    let test_files: Vec<PathBuf> = if let Some(input) = input {
        // Specific file given
        if !input.exists() {
            eprintln!(
                "{} File not found: {}",
                "Error:".red().bold(),
                input.display()
            );
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
        let test_count =
            main_rs.matches("#[test]").count() + main_rs.matches("#[tokio::test]").count();
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
        if let Some(filter) = filter {
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
            if verbose {
                for line in stdout.lines() {
                    if line.contains("test test_") {
                        let display_line =
                            line.replace("test test_", "    ✓ ").replace(" ... ok", "");
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
                    let display_line = line.replace("test test_", "    ✓ ").replace(" ... ok", "");
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
                format!(
                    "{} test{} failed",
                    failed,
                    if failed == 1 { "" } else { "s" }
                ),
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

/// Check if cargo is available in PATH
fn check_cargo_available() -> Result<(), CompilerError> {
    match Command::new("cargo").arg("--version").output() {
        Ok(output) if output.status.success() => Ok(()),
        _ => {
            let msg = format!(
                "Rust toolchain not found.\n\n\
                 Liva compiles to Rust and requires `cargo` to build the generated code.\n\n\
                 Install Rust:\n\
                 {}",
                if cfg!(windows) {
                    "  winget install Rustlang.Rustup\n  \
                     — or download from https://rustup.rs"
                } else {
                    "  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
                }
            );
            Err(CompilerError::IoError(msg))
        }
    }
}

fn compile(args: &CompileArgs, input: &PathBuf) -> Result<(), CompilerError> {
    let skip_cargo = std::env::var("LIVAC_SKIP_CARGO").is_ok();

    // Check Rust toolchain early (unless we're skipping cargo or just checking)
    if !skip_cargo && !args.check {
        check_cargo_available()?;
    }

    if !args.json {
        println!("{}", format!("🧩 Liva Compiler v{}", env!("CARGO_PKG_VERSION")).cyan().bold());
        println!("{} {}", "→ Compiling".green(), input.display());
    }

    let options = CompilerOptions {
        input: input.clone(),
        output: args.output.clone(),
        verbose: false,
        check_only: args.check,
    };

    let result = livac::compile_file(&options)?;

    if args.check {
        if !args.json {
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
    let output_dir = if let Some(output) = &args.output {
        // User specified --output, use that
        output.clone()
    } else if args.run && result.has_imports {
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

    if args.verbose {
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
        println!("  {} Running cargo build{}...", "→".blue(), if args.release { " --release" } else { "" });
        let mut cargo_cmd = Command::new("cargo");
        cargo_cmd
            .arg("build")
            .arg("--color=always");
        if args.release {
            cargo_cmd.arg("--release");
        }
        let output = cargo_cmd
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
            eprintln!(
                "\n{}",
                "💡 Tip: This is a Rust type error in the generated code.".yellow()
            );
            eprintln!("   Check the Liva code for type mismatches or incompatible operations.\n");

            return Err(CompilerError::CodegenError("Cargo build failed".into()));
        }
    }

    println!("{}", "✓ Compilation successful!".green().bold());

    // 9. Run if requested
    if args.run {
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

            // Run the compiled binary from the user's working directory
            // (not from the build dir, so relative paths in the program work correctly)
            let profile = if args.release { "release" } else { "debug" };
            let binary_path = output_dir.join("target").join(profile).join("liva_project");

            let mut cmd = Command::new(&binary_path);

            // Pass LIVA_VERBOSE env var when --verbose is set (enables Log.debug output)
            if args.verbose {
                cmd.env("LIVA_VERBOSE", "1");
            }

            // Pass through program arguments after --
            for arg in &args.program_args {
                cmd.arg(arg);
            }

            let status = cmd
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

        let args = CompileArgs {
            output: None,
            run: false,
            verbose: false,
            check: true,
            json: false,
            release: false,
            program_args: vec![],
        };

        let _guard = EnvVarGuard::set("LIVAC_SKIP_CARGO", "1");
        let result = compile(&args, &input);
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

        let args = CompileArgs {
            output: Some(output_dir.path().to_path_buf()),
            run: false,
            verbose: true,
            check: false,
            json: false,
            release: false,
            program_args: vec![],
        };

        let _guard = EnvVarGuard::set("LIVAC_SKIP_CARGO", "1");
        compile(&args, &input).expect("compile should succeed");

        let src_main = output_dir.path().join("src/main.rs");
        let cargo_toml = output_dir.path().join("Cargo.toml");
        assert!(src_main.exists());
        assert!(cargo_toml.exists());
    }

    #[test]
    fn test_compile_missing_file_error() {
        let input = PathBuf::from("does_not_exist.liva");
        let args = CompileArgs {
            output: None,
            run: false,
            verbose: false,
            check: false,
            json: false,
            release: false,
            program_args: vec![],
        };

        let err = compile(&args, &input).expect_err("expected IO error");
        match err {
            CompilerError::IoError(msg) => {
                // "No such file" on Unix, "cannot find" on Windows
                assert!(
                    msg.contains("No such file") || msg.contains("cannot find"),
                    "unexpected IO error message: {msg}"
                );
            }
            other => panic!("unexpected error: {other:?}"),
        }
    }
}
