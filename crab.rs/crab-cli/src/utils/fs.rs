use std::fs;

pub fn glob_crab_files(dir: &str) -> Vec<String> {
    let mut files = Vec::new();
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map(|e| e == "crab").unwrap_or(false) {
                files.push(path.to_string_lossy().to_string());
            } else if path.is_dir() {
                files.extend(glob_crab_files(&path.to_string_lossy()));
            }
        }
    }
    files
}
