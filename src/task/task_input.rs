//! # 任务输入定义
//!
//! 定义任务执行时的输入数据。

use serde::{Deserialize, Serialize};

/// 任务输入
///
/// 表示任务执行时的输入数据。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskInput {
    /// 输入数据
    pub data: serde_json::Value,
    /// 来源任务ID
    pub source_task_id: Option<String>,
    /// 元数据
    pub metadata: std::collections::HashMap<String, serde_json::Value>,
}

impl TaskInput {
    /// 创建新的任务输入
    pub fn new(data: impl Into<serde_json::Value>) -> Self {
        Self {
            data: data.into(),
            source_task_id: None,
            metadata: std::collections::HashMap::new(),
        }
    }

    /// 从指定任务创建输入
    pub fn from_task(task_id: impl Into<String>, data: impl Into<serde_json::Value>) -> Self {
        Self {
            data: data.into(),
            source_task_id: Some(task_id.into()),
            metadata: std::collections::HashMap::new(),
        }
    }

    /// 添加元数据
    pub fn with_metadata(
        mut self,
        key: impl Into<String>,
        value: impl Into<serde_json::Value>,
    ) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    /// 获取元数据
    pub fn get_metadata(&self, key: &str) -> Option<&serde_json::Value> {
        self.metadata.get(key)
    }
}

impl Default for TaskInput {
    fn default() -> Self {
        Self {
            data: serde_json::Value::Null,
            source_task_id: None,
            metadata: std::collections::HashMap::new(),
        }
    }
}
