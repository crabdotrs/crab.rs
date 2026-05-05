use std::fs;
use std::io::Write;
use std::path::Path;
use crab_core::Manifest;

pub fn cmd_add(package: &str) {
    println!("Adding dependency: {}", package);
    if let Err(e) = Manifest::from_file(Path::new("crab.toml")) {
        eprintln!("[FAIL] Error reading crab.toml: {}", e);
        return;
    };
    let version = "*";
    let dependency_string = format!("{} = \"{}\"", package, version);
    match fs::OpenOptions::new().append(true).open("crab.toml") {
        Ok(mut file) => {
            writeln!(file, "{}", dependency_string).expect("Failed to write to crab.toml");
            println!("[OK] Added {} = \"{}\" to crab.toml", package, version);
        }
        Err(e) => {
            eprintln!("[FAIL] Error updating crab.toml: {}", e);
        }
    }
}
