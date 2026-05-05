use crate::c_compiler::CCompiler;
use crate::utils::fs::glob_crab_files;
use crab_codegen::CodeGenerator;
use crab_core::Manifest;
use crab_parser::{Parser as CrabParser, TopLevelItem};
use std::fs;
use std::path::Path;
use std::process::Command;

pub fn cmd_build() {
    println!("Building Crab project...");
    let mut manifest = match Manifest::from_file(Path::new("crab.toml")) {
        Ok(m) => m,
        Err(e) => {
            eprintln!("[FAIL] Error reading crab.toml: {}", e);
            return;
        }
    };
    let source_files = glob_crab_files("src");
    let cache_dir = ".crab_cache";
    fs::create_dir_all(cache_dir).expect("Failed to create .crab_cache directory");
    fs::create_dir_all(format!("{}/src", cache_dir)).expect("Failed to create cache src directory");
    let mut c_compiler = CCompiler::new(Path::new(cache_dir));
    let mut cblock_index = 0;
    let mut all_imports = Vec::new();
    let mut has_classes = false;
    for source_file in &source_files {
        match extract_imports_and_cblocks_from_file(
            source_file,
            &mut c_compiler,
            &mut cblock_index,
            &mut all_imports,
        ) {
            Ok(file_has_classes) => {
                if file_has_classes {
                    has_classes = true;
                }
            }
            Err(e) => {
                eprintln!("[FAIL] Error processing {}: {}", source_file, e);
            }
        }
    }
    if has_classes {
        if !manifest.dependencies.contains_key("serde") {
            use crab_core::{Dependency, DetailedDependency};
            manifest.dependencies.insert(
                "serde".to_string(),
                Dependency::Detailed(DetailedDependency {
                    version: Some("1.0".to_string()),
                    features: Some(vec!["derive".to_string()]),
                    optional: None,
                    default_features: None,
                    path: None,
                    git: None,
                    branch: None,
                    tag: None,
                }),
            );
        }
        if !manifest.dependencies.contains_key("serde_json") {
            manifest.add_dependency("serde_json", "1.0");
        }
        if !manifest.dependencies.contains_key("lazy_static") {
            manifest.add_dependency("lazy_static", "1.4.0");
        }
        if !manifest.dependencies.contains_key("chrono") {
            use crab_core::{Dependency, DetailedDependency};
            manifest.dependencies.insert(
                "chrono".to_string(),
                Dependency::Detailed(DetailedDependency {
                    version: Some("0.4".to_string()),
                    features: Some(vec!["serde".to_string()]),
                    optional: None,
                    default_features: None,
                    path: None,
                    git: None,
                    branch: None,
                    tag: None,
                }),
            );
        }
    }
    let mut has_serde = false;
    for import in &all_imports {
        match import.as_str() {
            "actix-web" => {
                if !manifest.dependencies.contains_key("actix-web") {
                    manifest.add_dependency("actix-web", "*");
                }
            }
            "serde" => {
                has_serde = true;
                use crab_core::{Dependency, DetailedDependency};
                manifest.dependencies.insert(
                    "serde".to_string(),
                    Dependency::Detailed(DetailedDependency {
                        version: Some("*".to_string()),
                        features: Some(vec!["derive".to_string()]),
                        optional: None,
                        default_features: None,
                        path: None,
                        git: None,
                        branch: None,
                        tag: None,
                    }),
                );
            }
            "serde_json" => {
                if !manifest.dependencies.contains_key("serde_json") {
                    manifest.add_dependency("serde_json", "*");
                }
            }
            "tokio" => continue,
            _ => {
                if import.starts_with("std::") {
                    continue;
                }
                if import.ends_with(".crab") || import.contains('/') || import.contains("\\") {
                    continue;
                }
                if !manifest.dependencies.contains_key(import) {
                    manifest.add_dependency(import, "*");
                }
            }
        };
    }
    if has_serde && !manifest.dependencies.contains_key("serde_json") {
        manifest.add_dependency("serde_json", "*");
    }
    for source_file in &source_files {
        match transpile_file(source_file, cache_dir) {
            Ok(_) => println!("[OK] Transpiled {}", source_file),
            Err(e) => eprintln!("[FAIL] Error transpiling {}: {}", source_file, e),
        }
    }

    // Add module declarations to main.rs
    let main_rs_path = format!("{}/main.rs", cache_dir);
    if Path::new(&main_rs_path).exists() {
        let mut mod_decls = String::new();
        for source_file in &source_files {
            let stem = Path::new(source_file)
                .file_stem()
                .unwrap()
                .to_str()
                .unwrap();
            if stem != "main" {
                mod_decls.push_str(&format!("pub mod {};\n", stem));
            }
        }
        if !mod_decls.is_empty() {
            let main_content = fs::read_to_string(&main_rs_path).unwrap_or_default();
            let new_main = format!("{}\n{}", mod_decls, main_content);
            fs::write(&main_rs_path, new_main).expect("Failed to update main.rs");
        }
    }
    let cargo_toml = manifest.to_cargo_toml();
    fs::write(format!("{}/Cargo.toml", cache_dir), cargo_toml).expect("Failed to write Cargo.toml");
    let c_lib = match c_compiler.compile_c_code() {
        Ok(lib) => lib,
        Err(e) => {
            eprintln!("[FAIL] C compilation failed: {}", e);
            None
        }
    };
    if let Some(lib_path) = c_lib {
        let build_rs = c_compiler.generate_build_script();
        fs::write(format!("{}/build.rs", cache_dir), build_rs).expect("Failed to write build.rs");
        let lib_dest = Path::new(cache_dir).join("libcblock.a");
        fs::copy(&lib_path, lib_dest).ok();
    }
    let output = Command::new("cargo")
        .arg("build")
        .current_dir(".crab_cache")
        .output()
        .expect("Failed to run cargo build");
    if output.status.success() {
        println!("[OK] Build successful!");
    } else {
        eprintln!("[FAIL] Build failed!");
        eprintln!("{}", String::from_utf8_lossy(&output.stderr));
    }
}

fn extract_imports_and_cblocks_from_file(
    source: &str,
    c_compiler: &mut CCompiler,
    cblock_index: &mut usize,
    imports: &mut Vec<String>,
) -> Result<bool, Box<dyn std::error::Error>> {
    let content = fs::read_to_string(source)?;
    let mut parser = CrabParser::new(&content)?;
    let ast = parser.parse()?;
    let mut has_classes = false;
    for item in &ast.items {
        match item {
            TopLevelItem::Import(import_stmt) => {
                if import_stmt.path.ends_with(".h") {
                    c_compiler.process_header_import(import_stmt)?;
                }
                imports.push(import_stmt.path.clone());
            }
            TopLevelItem::CBlock(cblock) => {
                c_compiler.process_cblock(cblock, *cblock_index)?;
                *cblock_index += 1;
            }
            TopLevelItem::ClassDecl(_) => {
                has_classes = true;
            }
            _ => {}
        }
    }
    Ok(has_classes)
}

pub fn transpile_file(
    source: &str,
    output_dir: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let content = fs::read_to_string(source)?;
    let mut parser = CrabParser::new(&content)?;
    let ast = match parser.parse() {
        Ok(ast) => ast,
        Err(e) => {
            eprintln!("Parse error in {}: {:?}", source, e);
            return Err(e.into());
        }
    };
    let mut codegen = CodeGenerator::new();
    let rust_code = codegen.generate(&ast);
    let source_path = Path::new(source);
    let stem = source_path.file_stem().unwrap().to_str().unwrap();
    let output_filename = if stem == "main" {
        "main.rs"
    } else {
        &format!("{}.rs", stem)
    };
    let output_path = format!("{}/{}", output_dir, output_filename);
    fs::write(&output_path, rust_code)?;
    Ok(output_path)
}
