use anyhow::Result;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Serialize, Deserialize)]
pub struct LockFile {
    pub version: String,
    pub packages: HashMap<String, LockedPackage>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LockedPackage {
    pub version: String,
    pub checksum: String,
    pub dependencies: Vec<String>,
}

impl LockFile {
    pub fn new() -> Self {
        Self {
            version: "1".to_string(),
            packages: HashMap::new(),
        }
    }

    #[allow(dead_code)]
    pub fn load() -> Result<Self> {
        if !Path::new("jpkg.lock").exists() {
            return Ok(Self::new());
        }
        let content = fs::read_to_string("jpkg.lock")?;
        Ok(serde_json::from_str(&content)?)
    }

    pub fn save(&self) -> Result<()> {
        let content = serde_json::to_string_pretty(self)?;
        fs::write("jpkg.lock", content)?;
        Ok(())
    }

    pub fn add_package(
        &mut self,
        key: String,
        version: String,
        jar_path: &Path,
        dependencies: Vec<String>,
    ) -> Result<()> {
        let checksum = if jar_path.exists() {
            calculate_sha256(jar_path)?
        } else {
            String::new()
        };

        self.packages.insert(
            key,
            LockedPackage {
                version,
                checksum,
                dependencies,
            },
        );
        Ok(())
    }

    #[allow(dead_code)]
    pub fn get_locked_version(&self, key: &str) -> Option<&str> {
        self.packages.get(key).map(|p| p.version.as_str())
    }

    pub fn verify_package(&self, key: &str, jar_path: &Path) -> Result<bool> {
        if let Some(locked) = self.packages.get(key) {
            if locked.checksum.is_empty() {
                // No checksum to verify
                return Ok(true);
            }
            if jar_path.exists() {
                let checksum = calculate_sha256(jar_path)?;
                Ok(checksum == locked.checksum)
            } else {
                Ok(false)
            }
        } else {
            Ok(false)
        }
    }
}

fn calculate_sha256(path: &Path) -> Result<String> {
    let bytes = fs::read(path)?;
    let mut hasher = Sha256::new();
    hasher.update(&bytes);
    let result = hasher.finalize();
    Ok(hex::encode(result))
}
