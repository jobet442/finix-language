use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Represents the `finix.toml` package manifest.
#[derive(Debug, Serialize, Deserialize)]
pub struct Manifest {
    pub package: PackageMeta,
    pub dependencies: Option<HashMap<String, String>>,
    pub dev_dependencies: Option<HashMap<String, String>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PackageMeta {
    pub name: String,
    pub version: String,
    pub authors: Option<Vec<String>>,
    pub description: Option<String>,
}

/// Represents the `finix.lock` resolved graph.
#[derive(Debug, Serialize, Deserialize)]
pub struct LockFile {
    pub version: u8,
    pub packages: Vec<LockedPackage>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LockedPackage {
    pub name: String,
    pub version: String,
    pub checksum: String,
    pub dependencies: Option<Vec<String>>,
}