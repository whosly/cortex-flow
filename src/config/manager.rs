//! # 配置管理器
//!
//! 管理框架配置，支持文件加载、环境变量覆盖、验证和热更新。

use super::FrameworkConfig;
use crate::error::{Error, Result};

pub struct ConfigManager {
    framework: FrameworkConfig,
    /// 配置文件路径（用于热更新）
    config_path: Option<String>,
}

impl ConfigManager {
    /// 创建默认配置管理器
    pub fn new() -> Self {
        Self {
            framework: FrameworkConfig::default(),
            config_path: None,
        }
    }

    /// 从文件加载配置
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

    /// 从环境变量创建配置（覆盖默认值）
    ///
    /// 支持的环境变量：
    /// - `CORTEX_FLOW_LOG_LEVEL`: 日志级别
    /// - `CORTEX_FLOW_WORKERS`: 最大工作线程数
    /// - `CORTEX_FLOW_TIMEOUT`: 默认超时时间（毫秒）
    /// - `CORTEX_FLOW_MAX_RETRIES`: 最大重试次数
    /// - `CORTEX_FLOW_STRATEGY`: 默认策略 (dag/sequential)
    pub fn from_env() -> Self {
        let mut config = Self::new();

        if let Ok(level) = std::env::var("CORTEX_FLOW_LOG_LEVEL") {
            config.framework.logging.level = level;
        }
        if let Ok(workers) = std::env::var("CORTEX_FLOW_WORKERS") {
            if let Ok(workers) = workers.parse() {
                config.framework.execution.max_workers = workers;
            }
        }
        if let Ok(timeout) = std::env::var("CORTEX_FLOW_TIMEOUT") {
            if let Ok(timeout) = timeout.parse() {
                config.framework.execution.default_timeout = Some(timeout);
            }
        }
        if let Ok(retries) = std::env::var("CORTEX_FLOW_MAX_RETRIES") {
            if let Ok(retries) = retries.parse() {
                config.framework.execution.max_retries = retries;
            }
        }
        if let Ok(strategy) = std::env::var("CORTEX_FLOW_STRATEGY") {
            match strategy.to_lowercase().as_str() {
                "sequential" => {
                    config.framework.strategy.default_type =
                        crate::strategy::StrategyType::Sequential
                }
                _ => config.framework.strategy.default_type = crate::strategy::StrategyType::Dag,
            }
        }

        config
    }

    /// 自动加载配置文件
    ///
    /// 依次尝试加载以下文件：
    /// 1. cortex-flow.toml
    /// 2. cortex-flow.json
    /// 3. ai_scheduler.toml
    /// 4. ai_scheduler.json
    /// 5. config.toml
    /// 6. config.json
    ///
    /// 未找到任何文件则使用默认配置。
    pub fn load() -> Result<Self> {
        for filename in [
            "cortex-flow.toml",
            "cortex-flow.json",
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

    /// 加载配置并应用环境变量覆盖
    ///
    /// 先加载配置文件，再用环境变量覆盖。
    pub fn load_with_env_override() -> Result<Self> {
        let mut config = Self::load()?;

        // 环境变量覆盖文件配置
        if let Ok(level) = std::env::var("CORTEX_FLOW_LOG_LEVEL") {
            config.framework.logging.level = level;
        }
        if let Ok(workers) = std::env::var("CORTEX_FLOW_WORKERS") {
            if let Ok(workers) = workers.parse() {
                config.framework.execution.max_workers = workers;
            }
        }
        if let Ok(timeout) = std::env::var("CORTEX_FLOW_TIMEOUT") {
            if let Ok(timeout) = timeout.parse() {
                config.framework.execution.default_timeout = Some(timeout);
            }
        }
        if let Ok(retries) = std::env::var("CORTEX_FLOW_MAX_RETRIES") {
            if let Ok(retries) = retries.parse() {
                config.framework.execution.max_retries = retries;
            }
        }

        config.validate()?;
        Ok(config)
    }

    /// 验证配置
    pub fn validate(&self) -> Result<()> {
        self.framework.validate()
    }

    /// 热更新：重新从配置文件加载
    ///
    /// 如果之前是从文件加载的配置，重新读取文件内容并更新。
    /// 如果没有关联文件路径，返回错误。
    pub fn reload(&mut self) -> Result<()> {
        let path = self
            .config_path
            .clone()
            .ok_or_else(|| Error::Config("No config file path set, cannot reload".to_string()))?;

        let new_config = Self::from_file(&path)?;
        self.framework = new_config.framework;
        Ok(())
    }

    /// 获取框架配置
    pub fn framework(&self) -> &FrameworkConfig {
        &self.framework
    }

    /// 获取框架配置的可变引用
    pub fn framework_mut(&mut self) -> &mut FrameworkConfig {
        &mut self.framework
    }

    /// 获取配置文件路径
    pub fn config_path(&self) -> Option<&str> {
        self.config_path.as_deref()
    }
}

impl Default for ConfigManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config_is_valid() {
        let config = ConfigManager::new();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_from_env() {
        // 不设置环境变量也能正常创建
        let config = ConfigManager::from_env();
        assert!(config.validate().is_ok());
    }
}
