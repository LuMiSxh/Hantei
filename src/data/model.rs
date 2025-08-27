use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;

/// Represents the runtime data structure, matching the expected JSON format for evaluation.
#[derive(Serialize, Deserialize, Debug)]
pub struct SampleData {
    pub static_data: HashMap<String, f64>,
    pub dynamic_data: HashMap<String, Vec<HashMap<String, f64>>>,
}

impl SampleData {
    /// Load sample data from a JSON file.
    pub fn from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        let data = serde_json::from_str(&content)?;
        Ok(data)
    }

    /// Creates default mock data when no file is provided.
    pub fn default() -> Self {
        let mut static_data = HashMap::new();
        static_data.insert("Leading width".to_string(), 1970.0);
        static_data.insert("Trailing width".to_string(), 1965.0);

        let mut dynamic_data = HashMap::new();
        let mut hole_event = HashMap::new();
        hole_event.insert("Diameter".to_string(), 30.0);
        dynamic_data.insert("hole".to_string(), vec![hole_event]);

        Self {
            static_data,
            dynamic_data,
        }
    }

    /// Get a reference to the static data.
    pub fn static_data(&self) -> &HashMap<String, f64> {
        &self.static_data
    }

    /// Get a reference to the dynamic data.
    pub fn dynamic_data(&self) -> &HashMap<String, Vec<HashMap<String, f64>>> {
        &self.dynamic_data
    }
}
