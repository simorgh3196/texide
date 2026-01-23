//! Linter configuration.

use std::collections::HashMap;
use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::LinterError;

/// Configuration for the linter.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinterConfig {
    /// Rule configurations.
    #[serde(default)]
    pub rules: HashMap<String, RuleConfig>,

    /// Plugin configurations.
    #[serde(default)]
    pub plugins: Vec<String>,

    /// File patterns to include.
    #[serde(default)]
    pub include: Vec<String>,

    /// File patterns to exclude.
    #[serde(default)]
    pub exclude: Vec<String>,

    /// Whether to enable caching.
    #[serde(default = "default_cache")]
    pub cache: bool,

    /// Cache directory.
    #[serde(default = "default_cache_dir")]
    pub cache_dir: String,
}

fn default_cache() -> bool {
    true
}

fn default_cache_dir() -> String {
    ".texide-cache".to_string()
}

/// Configuration for a single rule.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RuleConfig {
    /// Rule is enabled with no options.
    Enabled(bool),
    /// Rule is enabled with severity.
    Severity(String),
    /// Rule is enabled with options.
    Options(serde_json::Value),
}

impl RuleConfig {
    /// Returns whether the rule is enabled.
    pub fn is_enabled(&self) -> bool {
        match self {
            RuleConfig::Enabled(enabled) => *enabled,
            RuleConfig::Severity(s) => s != "off",
            RuleConfig::Options(_) => true,
        }
    }

    /// Gets the rule options as JSON value.
    pub fn options(&self) -> serde_json::Value {
        match self {
            RuleConfig::Enabled(_) => serde_json::Value::Null,
            RuleConfig::Severity(_) => serde_json::Value::Null,
            RuleConfig::Options(v) => v.clone(),
        }
    }
}

impl LinterConfig {
    /// Creates a new empty configuration.
    pub fn new() -> Self {
        Self {
            rules: HashMap::new(),
            plugins: Vec::new(),
            include: Vec::new(),
            exclude: Vec::new(),
            cache: true,
            cache_dir: ".texide-cache".to_string(),
        }
    }

    /// Loads configuration from a file.
    ///
    /// Supports `.texide.json`, `.texiderc`, `texide.config.json`.
    pub fn from_file(path: impl AsRef<Path>) -> Result<Self, LinterError> {
        let path = path.as_ref();
        let content = fs::read_to_string(path)
            .map_err(|e| LinterError::config(format!("Failed to read config: {}", e)))?;

        Self::from_json(&content)
    }

    /// Parses configuration from JSON string.
    pub fn from_json(json: &str) -> Result<Self, LinterError> {
        serde_json::from_str(json)
            .map_err(|e| LinterError::config(format!("Invalid config: {}", e)))
    }

    /// Returns enabled rules.
    pub fn enabled_rules(&self) -> Vec<(&str, &RuleConfig)> {
        self.rules
            .iter()
            .filter(|(_, config)| config.is_enabled())
            .map(|(name, config)| (name.as_str(), config))
            .collect()
    }

    /// Computes a hash of the configuration for cache invalidation.
    pub fn hash(&self) -> String {
        let json = serde_json::to_string(self).unwrap_or_default();
        blake3::hash(json.as_bytes()).to_hex().to_string()
    }
}

impl Default for LinterConfig {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_new() {
        let config = LinterConfig::new();
        assert!(config.rules.is_empty());
        assert!(config.cache);
    }

    #[test]
    fn test_config_from_json() {
        let json = r#"{
            "rules": {
                "no-todo": true,
                "max-lines": { "max": 100 }
            }
        }"#;

        let config = LinterConfig::from_json(json).unwrap();
        assert_eq!(config.rules.len(), 2);
    }

    #[test]
    fn test_rule_config_enabled() {
        let enabled = RuleConfig::Enabled(true);
        let disabled = RuleConfig::Enabled(false);
        let off = RuleConfig::Severity("off".to_string());
        let error = RuleConfig::Severity("error".to_string());

        assert!(enabled.is_enabled());
        assert!(!disabled.is_enabled());
        assert!(!off.is_enabled());
        assert!(error.is_enabled());
    }

    #[test]
    fn test_enabled_rules() {
        let json = r#"{
            "rules": {
                "enabled-rule": true,
                "disabled-rule": false,
                "options-rule": { "option": "value" }
            }
        }"#;

        let config = LinterConfig::from_json(json).unwrap();
        let enabled = config.enabled_rules();

        assert_eq!(enabled.len(), 2);
    }
}
