//! Configuration for mx task runner

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

use crate::error::{Error, Result};

/// Configuration for mx task runner
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Runtime mappings: language -> command
    #[serde(default = "default_runtimes")]
    pub runtimes: HashMap<String, String>,

    /// Heading level for sections (default: 2)
    #[serde(default = "default_heading_level")]
    pub heading_level: u8,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            runtimes: default_runtimes(),
            heading_level: default_heading_level(),
        }
    }
}

impl Config {
    /// Load configuration from a TOML file
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = fs::read_to_string(path)?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }

    /// Get runtime command for a language
    pub fn get_runtime(&self, lang: &str) -> Option<&str> {
        self.runtimes.get(lang).map(|s| s.as_str())
    }

    /// Check if runtime exists for a language
    pub fn has_runtime(&self, lang: &str) -> bool {
        self.runtimes.contains_key(lang)
    }

    /// Validate that all configured runtimes are available in PATH
    pub fn validate_runtimes(&self) -> Result<()> {
        for (lang, cmd) in &self.runtimes {
            let binary = cmd.split_whitespace().next().unwrap_or(cmd);
            if which::which(binary).is_err() {
                return Err(Error::Config(format!(
                    "Runtime '{}' for language '{}' not found in PATH",
                    binary, lang
                )));
            }
        }
        Ok(())
    }
}

/// Default runtime mappings
fn default_runtimes() -> HashMap<String, String> {
    let mut runtimes = HashMap::new();
    runtimes.insert("bash".to_string(), "bash".to_string());
    runtimes.insert("sh".to_string(), "sh".to_string());
    runtimes.insert("python".to_string(), "python3".to_string());
    runtimes.insert("ruby".to_string(), "ruby".to_string());
    runtimes.insert("node".to_string(), "node".to_string());
    runtimes.insert("javascript".to_string(), "node".to_string());
    runtimes.insert("js".to_string(), "node".to_string());
    runtimes.insert("go".to_string(), "go run".to_string());
    runtimes.insert("golang".to_string(), "go run".to_string());
    runtimes.insert("php".to_string(), "php".to_string());
    runtimes.insert("perl".to_string(), "perl".to_string());
    runtimes.insert("jq".to_string(), "jq".to_string());
    runtimes.insert("mq".to_string(), "mq".to_string());
    runtimes
}

/// Default heading level
fn default_heading_level() -> u8 {
    2
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.heading_level, 2);
        assert!(config.has_runtime("bash"));
        assert!(config.has_runtime("python"));
    }

    #[test]
    fn test_get_runtime() {
        let config = Config::default();
        assert_eq!(config.get_runtime("bash"), Some("bash"));
        assert_eq!(config.get_runtime("python"), Some("python3"));
        assert_eq!(config.get_runtime("unknown"), None);
    }
}
