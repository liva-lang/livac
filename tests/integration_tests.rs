use livac::{compile_file, CompilerOptions};
use std::path::PathBuf;
use std::process::Command;
use tempfile::TempDir;

/// Test helper para proyectos de integración
fn test_integration_project(project_name: &str) {
    let project_path = PathBuf::from(format!("tests/integration/{}", project_name));
    let main_liva = project_path.join("main.liva");

    // Crear directorio temporal para output
    let temp_dir = TempDir::new().unwrap();
    let output_dir = temp_dir.path().to_path_buf();

    let options = CompilerOptions {
        input: main_liva,
        output: Some(output_dir.clone()),
        verbose: false,
        check_only: false,
    };

    // Compilar el proyecto
    let compilation_result = compile_file(&options).unwrap_or_else(|err| {
        panic!(
            "Compilation failed for project: {} -- {}",
            project_name, err
        )
    });

    // Verificar que se generaron los archivos
    assert!(compilation_result.rust_code.is_some());
    assert!(compilation_result.cargo_toml.is_some());
    assert!(compilation_result.output_dir.is_some());

    let output_dir = compilation_result.output_dir.unwrap();

    // Verificar que existen los archivos generados
    let main_rs = output_dir.join("src/main.rs");
    let cargo_toml = output_dir.join("Cargo.toml");

    assert!(
        main_rs.exists(),
        "main.rs not generated for project: {}",
        project_name
    );
    assert!(
        cargo_toml.exists(),
        "Cargo.toml not generated for project: {}",
        project_name
    );

    // Verificar que el código Rust generado es válido (al menos sintácticamente)
    let rust_code = std::fs::read_to_string(&main_rs).unwrap();
    assert!(
        !rust_code.is_empty(),
        "Generated Rust code is empty for project: {}",
        project_name
    );

    let cargo_toml_content = std::fs::read_to_string(&cargo_toml).unwrap();
    assert!(
        !cargo_toml_content.is_empty(),
        "Generated Cargo.toml is empty for project: {}",
        project_name
    );

    // Verificar que Cargo.toml tiene la estructura básica
    assert!(
        cargo_toml_content.contains("[package]"),
        "Cargo.toml missing [package] section for project: {}",
        project_name
    );
    assert!(
        cargo_toml_content.contains("name ="),
        "Cargo.toml missing name for project: {}",
        project_name
    );

    // Ejecutar cargo check para asegurar que el proyecto compila (en modo offline si es necesario)
    if std::env::var("LIVA_RUN_CARGO_CHECK")
        .map(|value| value == "1")
        .unwrap_or(false)
    {
        let check_output = Command::new("cargo")
            .arg("check")
            .arg("--offline")
            .current_dir(&output_dir)
            .output()
            .expect("Failed to execute cargo check");

        if !check_output.status.success() {
            let stderr = String::from_utf8_lossy(&check_output.stderr);
            if stderr.contains("Could not resolve host")
                || stderr.contains("failed to download")
                || stderr.contains("registry index")
            {
                eprintln!(
                    "Skipping cargo check verification for '{}' due to offline dependency resolution:\n{}",
                    project_name, stderr
                );
            } else {
                panic!(
                    "cargo check failed for project: {}\n{}",
                    project_name, stderr
                );
            }
        }
    } else {
        eprintln!(
            "Skipping cargo check for '{}' (set LIVA_RUN_CARGO_CHECK=1 to enable)",
            project_name
        );
    }
}

#[test]
fn test_hello_world_integration() {
    test_integration_project("proj_hello");
}

#[test]
fn test_async_integration() {
    test_integration_project("proj_async");
}

#[test]
fn test_classes_integration() {
    test_integration_project("proj_classes");
}

#[test]
fn test_basic_integration() {
    test_integration_project("proj_basic");
}

#[test]
fn test_control_flow_integration() {
    test_integration_project("proj_control_flow");
}

#[test]
fn test_generics_integration() {
    test_integration_project("proj_generics");
}

#[test]
fn test_use_integration() {
    test_integration_project("proj_use");
}

#[test]
fn test_examples_integration() {
    test_integration_project("proj_examples");
}

#[test]
fn test_data_parallel_integration() {
    test_integration_project("proj_data_parallel");
}

#[test]
fn test_comprehensive_integration() {
    test_integration_project("proj_comprehensive");
}

#[test]
fn test_rust_interop_integration() {
    test_integration_project("proj_rust_interop");
}

#[test]
fn test_enum_import_integration() {
    test_integration_project("proj_enum_import");
}

#[test]
fn test_compile_check_only() {
    let project_path = PathBuf::from("tests/integration/proj_hello");
    let main_liva = project_path.join("main.liva");

    let options = CompilerOptions {
        input: main_liva,
        output: None,
        verbose: false,
        check_only: true,
    };

    let result = compile_file(&options);
    assert!(result.is_ok(), "Check-only compilation failed");

    let compilation_result = result.unwrap();
    assert!(compilation_result.rust_code.is_none());
    assert!(compilation_result.cargo_toml.is_none());
    assert!(compilation_result.output_dir.is_none());
}

// ── livac init tests ────────────────────────────────────────

fn livac_binary() -> PathBuf {
    // Use debug binary from cargo build
    PathBuf::from(env!("CARGO_BIN_EXE_livac"))
}

#[test]
fn test_init_creates_multi_file_project() {
    let tmp = TempDir::new().unwrap();
    let project_name = "test-project";

    let output = Command::new(livac_binary())
        .args(["init", project_name])
        .current_dir(tmp.path())
        .output()
        .expect("Failed to execute livac init");

    assert!(output.status.success(), "livac init failed: {}", String::from_utf8_lossy(&output.stderr));

    let project_dir = tmp.path().join(project_name);
    assert!(project_dir.join("main.liva").exists(), "main.liva not created");
    assert!(project_dir.join("math.liva").exists(), "math.liva not created");
    assert!(project_dir.join("models.liva").exists(), "models.liva not created");
    assert!(project_dir.join("tests/main.test.liva").exists(), "test file not created");
    assert!(project_dir.join(".gitignore").exists(), ".gitignore not created");

    // Verify main.liva content
    let main_content = std::fs::read_to_string(project_dir.join("main.liva")).unwrap();
    assert!(main_content.contains("main()"), "main.liva should contain main()");
    assert!(main_content.contains(project_name), "main.liva should contain project name");
    assert!(main_content.contains("import"), "main.liva should import from modules");

    // Verify math.liva content
    let math_content = std::fs::read_to_string(project_dir.join("math.liva")).unwrap();
    assert!(math_content.contains("add("), "math.liva should contain add function");
    assert!(math_content.contains("factorial("), "math.liva should contain factorial");

    // Verify models.liva content
    let models_content = std::fs::read_to_string(project_dir.join("models.liva")).unwrap();
    assert!(models_content.contains("Point"), "models.liva should contain Point data class");
    assert!(models_content.contains("Pet"), "models.liva should contain Pet class");

    // Verify test file content
    let test_content = std::fs::read_to_string(project_dir.join("tests/main.test.liva")).unwrap();
    assert!(test_content.contains("test "), "test file should contain test blocks");
}



#[test]
fn test_init_already_exists() {
    let tmp = TempDir::new().unwrap();
    let project_name = "existing";

    // Create directory first
    std::fs::create_dir(tmp.path().join(project_name)).unwrap();

    let output = Command::new(livac_binary())
        .args(["init", project_name])
        .current_dir(tmp.path())
        .output()
        .expect("Failed to execute livac init");

    assert!(!output.status.success(), "Should fail when directory exists");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("already exists"), "Error message should mention 'already exists'");
}

#[test]
fn test_init_invalid_name() {
    let tmp = TempDir::new().unwrap();

    // Name with spaces
    let output = Command::new(livac_binary())
        .args(["init", "bad name"])
        .current_dir(tmp.path())
        .output()
        .expect("Failed to execute livac init");

    assert!(!output.status.success(), "Should fail with invalid name");
}

#[test]
fn test_init_dot_current_dir() {
    let tmp = TempDir::new().unwrap();
    let project_dir = tmp.path().join("my-cool-app");
    std::fs::create_dir(&project_dir).unwrap();

    let output = Command::new(livac_binary())
        .args(["init", "."])
        .current_dir(&project_dir)
        .output()
        .expect("Failed to execute livac init .");

    assert!(output.status.success(), "livac init . failed: {}", String::from_utf8_lossy(&output.stderr));

    assert!(project_dir.join("main.liva").exists(), "main.liva not created");
    assert!(project_dir.join("math.liva").exists(), "math.liva not created");
    assert!(project_dir.join("models.liva").exists(), "models.liva not created");
    assert!(project_dir.join("tests/main.test.liva").exists(), "test file not created");
    assert!(project_dir.join(".gitignore").exists(), ".gitignore not created");

    // Verify project name is derived from directory name
    let main_content = std::fs::read_to_string(project_dir.join("main.liva")).unwrap();
    assert!(main_content.contains("my-cool-app"), "Should use directory name as project name");
}

#[test]
fn test_init_dot_already_has_main() {
    let tmp = TempDir::new().unwrap();
    let project_dir = tmp.path().join("existing-proj");
    std::fs::create_dir(&project_dir).unwrap();
    std::fs::write(project_dir.join("main.liva"), "main() { }").unwrap();

    let output = Command::new(livac_binary())
        .args(["init", "."])
        .current_dir(&project_dir)
        .output()
        .expect("Failed to execute livac init .");

    assert!(!output.status.success(), "Should fail when main.liva exists");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("already exists"), "Error should mention file already exists");
}

#[test]
fn test_init_no_args_defaults_to_dot() {
    let tmp = TempDir::new().unwrap();
    let project_dir = tmp.path().join("auto-init");
    std::fs::create_dir(&project_dir).unwrap();

    let output = Command::new(livac_binary())
        .args(["init"])
        .current_dir(&project_dir)
        .output()
        .expect("Failed to execute livac init");

    assert!(output.status.success(), "livac init (no args) failed: {}", String::from_utf8_lossy(&output.stderr));
    assert!(project_dir.join("main.liva").exists(), "main.liva not created");
}
