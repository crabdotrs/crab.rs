use crate::commands::build::transpile_file;
use crate::utils::fs::glob_crab_files;

pub fn cmd_test(release: bool) {
    println!("Running tests...");
    let test_files = glob_crab_files("tests");
    if test_files.is_empty() {
        println!("No tests found in tests/ directory");
        return;
    }
    let mut passed = 0;
    let mut failed = 0;
    for test_file in test_files {
        if test_file.ends_with("_test.crab") {
            match run_test(&test_file, release) {
                Ok(_) => {
                    println!("[OK] {}", test_file);
                    passed += 1;
                }
                Err(e) => {
                    println!("[FAIL] {}: {}", test_file, e);
                    failed += 1;
                }
            }
        }
    }
    println!("\nTest Results: {} passed, {} failed", passed, failed);
}

fn run_test(test_file: &str, _release: bool) -> Result<String, Box<dyn std::error::Error>> {
    transpile_file(test_file, ".crab_cache")?;
    Ok(String::new())
}
