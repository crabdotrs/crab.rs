use serde::{Deserialize, Serialize};
use std::collections::HashMap;
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct Manifest {
    pub package: Package,
    #[serde(default)]
    pub dependencies: HashMap<String, Dependency>,
    #[serde(default)]
    pub features: HashMap<String, Vec<String>>,
    #[serde(default)]
    pub profile: Profiles,
}
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct Package {
    pub name: String,
    #[serde(default = "default_version")]
    pub version: String,
    #[serde(default = "default_edition")]
    pub edition: String,
    pub authors: Option<Vec<String>>,
    pub description: Option<String>,
    pub license: Option<String>,
    pub repository: Option<String>,
}
fn default_version() -> String {
    "0.1.0".to_string()
}
fn default_edition() -> String {
    "2024".to_string()
}
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(untagged)]
pub enum Dependency {
    Simple(String),
    Detailed(DetailedDependency),
}
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct DetailedDependency {
    pub version: Option<String>,
    pub features: Option<Vec<String>>,
    pub optional: Option<bool>,
    pub default_features: Option<bool>,
    pub path: Option<String>,
    pub git: Option<String>,
    pub branch: Option<String>,
    pub tag: Option<String>,
}
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Default)]
pub struct Profiles {
    pub dev: Option<Profile>,
    pub release: Option<Profile>,
}
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct Profile {
    #[serde(rename = "opt-level")]
    pub opt_level: Option<i32>,
    pub lto: Option<bool>,
    pub debug: Option<bool>,
    #[serde(rename = "debug-assertions")]
    pub debug_assertions: Option<bool>,
}
impl Manifest {
    pub fn from_str(content: &str) -> Result<Self, toml::de::Error> {
        toml::from_str(content)
    }
    pub fn from_file(path: &std::path::Path) -> Result<Self, Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(path)?;
        Ok(Self::from_str(&content)?)
    }
    pub fn to_cargo_toml(&self) -> String {
        let mut output = String::new();
        output.push_str("[package]\n");
        output.push_str(&format!("name = \"{}\"\n", self.package.name));
        output.push_str(&format!("version = \"{}\"\n", self.package.version));
        output.push_str(&format!("edition = \"{}\"\n", self.package.edition));
        if let Some(ref authors) = self.package.authors {
            output.push_str(&format!("authors = {:?}\n", authors));
        }
        if let Some(ref desc) = self.package.description {
            output.push_str(&format!("description = \"{}\"\n", desc));
        }
        if let Some(ref license) = self.package.license {
            output.push_str(&format!("license = \"{}\"\n", license));
        }
        if let Some(ref repo) = self.package.repository {
            output.push_str(&format!("repository = \"{}\"\n", repo));
        }
        output.push('\n');
        output.push_str("[[bin]]\n");
        output.push_str(&format!("name = \"{}\"\n", self.package.name));
        output.push_str("path = \"main.rs\"\n");
        output.push('\n');
        output.push_str("[dependencies]\n");
        output.push_str("tokio = { version = \"1.0\", features = [\"full\"] }\n");
        for (name, dep) in &self.dependencies {
            if name == "tokio" {
                continue;
            }
            match dep {
                Dependency::Simple(version) => {
                    output.push_str(&format!("{} = \"{}\"\n", name, version));
                }
                Dependency::Detailed(details) => {
                    output.push_str(&format!("{} = ", name));
                    let mut parts = Vec::new();
                    if let Some(ref v) = details.version {
                        parts.push(format!("version = \"{}\"", v));
                    }
                    if let Some(ref f) = details.features {
                        parts.push(format!("features = {:?}", f));
                    }
                    if let Some(true) = details.optional {
                        parts.push("optional = true".to_string());
                    }
                    if let Some(ref p) = details.path {
                        parts.push(format!("path = \"{}\"", p));
                    }
                    if let Some(ref g) = details.git {
                        parts.push(format!("git = \"{}\"", g));
                    }
                    output.push_str("{");
                    output.push_str(&parts.join(", "));
                    output.push_str("}\n");
                }
            }
        }
        output.push('\n');
        if !self.features.is_empty() {
            output.push_str("[features]\n");
            for (name, features) in &self.features {
                output.push_str(&format!("{} = {:?}\n", name, features));
            }
            output.push('\n');
        }
        if let Some(ref dev) = self.profile.dev {
            output.push_str("[profile.dev]\n");
            if let Some(level) = dev.opt_level {
                output.push_str(&format!("opt-level = {}\n", level));
            }
            if let Some(lto) = dev.lto {
                output.push_str(&format!("lto = {}\n", lto));
            }
            output.push('\n');
        }
        if let Some(ref release) = self.profile.release {
            output.push_str("[profile.release]\n");
            if let Some(level) = release.opt_level {
                output.push_str(&format!("opt-level = {}\n", level));
            }
            if let Some(lto) = release.lto {
                output.push_str(&format!("lto = {}\n", lto));
            }
            output.push('\n');
        }
        output
    }
    pub fn add_dependency(&mut self, name: &str, version: &str) {
        self.dependencies
            .insert(name.to_string(), Dependency::Simple(version.to_string()));
    }
    pub fn remove_dependency(&mut self, name: &str) -> bool {
        self.dependencies.remove(name).is_some()
    }
    pub fn to_string(&self) -> Result<String, toml::ser::Error> {
        toml::to_string(self)
    }
}
impl Default for Package {
    fn default() -> Self {
        Package {
            name: "unnamed".to_string(),
            version: default_version(),
            edition: default_edition(),
            authors: None,
            description: None,
            license: None,
            repository: None,
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_parse_simple_manifest() {
        let toml = r#"
[package]
name = "myapp"
version = "0.1.0"
[dependencies]
tokio = "1"
"#;
        let manifest = Manifest::from_str(toml).unwrap();
        assert_eq!(manifest.package.name, "myapp");
        assert_eq!(manifest.package.version, "0.1.0");
        assert!(manifest.dependencies.contains_key("tokio"));
    }
    #[test]
    fn test_to_cargo_toml() {
        let manifest = Manifest {
            package: Package {
                name: "test".to_string(),
                version: "0.1.0".to_string(),
                edition: "2024".to_string(),
                authors: Some(vec!["crabdotrs".to_string()]),
                description: None,
                license: None,
                repository: None,
            },
            dependencies: {
                let mut deps = HashMap::new();
                deps.insert("tokio".to_string(), Dependency::Simple("1".to_string()));
                deps
            },
            features: HashMap::new(),
            profile: Profiles::default(),
        };
        let cargo = manifest.to_cargo_toml();
        assert!(cargo.contains("name = \"test\""));
        assert!(cargo.contains("tokio"));
    }
}
