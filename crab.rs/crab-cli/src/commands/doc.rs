use std::process::Command;

pub fn cmd_doc() {
    println!("Generating documentation...");
    let output = Command::new("cargo")
        .arg("doc")
        .arg("--no-deps")
        .current_dir(".crab_cache")
        .output()
        .expect("Failed to run cargo doc");
    if output.status.success() {
        println!("[OK] Documentation generated in .crab_cache/target/doc/");
    } else {
        eprintln!("[FAIL] Documentation generation failed!");
        eprintln!("{}", String::from_utf8_lossy(&output.stderr));
    }
}
