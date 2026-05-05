use crate::commands::build::transpile_file;
use crate::utils::fs::glob_crab_files;

pub fn cmd_check() {
    println!("Checking Crab project...");
    let source_files = glob_crab_files("src");
    for source_file in &source_files {
        match transpile_file(&source_file, ".crab_cache") {
            Ok(_) => println!("[OK] {}", source_file),
            Err(e) => eprintln!("[FAIL] {}: {}", source_file, e),
        }
    }
}
