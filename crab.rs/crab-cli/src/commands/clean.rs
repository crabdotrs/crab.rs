use std::fs;

pub fn cmd_clean() {
    println!("Cleaning...");
    let _ = fs::remove_dir_all(".crab_cache");
    println!("Clean complete!");
}
