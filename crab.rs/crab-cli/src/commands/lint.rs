use std::fs;
use crate::commands::build::transpile_file;
use crate::utils::fs::glob_crab_files;

pub fn cmd_lint() {
    println!("Linting Crab source code...");
    let source_files = glob_crab_files("src");
    let mut issues_found = 0;
    for file in source_files {
        match fs::read_to_string(&file) {
            Ok(content) => {
                let lines: Vec<&str> = content.lines().collect();
                for (i, line) in lines.iter().enumerate() {
                    let line_num = i + 1;
                    if line.len() > 100 {
                        println!("[WARN] {}:{}: Line exceeds 100 characters", file, line_num);
                        issues_found += 1;
                    }
                    if line.ends_with(' ') || line.ends_with('\t') {
                        println!("[WARN] {}:{}: Trailing whitespace", file, line_num);
                        issues_found += 1;
                    }
                }
                match transpile_file(&file, ".crab_cache") {
                    Ok(_) => {}
                    Err(e) => {
                        println!("[FAIL] {}: {}", file, e);
                        issues_found += 1;
                    }
                }
            }
            Err(e) => {
                eprintln!("[FAIL] Error reading {}: {}", file, e);
                issues_found += 1;
            }
        }
    }
    if issues_found == 0 {
        println!("[OK] No issues found");
    } else {
        println!("\nFound {} issue(s)", issues_found);
    }
}
