use anyhow::Result;
use serde::Deserialize;
use serde_json::{Value, json};
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::Path;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("File error: {0}")]
    FileError(String),

    #[error("Parse error: {0}")]
    ParseError(String),
}

/// Dynamic configuration system similar to Viper (Go)
#[derive(Debug, Clone)]
pub struct Loader {
    data: Value,
    env_prefix: String,
}

impl Loader {
    /// Create a new empty config with optional environment prefix
    pub fn new(env_prefix: Option<&str>) -> Self {
        Loader {
            data: json!({}),
            env_prefix: env_prefix.unwrap_or("APP").to_uppercase(),
        }
    }

    /// Load configuration from YAML file
    #[allow(dead_code)]
    pub fn load_yaml<P: AsRef<Path>>(mut self, path: P) -> Result<Self> {
        let content =
            fs::read_to_string(path).map_err(|e| ConfigError::FileError(e.to_string()))?;

        let yaml_value: Value =
            serde_yaml::from_str(&content).map_err(|e| ConfigError::ParseError(e.to_string()))?;

        self.data = Self::merge_values(self.data, yaml_value);
        Ok(self)
    }

    /// Load configuration from TOML file
    #[allow(dead_code)]
    pub fn load_toml<P: AsRef<Path>>(mut self, path: P) -> Result<Self> {
        let content =
            fs::read_to_string(path).map_err(|e| ConfigError::FileError(e.to_string()))?;

        let toml_value: Value =
            toml::from_str(&content).map_err(|e| ConfigError::ParseError(e.to_string()))?;

        self.data = Self::merge_values(self.data, toml_value);
        Ok(self)
    }

    /// Load configuration from JSON file
    #[allow(dead_code)]
    pub fn load_json<P: AsRef<Path>>(mut self, path: P) -> Result<Self> {
        let content =
            fs::read_to_string(path).map_err(|e| ConfigError::FileError(e.to_string()))?;

        let json_value: Value =
            serde_json::from_str(&content).map_err(|e| ConfigError::ParseError(e.to_string()))?;

        self.data = Self::merge_values(self.data, json_value);
        Ok(self)
    }

    /// Load environment variables (supports nested keys with __)
    /// Example: APP_DATABASE__HOST becomes database.host
    pub fn load_env(mut self) -> Result<Self> {
        let env_vars: HashMap<String, String> = env::vars()
            .filter(|(key, _)| key.starts_with(&self.env_prefix))
            .collect();

        for (key, value) in env_vars {
            let config_key = key
                .strip_prefix(&format!("{}_", self.env_prefix))
                .unwrap_or(&key)
                .to_lowercase()
                .replace("__", ".");

            self.set(&config_key, &value);
        }
        Ok(self)
    }

    /// Load from .env file
    pub fn load_dotenv(self) -> Result<Self> {
        dotenvy::dotenv().ok();
        self.load_env()
    }

    /// Get raw JSON value
    #[allow(dead_code)]
    pub fn get(&self, key: &str) -> Result<Value> {
        let parts: Vec<&str> = key.split('.').collect();
        let mut current = &self.data;

        for part in parts {
            if current.is_null() {
                return Ok(Value::Null);
            }
            current = &current[part];
        }

        Ok(current.clone())
    }

    #[allow(dead_code)]
    pub fn set(&mut self, key: &str, value: &str) {
        let parts: Vec<&str> = key.split('.').collect();
        if parts.is_empty() {
            return;
        }

        let mut current = &mut self.data;
        for (i, part) in parts.iter().enumerate() {
            if i == parts.len() - 1 {
                let val = Self::parse_value(value);
                current[part] = val;
            } else {
                // Create intermediate objects if needed
                if current[part].is_null() {
                    current[part] = json!({});
                }
                current = &mut current[part];
            }
        }
    }

    /// Get entire configuration as JSON
    #[allow(dead_code)]
    pub fn as_json(&self) -> Value {
        self.data.clone()
    }

    /// Deserialize into a struct
    #[allow(dead_code)]
    pub fn deserialize<T: for<'de> Deserialize<'de>>(&self) -> Result<T> {
        serde_json::from_value(self.data.clone())
            .map_err(|e| ConfigError::ParseError(e.to_string()).into())
    }

    /// Deserialize a specific key into a struct
    #[allow(dead_code)]
    pub fn deserialize_key<T: for<'de> Deserialize<'de>>(&self, key: &str) -> Result<T> {
        let value = self.get(key)?;
        serde_json::from_value(value).map_err(|e| ConfigError::ParseError(e.to_string()).into())
    }

    /// Print configuration (pretty printed)
    #[allow(dead_code)]
    pub fn print(&self) {
        println!(
            "{}",
            serde_json::to_string_pretty(&self.data).unwrap_or_default()
        );
    }

    // Private helpers
    #[allow(dead_code)]
    fn parse_value(value: &str) -> Value {
        // Try to parse as JSON first (handles numbers, bools, arrays, objects)
        if let Ok(json_val) = serde_json::from_str(value) {
            return json_val;
        }

        // Try to parse as number
        if let Ok(num) = value.parse::<i64>() {
            return json!(num);
        }

        if let Ok(num) = value.parse::<f64>() {
            return json!(num);
        }

        // Try to parse as bool
        match value.to_lowercase().as_str() {
            "true" | "yes" | "1" => return json!(true),
            "false" | "no" | "0" => return json!(false),
            _ => {}
        }

        // Keep as string
        json!(value)
    }

    #[allow(dead_code)]
    fn merge_values(base: Value, override_val: Value) -> Value {
        match (base, override_val) {
            (Value::Object(mut base_map), Value::Object(override_map)) => {
                for (key, value) in override_map {
                    base_map.insert(
                        key.clone(),
                        Self::merge_values(
                            base_map.get(&key).cloned().unwrap_or(Value::Null),
                            value,
                        ),
                    );
                }
                Value::Object(base_map)
            }
            (_, override_val) => override_val,
        }
    }
}
