use std::path::Path;
use std::process::Command;
use crab_core::Manifest;

pub fn cmd_publish() {
    println!("Packaging for publication...");
    let manifest = match Manifest::from_file(Path::new("crab.toml")) {
        Ok(m) => m,
        Err(e) => {
            eprintln!("[FAIL] Error reading crab.toml: {}", e);
            return;
        }
    };
    let package_name = manifest.package.name;
    let version = manifest.package.version;
    let output = Command::new("tar")
        .args(&["-czf", &format!("{}-{}.tar.gz", package_name, version)])
        .args(&["-C", "."])
        .args(&["src/", "crab.toml", "README.md"])
        .output();
    match output {
        Ok(_) => {
            println!("[OK] Package created: {}-{}.tar.gz", package_name, version);
            println!("  (Note: Connect to registry for actual publishing)");
        }
        Err(e) => {
            eprintln!("[FAIL] Packaging failed: {}", e);
        }
    }
}
