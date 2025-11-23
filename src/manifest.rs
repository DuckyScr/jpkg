use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct Manifest {
    pub package: PackageInfo,
    #[serde(default)]
    pub dependencies: HashMap<String, String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PackageInfo {
    pub name: String,
    pub version: String,
    pub description: Option<String>,
}

impl Manifest {
    pub fn new(name: &str, version: &str) -> Self {
        Manifest {
            package: PackageInfo {
                name: name.to_string(),
                version: version.to_string(),
                description: None,
            },
            dependencies: HashMap::new(),
        }
    }
}
