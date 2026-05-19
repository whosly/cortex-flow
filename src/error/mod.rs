//! # 错误处理模块
//!
//! 定义框架统一的错误类型和处理机制。
//!
//! ## 模块概述
//!
//! 本模块提供了框架级别的错误处理能力，包括：
//!
//! - **统一错误类型**: 使用 [`Error`] 枚举定义所有可能的错误
//! - **错误码系统**: 每个错误都有对应的错误码，便于追踪
//! - **可恢复性判断**: 支持判断错误是否可恢复
//! - **上下文丰富**: 错误信息包含详细的上下文描述
//!
//! ## 错误码映射
//!
//! | 错误类型 | 错误码 | 说明 |
//! |---------|--------|------|
//! | TaskExecution | TASK_001 | 任务执行失败 |
//! | TaskNotFound | TASK_002 | 任务不存在 |
//! | DAGValidation | DAG_001 | DAG 验证失败 |
//! | DAGCycle | DAG_002 | DAG 包含循环 |
//! | LLMCall | LLM_001 | LLM 调用失败 |
//! | Config | CFG_001 | 配置错误 |
//!
//! ## 使用示例
//!
//! ```rust,ignore
//! use ai_collector::error::{Error, Result};
//!
//! fn example() -> Result<()> {
//!     return Err(Error::task_execution("fetch", "Network timeout"));
//! }
//! ```

use thiserror::Error;

/// 框架结果类型别名
pub type Result<T> = std::result::Result<T, Error>;

/// 框架错误类型
#[derive(Debug, Error)]
pub enum Error {
    /// 任务执行错误
    #[error("Task execution failed: {0}")]
    TaskExecution(String),

    /// 任务不存在
    #[error("Task not found: {0}")]
    TaskNotFound(String),

    /// DAG结构错误
    #[error("DAG validation error: {0}")]
    DAGValidation(String),

    /// DAG存在环
    #[error("DAG contains cycle: {0}")]
    DAGCycle(String),

    /// LLM调用错误
    #[error("LLM call failed: {0}")]
    LLMCall(String),

    /// 配置错误
    #[error("Configuration error: {0}")]
    Config(String),

    /// 执行上下文错误
    #[error("Execution context error: {0}")]
    Context(String),

    /// 策略错误
    #[error("Strategy error: {0}")]
    Strategy(String),

    /// 序列化错误
    #[error("Serialization error: {0}")]
    Serialization(String),

    /// 超时错误
    #[error("Operation timed out: {0}")]
    Timeout(String),

    /// 取消错误
    #[error("Operation cancelled: {0}")]
    Cancelled(String),

    /// 资源错误
    #[error("Resource error: {0}")]
    Resource(String),

    /// 验证错误
    #[error("Validation error: {0}")]
    Validation(String),

    /// 内部错误
    #[error("Internal error: {0}")]
    Internal(String),

    /// IO错误
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// JSON错误
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// TOML解析错误
    #[error("TOML parse error: {0}")]
    TomlParse(#[from] toml::de::Error),

    /// TOML序列化错误
    #[error("TOML serialize error: {0}")]
    TomlSerialize(#[from] toml::ser::Error),

    /// 其他错误
    #[error("{0}")]
    Other(String),
}

impl Error {
    /// 判断错误是否可恢复
    pub fn is_recoverable(&self) -> bool {
        matches!(
            self,
            Error::TaskExecution(_) | Error::Timeout(_) | Error::LLMCall(_) | Error::Resource(_)
        )
    }

    /// 获取错误码
    pub fn error_code(&self) -> &'static str {
        match self {
            Error::TaskExecution(_) => "TASK_001",
            Error::TaskNotFound(_) => "TASK_002",
            Error::DAGValidation(_) => "DAG_001",
            Error::DAGCycle(_) => "DAG_002",
            Error::LLMCall(_) => "LLM_001",
            Error::Config(_) => "CFG_001",
            Error::Context(_) => "CTX_001",
            Error::Strategy(_) => "STR_001",
            Error::Serialization(_) => "SER_001",
            Error::Timeout(_) => "TMO_001",
            Error::Cancelled(_) => "CNL_001",
            Error::Resource(_) => "RES_001",
            Error::Validation(_) => "VAL_001",
            Error::Internal(_) => "INT_001",
            Error::Io(_) => "IO_001",
            Error::Json(_) => "JSON_001",
            Error::TomlParse(_) => "TOML_001",
            Error::TomlSerialize(_) => "TOML_002",
            Error::Other(_) => "OTH_001",
        }
    }

    /// 创建任务执行错误
    pub fn task_execution(task_name: impl Into<String>, msg: impl Into<String>) -> Self {
        Error::TaskExecution(format!("{}: {}", task_name.into(), msg.into()))
    }

    /// 创建DAG验证错误
    pub fn dag_validation(msg: impl Into<String>) -> Self {
        Error::DAGValidation(msg.into())
    }

    /// 创建DAG环错误
    pub fn dag_cycle(msg: impl Into<String>) -> Self {
        Error::DAGCycle(msg.into())
    }
}
