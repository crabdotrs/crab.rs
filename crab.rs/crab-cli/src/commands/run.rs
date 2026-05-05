use std::process::Command;
use crate::commands::build::cmd_build;

pub fn cmd_run() {
    println!("Running Crab project...");
    cmd_build();
    let output = Command::new("cargo")
        .arg("run")
        .current_dir(".crab_cache")
        .output()
        .expect("Failed to run cargo run");
    println!("{}", String::from_utf8_lossy(&output.stdout));
    if !output.status.success() {
        eprintln!("{}", String::from_utf8_lossy(&output.stderr));
    }
}
