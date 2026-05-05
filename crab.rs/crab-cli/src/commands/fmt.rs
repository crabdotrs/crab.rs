use std::fs;
use crate::utils::fs::glob_crab_files;

pub fn cmd_fmt() {
    println!("Formatting Crab source code...");
    let source_files = glob_crab_files("src");
    for file in source_files {
        match fs::read_to_string(&file) {
            Ok(content) => {
                let formatted: String = content
                    .lines()
                    .map(|line| line.trim_end())
                    .collect::<Vec<_>>()
                    .join("\n");
                let formatted = format!("{}\n", formatted);
                if let Err(e) = fs::write(&file, formatted) {
                    eprintln!("[FAIL] Error formatting {}: {}", file, e);
                } else {
                    println!("[OK] Formatted {}", file);
                }
            }
            Err(e) => {
                eprintln!("[FAIL] Error reading {}: {}", file, e);
            }
        }
    }
    println!("Formatting complete!");
}
