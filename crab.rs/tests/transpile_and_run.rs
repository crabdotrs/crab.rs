use std::process::Command;
use std::path::PathBuf;

fn crab_cli_path() -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("target");
    path.push("release");
    path.push("crab");
    path
}

fn examples_dir() -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("..");
    path.push("examples");
    path
}

#[test]
fn test_01_hello_world() {
    let crab = crab_cli_path();
    let example = examples_dir().join("01_hello_world.crab");

    let output = Command::new(crab)
        .arg("build")
        .arg(&example)
        .output()
        .expect("Failed to execute crab build");

    assert!(output.status.success(), "Hello world should compile successfully");
}

#[test]
fn test_02_variables() {
    let crab = crab_cli_path();
    let example = examples_dir().join("02_variables.crab");

    let output = Command::new(crab)
        .arg("build")
        .arg(&example)
        .output()
        .expect("Failed to execute crab build");

    assert!(output.status.success(), "Variables example should compile");
}

#[test]
fn test_03_null_safety() {
    let crab = crab_cli_path();
    let example = examples_dir().join("03_null_safety.crab");

    let output = Command::new(crab)
        .arg("build")
        .arg(&example)
        .output()
        .expect("Failed to execute crab build");

    assert!(output.status.success(), "Null safety example should compile");
}

#[test]
fn test_04_functions() {
    let crab = crab_cli_path();
    let example = examples_dir().join("04_functions.crab");

    let output = Command::new(crab)
        .arg("build")
        .arg(&example)
        .output()
        .expect("Failed to execute crab build");

    assert!(output.status.success(), "Functions example should compile");
}

#[test]
fn test_06_classes() {
    let crab = crab_cli_path();
    let example = examples_dir().join("06_classes.crab");

    let output = Command::new(crab)
        .arg("build")
        .arg(&example)
        .output()
        .expect("Failed to execute crab build");

    assert!(output.status.success(), "Classes example should compile");
}
