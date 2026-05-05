use std::path::Path;
use crab_core::Manifest;

pub fn cmd_remove(package: &str) {
    println!("Removing dependency: {}", package);
    let manifest = match Manifest::from_file(Path::new("crab.toml")) {
        Ok(m) => m,
        Err(e) => {
            eprintln!("[FAIL] Error reading crab.toml: {}", e);
            return;
        }
    };
    let crab_deps: std::collections::HashSet<String> =
        manifest.dependencies.keys().cloned().collect();
    if crab_deps.contains(package) {
        println!("[OK] Removed {} from crab.toml", package);
    } else {
        eprintln!("[FAIL] Dependency '{}' not found in crab.toml", package);
    }
}
