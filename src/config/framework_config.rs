//! # 框架配置

use serde::{Deserialize, Serialize};
use crate::error::{Error, Result};
use crate::strategy::StrategyType;
use crate::llm::LLMConfig;

/// 框架配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrameworkConfig {
    /// 执行配置
    pub execution: ExecutionConfig,
    /// 日志配置
    pub logging: LoggingConfig,
    /// LLM配置
    pub llm: Option<LLMConfig>,
    /// 策略配置
    pub strategy: StrategyConfig,
    /// 可观测性配置
    pub observability: ObservabilityConfig,
}

impl FrameworkConfig {
    /// 验证配置
    pub fn validate(&self) -> Result<()> {
        if self.execution.max_workers == 0 {
            return Err(Error::Config("max_workers must be greater than 0".to_string()));
        }
        if self.execution.max_workers > 1000 {
            return Err(Error::Config("max_workers exceeds maximum limit (1000)".to_string()));
        }
        if let Some(ref timeout) = self.execution.default_timeout {
            if *timeout == 0 {
                return Err(Error::Config("default_timeout must be greater than 0".to_string()));
            }
        }
        Ok(())
    }

    /// 从文件加载配置
    pub fn from_file(path: &str) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        Self::from_str(&content)
    }

    /// 从字符串解析配置
    pub fn from_str(s: &str) -> Result<Self> {
        // 尝试TOML格式
        if let Ok(config) = toml::from_str::<Self>(s) {
            return Ok(config);
        }
        // 尝试JSON格式
        if let Ok(config) = serde_json::from_str::<Self>(s) {
            return Ok(config);
        }
        Err(Error::Config("Failed to parse configuration".to_string()))
    }

    /// 保存配置到文件
    pub fn save(&self, path: &str) -> Result<()> {
        let content = toml::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }
}

impl Default for FrameworkConfig {
    fn default() -> Self {
        Self {
            execution: ExecutionConfig::default(),
            logging: LoggingConfig::default(),
            llm: None,
            strategy: StrategyConfig::default(),
            observability: ObservabilityConfig::default(),
        }
    }
}

/// 执行配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionConfig {
    /// 最大工作线程数
    pub max_workers: usize,
    /// 默认超时时间（毫秒）
    pub default_timeout: Option<u64>,
    /// 最大重试次数
    pub max_retries: u32,
    /// 重试间隔（毫秒）
    pub retry_interval_ms: u64,
}

impl Default for ExecutionConfig {
    fn default() -> Self {
        Self {
            max_workers: num_cpus::get(),
            default_timeout: Some(60000),
            max_retries: 3,
            retry_interval_ms: 1000,
        }
    }
}

impl ExecutionConfig {
    pub fn new(max_workers: usize) -> Self {
        Self {
            max_workers,
            default_timeout: Some(60000),
            max_retries: 3,
            retry_interval_ms: 1000,
        }
    }

    pub fn with_timeout(mut self, timeout_ms: u64) -> Self {
        self.default_timeout = Some(timeout_ms);
        self
    }

    pub fn with_retries(mut self, max_retries: u32, interval_ms: u64) -> Self {
        self.max_retries = max_retries;
        self.retry_interval_ms = interval_ms;
        self
    }
}

/// 日志配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    /// 日志级别
    pub level: String,
    /// 日志格式
    pub format: String,
    /// 日志目标
    pub target: Option<String>,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self { 
            level: "info".to_string(), 
            format: "json".to_string(), 
            target: None 
        }
    }
}

impl LoggingConfig {
    pub fn new(level: impl Into<String>) -> Self {
        Self {
            level: level.into(),
            format: "json".to_string(),
            target: None,
        }
    }

    pub fn with_format(mut self, format: impl Into<String>) -> Self {
        self.format = format.into();
        self
    }

    pub fn with_target(mut self, target: impl Into<String>) -> Self {
        self.target = Some(target.into());
        self
    }
}

/// 策略配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyConfig {
    /// 默认策略类型
    pub default_type: StrategyType,
    /// DAG配置
    pub dag: Option<DAGConfig>,
}

impl Default for StrategyConfig {
    fn default() -> Self {
        Self { 
            default_type: StrategyType::Dag, 
            dag: None 
        }
    }
}

impl StrategyConfig {
    pub fn new(default_type: StrategyType) -> Self {
        Self {
            default_type,
            dag: None,
        }
    }

    pub fn with_dag_config(mut self, dag: DAGConfig) -> Self {
        self.dag = Some(dag);
        self
    }
}

/// DAG配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DAGConfig {
    /// 最大节点数
    pub max_nodes: Option<usize>,
    /// 是否启用并行执行
    pub parallel: bool,
    /// 最大并行度
    pub max_parallelism: Option<usize>,
}

impl Default for DAGConfig {
    fn default() -> Self {
        Self { 
            max_nodes: Some(1000), 
            parallel: true, 
            max_parallelism: Some(10) 
        }
    }
}

impl DAGConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_max_nodes(mut self, max: usize) -> Self {
        self.max_nodes = Some(max);
        self
    }

    pub fn with_parallelism(mut self, max: usize) -> Self {
        self.max_parallelism = Some(max);
        self
    }
}

/// 可观测性配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObservabilityConfig {
    /// 是否启用追踪
    pub tracing: bool,
    /// 是否启用指标
    pub metrics: bool,
    /// 追踪采样率
    pub tracing_sample_rate: f32,
}

impl Default for ObservabilityConfig {
    fn default() -> Self {
        Self { 
            tracing: true, 
            metrics: true, 
            tracing_sample_rate: 1.0 
        }
    }
}

impl ObservabilityConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_tracing(mut self, enabled: bool, sample_rate: f32) -> Self {
        self.tracing = enabled;
        self.tracing_sample_rate = sample_rate;
        self
    }

    pub fn with_metrics(mut self, enabled: bool) -> Self {
        self.metrics = enabled;
        self
    }
}
