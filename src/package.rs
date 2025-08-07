use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageJson {
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    pub main: Option<String>,
    pub dependencies: Option<HashMap<String, String>>,
    pub dev_dependencies: Option<HashMap<String, String>>,
    pub scripts: Option<HashMap<String, String>>,
    pub author: Option<String>,
    pub license: Option<String>,
}

impl Default for PackageJson {
    fn default() -> Self {
        Self {
            name: "zano-app".to_string(),
            version: "1.0.0".to_string(),
            description: None,
            main: Some("index.zn".to_string()),
            dependencies: Some(HashMap::new()),
            dev_dependencies: Some(HashMap::new()),
            scripts: Some({
                let mut scripts = HashMap::new();
                scripts.insert("start".to_string(), "zano index.zn".to_string());
                scripts
            }),
            author: None,
            license: Some("MIT".to_string()),
        }
    }
}

pub struct PackageManager {
    project_root: PathBuf,
}

impl PackageManager {
    pub fn new<P: AsRef<Path>>(project_root: P) -> Self {
        Self {
            project_root: project_root.as_ref().to_path_buf(),
        }
    }

    pub async fn init(&self) -> Result<()> {
        let package_json_path = self.project_root.join("package.json");
        
        if package_json_path.exists() {
            println!("package.json already exists");
            return Ok(());
        }

        let package = PackageJson::default();
        let json_content = serde_json::to_string_pretty(&package)?;
        
        tokio::fs::write(&package_json_path, json_content).await?;
        println!("Created package.json");
        
        Ok(())
    }

    pub async fn install(&self, package_name: Option<String>) -> Result<()> {
        let package_json_path = self.project_root.join("package.json");
        
        if !package_json_path.exists() {
            return Err(anyhow::anyhow!("No package.json found. Run 'zano init' first."));
        }

        let mut package: PackageJson = {
            let content = tokio::fs::read_to_string(&package_json_path).await?;
            serde_json::from_str(&content)?
        };

        if let Some(name) = package_name {
            // Add dependency
            if package.dependencies.is_none() {
                package.dependencies = Some(HashMap::new());
            }
            
            if let Some(ref mut deps) = package.dependencies {
                // For now, we'll use a simple version strategy
                deps.insert(name.clone(), "^1.0.0".to_string());
                println!("Added {} to dependencies", name);
            }

            // Update package.json
            let json_content = serde_json::to_string_pretty(&package)?;
            tokio::fs::write(&package_json_path, json_content).await?;
        }

        // Create zano_modules directory
        let modules_dir = self.project_root.join("zano_modules");
        if !modules_dir.exists() {
            tokio::fs::create_dir_all(&modules_dir).await?;
        }

        println!("Dependencies installed successfully");
        Ok(())
    }

    pub async fn run_script(&self, script_name: &str) -> Result<()> {
        let package_json_path = self.project_root.join("package.json");
        
        if !package_json_path.exists() {
            return Err(anyhow::anyhow!("No package.json found"));
        }

        let package: PackageJson = {
            let content = tokio::fs::read_to_string(&package_json_path).await?;
            serde_json::from_str(&content)?
        };

        if let Some(scripts) = &package.scripts {
            if let Some(command) = scripts.get(script_name) {
                println!("Running script '{}': {}", script_name, command);
                
                // Execute the command
                let output = tokio::process::Command::new("sh")
                    .arg("-c")
                    .arg(command)
                    .current_dir(&self.project_root)
                    .output()
                    .await?;

                if output.status.success() {
                    println!("{}", String::from_utf8_lossy(&output.stdout));
                } else {
                    eprintln!("Script failed: {}", String::from_utf8_lossy(&output.stderr));
                }
            } else {
                return Err(anyhow::anyhow!("Script '{}' not found in package.json", script_name));
            }
        } else {
            return Err(anyhow::anyhow!("No scripts defined in package.json"));
        }

        Ok(())
    }

    pub async fn load_package(&self) -> Result<PackageJson> {
        let package_json_path = self.project_root.join("package.json");
        
        if !package_json_path.exists() {
            return Ok(PackageJson::default());
        }

        let content = tokio::fs::read_to_string(&package_json_path).await?;
        Ok(serde_json::from_str(&content)?)
    }

    pub fn resolve_module(&self, module_name: &str) -> Option<PathBuf> {
        // First check built-in modules
        match module_name {
            "fs" | "http" | "path" | "console" => return Some(PathBuf::from(format!("builtin:{}", module_name))),
            _ => {}
        }

        // Check zano_modules
        let modules_dir = self.project_root.join("zano_modules").join(module_name);
        if modules_dir.exists() {
            return Some(modules_dir);
        }

        // Check relative path
        let relative_path = self.project_root.join(format!("{}.zn", module_name));
        if relative_path.exists() {
            return Some(relative_path);
        }

        None
    }
}