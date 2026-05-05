use crab_core::Manifest;
use std::fs;

pub fn cmd_new(name: &str) {
    println!("Creating new Crab project: {}", name);
    fs::create_dir_all(name).expect("Failed to create directory");
    let manifest = Manifest {
        package: crab_core::Package {
            name: name.to_string(),
            version: "0.1.0".to_string(),
            edition: "2024".to_string(),
            authors: Some(vec!["crabdotrs".to_string()]),
            description: Some(format!("{} - A Crab language project", name)),
            license: Some("MIT".to_string()),
            repository: None,
        },
        dependencies: {
            let mut deps = std::collections::HashMap::new();
            deps.insert(
                "tokio".to_string(),
                crab_core::Dependency::Detailed(crab_core::DetailedDependency {
                    version: Some("1".to_string()),
                    features: Some(vec!["full".to_string()]),
                    optional: None,
                    default_features: None,
                    path: None,
                    git: None,
                    branch: None,
                    tag: None,
                }),
            );
            deps.insert(
                "futures".to_string(),
                crab_core::Dependency::Simple("0.3".to_string()),
            );
            deps
        },
        features: std::collections::HashMap::new(),
        profile: crab_core::Profiles::default(),
    };
    let manifest_toml = manifest.to_string().expect("Failed to serialize manifest");
    fs::write(format!("{}/crab.toml", name), manifest_toml).expect("Failed to write crab.toml");
    fs::create_dir_all(format!("{}/src", name)).expect("Failed to create src directory");
    let hello_crab = r#"void main() {
    print("Hello, Crab!");
}
"#;
    fs::write(format!("{}/src/main.crab", name), hello_crab).expect("Failed to write main.crab");
    println!("[OK] Project '{}' created successfully!", name);
    println!("  cd {}", name);
    println!("  crab build");
}
