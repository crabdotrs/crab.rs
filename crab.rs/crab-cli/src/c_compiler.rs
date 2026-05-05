use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use crab_parser::{CBlockDecl, ImportStmt};
pub struct CCompiler {
    cache_dir: PathBuf,
    c_files: Vec<PathBuf>,
    header_files: Vec<PathBuf>,
}
impl CCompiler {
    pub fn new(cache_dir: &Path) -> Self {
        CCompiler {
            cache_dir: cache_dir.to_path_buf(),
            c_files: Vec::new(),
            header_files: Vec::new(),
        }
    }
    pub fn process_cblock(
        &mut self,
        cblock: &CBlockDecl,
        index: usize,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let c_file = self.cache_dir.join(format!("cblock_{}.c", index));
        fs::write(&c_file, &cblock.code)?;
        println!("[OK] Processed CBlock -> {}", c_file.display());
        self.c_files.push(c_file);
        Ok(())
    }
    pub fn process_header_import(
        &mut self,
        import: &ImportStmt,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let header_path = &import.path;
        if header_path.ends_with(".h") {
            self.header_files.push(PathBuf::from(header_path));
            println!("[OK] Processing header import: {}", header_path);
            self.generate_bindings(header_path)?;
        }
        Ok(())
    }
    fn generate_bindings(&self, header_path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let bindings_file = self.cache_dir.join("bindings.rs");
        let wrapper_h = self.cache_dir.join("wrapper.h");
        let include_directive = if header_path.starts_with("<") && header_path.ends_with(">") {
            format!("#include {}\n", header_path)
        } else if header_path.starts_with("\"") && header_path.ends_with("\"") {
            format!("#include {}\n", header_path)
        } else {
            format!("#include <{}>\n", header_path)
        };
        fs::write(&wrapper_h, include_directive)?;
        println!("[INFO] Generating bindings for {}...", header_path);
        let output = Command::new("bindgen")
            .arg(&wrapper_h)
            .arg("--allowlist-function")
            .arg(".*")
            .arg("--allowlist-type")
            .arg(".*")
            .arg("--allowlist-var")
            .arg(".*")
            .arg("-o")
            .arg(&bindings_file)
            .output();
        match output {
            Ok(result) => {
                if result.status.success() {
                    println!("  [OK] Bindings generated: {}", bindings_file.display());
                    Ok(())
                } else {
                    println!("  [WARN] Bindgen failed, creating placeholder bindings");
                    self.create_placeholder_bindings(&bindings_file, header_path)?;
                    Ok(())
                }
            }
            Err(e) => {
                println!(
                    "  [WARN] Could not run bindgen ({}), creating placeholder",
                    e
                );
                self.create_placeholder_bindings(&bindings_file, header_path)?;
                Ok(())
            }
        }
    }
    fn create_placeholder_bindings(
        &self,
        output: &Path,
        _header: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let placeholder = r#"use std::os::raw::{c_int, c_double, c_char};
use std::ffi::{CStr, CString};
extern "C" {}
pub fn c_string(rust_str: &str) -> CString {
    CString::new(rust_str).expect("CString::new failed")
}
pub unsafe fn c_str_to_string(c_str: *const c_char) -> String {
    CStr::from_ptr(c_str).to_string_lossy().into_owned()
}
"#;
        fs::write(output, placeholder)?;
        Ok(())
    }
    pub fn compile_c_code(&self) -> Result<Option<PathBuf>, Box<dyn std::error::Error>> {
        if self.c_files.is_empty() {
            return Ok(None);
        }
        let lib_path = self.cache_dir.join("libcblock.a");
        println!("Compiling C code...");
        let mut objects = Vec::new();
        for (i, c_file) in self.c_files.iter().enumerate() {
            let obj_file = self.cache_dir.join(format!("cblock_{}.o", i));
            let output = Command::new("cc")
                .arg("-c")
                .arg(c_file)
                .arg("-o")
                .arg(&obj_file)
                .output()?;
            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                return Err(format!("Failed to compile {}: {}", c_file.display(), stderr).into());
            }
            objects.push(obj_file);
        }
        let mut ar_cmd = Command::new("ar");
        ar_cmd.arg("rcs").arg(&lib_path);
        for obj in &objects {
            ar_cmd.arg(obj);
        }
        let output = ar_cmd.output()?;
        if !output.status.success() {
            return Err("Failed to create static library".into());
        }
        println!("[OK] Created static library: {}", lib_path.display());
        Ok(Some(lib_path))
    }
    pub fn generate_build_script(&self) -> String {
        let mut script = String::new();
        if !self.c_files.is_empty() {
            script.push_str(
                r#"fn main() {
    println!("cargo:rustc-link-lib=static=cblock");
    println!("cargo:rustc-link-search=native=.");
}
"#,
            );
        }
        script
    }
    #[allow(dead_code)]
    pub fn has_c_code(&self) -> bool {
        !self.c_files.is_empty()
    }
}
