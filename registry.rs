use serde::Deserialize;

const REGISTRY_URL: &str = "https://registry.finix-lang.org/api/v1";

#[derive(Debug, Deserialize)]
pub struct RegistryResponse {
    pub name: String,
    pub latest_version: String,
    pub download_url: String,
}

pub struct RegistryClient;

impl RegistryClient {
    /// Fetches package metadata from the global Finix registry.
    pub fn fetch_package_info(name: &str) -> Result<RegistryResponse, String> {
        let url = format!("{}/packages/{}", REGISTRY_URL, name);
        
        // Using reqwest::blocking for simplistic CLI tool logic
        reqwest::blocking::get(&url)
            .map_err(|e| format!("Network error: {}", e))?
            .json::<RegistryResponse>()
            .map_err(|e| format!("Failed to parse registry response: {}", e))
    }
    
    /// Downloads and unpacks a specific package version to the local cache.
    pub fn download_package(name: &str, version: &str) -> Result<(), String> {
        println!("Downloading {} v{} from registry...", name, version);
        // TODO: Implement actual .tar.gz download and extraction into ~/.finix/cache/
        Ok(())
    }
}

pub struct Module {
    pub name: String,
    pub exports: std::collections::HashMap<String, crate::value::Value>,
}

impl Module {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            exports: std::collections::HashMap::new(),
        }
    }

    pub fn register_const(&mut self, name: &str, value: crate::value::Value) {
        self.exports.insert(name.to_string(), value);
    }

    pub fn register_native(&mut self, name: &str, func: fn(&[crate::value::Value]) -> Result<crate::value::Value, String>) {
        self.exports.insert(name.to_string(), crate::value::Value::NativeFunction(func));
    }
}