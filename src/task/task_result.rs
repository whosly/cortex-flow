//! # 任务结果定义

use serde::{Deserialize, Serialize};

/// 任务结果 - 表示任务执行的输出
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskResult {
    /// 是否成功
    pub success: bool,
    /// 输出数据
    pub data: Option<serde_json::Value>,
    /// 错误信息
    pub error: Option<String>,
    /// 元数据
    pub metadata: Option<TaskResultMetadata>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskResultMetadata {
    /// 执行时间（毫秒）
    pub duration_ms: u64,
    /// 内存使用（字节）
    pub memory_used: Option<u64>,
    /// CPU使用率
    pub cpu_usage: Option<f32>,
    /// 自定义指标
    pub custom: std::collections::HashMap<String, serde_json::Value>,
}

impl TaskResult {
    pub fn success(data: serde_json::Value) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
            metadata: None,
        }
    }

    pub fn failure(error: impl Into<String>) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(error.into()),
            metadata: None,
        }
    }

    pub fn with_metadata(mut self, metadata: TaskResultMetadata) -> Self {
        self.metadata = Some(metadata);
        self
    }

    pub fn is_success(&self) -> bool {
        self.success
    }

    pub fn error_message(&self) -> Option<&str> {
        self.error.as_deref()
    }
}

impl Default for TaskResult {
    fn default() -> Self {
        Self {
            success: true,
            data: Some(serde_json::Value::Null),
            error: None,
            metadata: None,
        }
    }
}
