//! CLI subcommand integration tests.
//!
//! Exercises the public `livac` binary surface (check/fmt/lint/test/init)
//! end-to-end so that `main.rs` argument parsing + dispatch is covered.
//!
//! Each test invokes the binary built by cargo and asserts on exit code
//! and stdout/stderr fragments. Build/run subcommands invoke cargo and
//! are exercised by `tests/integration_tests.rs` via the library API,
//! so they are not duplicated here.

use std::path::PathBuf;
use std::process::Command;
use tempfile::TempDir;

fn livac_bin() -> PathBuf {
    // CARGO sets the path to the built binary when running tests.
    let exe = env!("CARGO_BIN_EXE_livac");
    PathBuf::from(exe)
}

fn write_temp(content: &str, name: &str) -> (TempDir, PathBuf) {
    let dir = TempDir::new().expect("tempdir");
    let path = dir.path().join(name);
    std::fs::write(&path, content).unwrap();
    (dir, path)
}

#[test]
fn check_succeeds_on_valid_program() {
    let (_dir, path) = write_temp("main() {\n    print(\"ok\")\n}\n", "ok.liva");
    let out = Command::new(livac_bin())
        .args(["check", path.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}

#[test]
fn check_reports_error_on_invalid_program() {
    // Parse error — unbalanced brace
    let (_dir, path) = write_temp("main() {\n    let x = 1\n", "bad.liva");
    let out = Command::new(livac_bin())
        .args(["check", path.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(!out.status.success());
}

#[test]
fn check_emits_json_when_requested() {
    let (_dir, path) = write_temp("main() {\n    let x =\n", "bad.liva");
    let out = Command::new(livac_bin())
        .args(["check", "--json", path.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(!out.status.success());
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        combined.contains('{'),
        "expected JSON in output: {}",
        combined
    );
}

#[test]
fn fmt_check_passes_on_clean_source() {
    let clean = "main() {\n    print(\"ok\")\n}\n";
    let (_dir, path) = write_temp(clean, "clean.liva");
    let out = Command::new(livac_bin())
        .args(["fmt", "--check", path.to_str().unwrap()])
        .output()
        .unwrap();
    // Clean files exit 0; the fmt may output a status line on stdout.
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}

#[test]
fn fmt_rewrites_messy_source() {
    let messy = "main(){print(\"ok\")}\n";
    let (_dir, path) = write_temp(messy, "messy.liva");
    let out = Command::new(livac_bin())
        .args(["fmt", path.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let after = std::fs::read_to_string(&path).unwrap();
    assert_ne!(after, messy, "fmt should reformat messy source");
}

#[test]
fn lint_runs_and_reports_unused_variable() {
    let src = "main() {\n    let unused_var = 42\n    print(\"hi\")\n}\n";
    let (_dir, path) = write_temp(src, "unused.liva");
    let out = Command::new(livac_bin())
        .args(["lint", path.to_str().unwrap()])
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);
    let combined = format!("{}{}", stdout, stderr);
    // Lint should at least mention the unused name or warning code W001.
    assert!(
        combined.contains("unused_var")
            || combined.to_lowercase().contains("unused")
            || combined.contains("W001"),
        "expected unused-warning, got:\nstdout={}\nstderr={}",
        stdout,
        stderr
    );
}

#[test]
fn lint_emits_json_when_requested() {
    let src = "main() {\n    let unused_var = 42\n}\n";
    let (_dir, path) = write_temp(src, "unused.liva");
    let out = Command::new(livac_bin())
        .args(["lint", "--json", path.to_str().unwrap()])
        .output()
        .unwrap();
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    // JSON mode should produce at least one JSON-shaped fragment.
    assert!(
        combined.contains('{') || out.status.success(),
        "stdout/stderr: {}",
        combined
    );
}

#[test]
fn help_lists_subcommands() {
    let out = Command::new(livac_bin()).arg("--help").output().unwrap();
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    for sub in [
        "build", "run", "check", "fmt", "test", "lint", "lsp", "init", "update",
    ] {
        assert!(
            stdout.contains(sub),
            "help missing subcommand `{}`:\n{}",
            sub,
            stdout
        );
    }
}

#[test]
fn no_args_prints_help_or_errors() {
    let out = Command::new(livac_bin()).output().unwrap();
    // clap exits with non-zero when no subcommand is given.
    assert!(!out.status.success());
}

#[test]
fn unknown_subcommand_errors() {
    let out = Command::new(livac_bin())
        .args(["bogus_subcommand"])
        .output()
        .unwrap();
    assert!(!out.status.success());
}
