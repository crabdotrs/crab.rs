use std::process::Command;
use std::path::PathBuf;
use std::fs;

fn crab_cli_path() -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("target");
    path.push("release");
    path.push("crab");
    path
}

#[test]
fn test_cblock_compilation() {
    let crab = crab_cli_path();
    let temp_dir = std::env::temp_dir();
    let test_file = temp_dir.join("test_cblock.crab");

    let source = r#"
CBlock {
  #include <stdio.h>
  int test_add(int a, int b) {
    return a + b;
  }
}

void main() {
  print("CBlock compilation test");
}
"#;

    fs::write(&test_file, source).expect("Failed to write test file");

    let output = Command::new(crab)
        .arg("build")
        .arg(&test_file)
        .output()
        .expect("Failed to execute crab build");

    let stderr = String::from_utf8_lossy(&output.stderr);
    println!("CBlock compilation stderr: {}", stderr);

    assert!(output.status.success(), "CBlock should compile successfully");

    fs::remove_file(test_file).ok();
}

#[test]
fn test_c_header_import() {
    let crab = crab_cli_path();
    let temp_dir = std::env::temp_dir();
    let test_file = temp_dir.join("test_c_header.crab");

    let source = r#"
CHello stdio.h;

void main() {
  print("C header import test");
}
"#;

    fs::write(&test_file, source).expect("Failed to write test file");

    let output = Command::new(crab)
        .arg("build")
        .arg(&test_file)
        .output()
        .expect("Failed to execute crab build");

    let stderr = String::from_utf8_lossy(&output.stderr);
    println!("C header import stderr: {}", stderr);

    fs::remove_file(test_file).ok();
}
