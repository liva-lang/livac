use livac::{compile_file, CompilerOptions};
use std::path::PathBuf;
use tempfile::TempDir;

/// Integration test for HTTP client functionality
#[test]
fn test_http_get() {
    let test_file = PathBuf::from("tests/integration/proj_http/test_get.liva");
    let temp_dir = TempDir::new().unwrap();
    
    let options = CompilerOptions {
        input: test_file,
        output: Some(temp_dir.path().to_path_buf()),
        verbose: false,
        check_only: false,
    };
    
    let result = compile_file(&options);
    assert!(result.is_ok(), "Failed to compile test_get.liva: {:?}", result.err());
}

#[test]
fn test_http_post() {
    let test_file = PathBuf::from("tests/integration/proj_http/test_post.liva");
    let temp_dir = TempDir::new().unwrap();
    
    let options = CompilerOptions {
        input: test_file,
        output: Some(temp_dir.path().to_path_buf()),
        verbose: false,
        check_only: false,
    };
    
    let result = compile_file(&options);
    assert!(result.is_ok(), "Failed to compile test_post.liva: {:?}", result.err());
}

#[test]
fn test_http_put() {
    let test_file = PathBuf::from("tests/integration/proj_http/test_put.liva");
    let temp_dir = TempDir::new().unwrap();
    
    let options = CompilerOptions {
        input: test_file,
        output: Some(temp_dir.path().to_path_buf()),
        verbose: false,
        check_only: false,
    };
    
    let result = compile_file(&options);
    assert!(result.is_ok(), "Failed to compile test_put.liva: {:?}", result.err());
}

#[test]
fn test_http_delete() {
    let test_file = PathBuf::from("tests/integration/proj_http/test_delete.liva");
    let temp_dir = TempDir::new().unwrap();
    
    let options = CompilerOptions {
        input: test_file,
        output: Some(temp_dir.path().to_path_buf()),
        verbose: false,
        check_only: false,
    };
    
    let result = compile_file(&options);
    assert!(result.is_ok(), "Failed to compile test_delete.liva: {:?}", result.err());
}

#[test]
fn test_http_errors() {
    let test_file = PathBuf::from("tests/integration/proj_http/test_errors.liva");
    let temp_dir = TempDir::new().unwrap();
    
    let options = CompilerOptions {
        input: test_file,
        output: Some(temp_dir.path().to_path_buf()),
        verbose: false,
        check_only: false,
    };
    
    let result = compile_file(&options);
    assert!(result.is_ok(), "Failed to compile test_errors.liva: {:?}", result.err());
}
