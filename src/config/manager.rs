//! # 配置管理器

use super::FrameworkConfig;
use crate::error::{Error, Result};

pub struct ConfigManager {
    framework: FrameworkConfig,
    /// 配置文件路径（用于后续热更新）
    #[allow(dead_code)]
    config_path: Option<String>,
}

impl ConfigManager {
    pub fn new() -> Self {
        Self {
            framework: FrameworkConfig::default(),
            config_path: None,
        }
    }

    pub fn from_file(path: impl AsRef<std::path::Path>) -> Result<Self> {
        let path = path.as_ref();
        let content = std::fs::read_to_string(path)
            .map_err(|e| Error::Config(format!("Failed to read config file: {}", e)))?;

        let framework = if path.extension().map(|e| e == "toml").unwrap_or(false) {
            toml::from_str(&content)?
        } else {
            serde_json::from_str(&content)?
        };

        Ok(Self {
            framework,
            config_path: Some(path.to_string_lossy().to_string()),
        })
    }

    pub fn from_env() -> Self {
        let mut config = Self::new();
        if let Ok(level) = std::env::var("AI_SCHEDULER_LOG_LEVEL") {
            config.framework.logging.level = level;
        }
        if let Ok(workers) = std::env::var("AI_SCHEDULER_WORKERS") {
            if let Ok(workers) = workers.parse() {
                config.framework.execution.max_workers = workers;
            }
        }
        config
    }

    pub fn load() -> Result<Self> {
        for filename in [
            "ai_scheduler.toml",
            "ai_scheduler.json",
            "config.toml",
            "config.json",
        ] {
            let path = std::path::Path::new(filename);
            if path.exists() {
                return Self::from_file(path);
            }
        }
        Ok(Self::new())
    }

    pub fn framework(&self) -> &FrameworkConfig {
        &self.framework
    }

    pub fn framework_mut(&mut self) -> &mut FrameworkConfig {
        &mut self.framework
    }

    pub fn validate(&self) -> Result<()> {
        self.framework.validate()
    }
}

impl Default for ConfigManager {
    fn default() -> Self {
        Self::new()
    }
}
