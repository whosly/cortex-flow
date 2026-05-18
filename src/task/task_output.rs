//! # 任务输出定义
//!
//! 定义任务执行后的输出数据。

use serde::{Deserialize, Serialize};

/// 任务输出
///
/// 表示任务执行后的输出数据。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskOutput {
    /// 输出数据
    pub data: serde_json::Value,
    /// 输出到哪些下游任务
    pub target_task_ids: Vec<String>,
    /// 元数据
    pub metadata: std::collections::HashMap<String, serde_json::Value>,
}

impl TaskOutput {
    /// 创建新的任务输出
    pub fn new(data: impl Into<serde_json::Value>) -> Self {
        Self {
            data: data.into(),
            target_task_ids: Vec::new(),
            metadata: std::collections::HashMap::new(),
        }
    }

    /// 设置目标任务
    pub fn to_tasks(mut self, task_ids: Vec<String>) -> Self {
        self.target_task_ids = task_ids;
        self
    }

    /// 添加元数据
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<serde_json::Value>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    /// 获取元数据
    pub fn get_metadata(&self, key: &str) -> Option<&serde_json::Value> {
        self.metadata.get(key)
    }

    /// 添加下游任务
    pub fn add_target(&mut self, task_id: impl Into<String>) {
        self.target_task_ids.push(task_id.into());
    }
}

impl Default for TaskOutput {
    fn default() -> Self {
        Self {
            data: serde_json::Value::Null,
            target_task_ids: Vec::new(),
            metadata: std::collections::HashMap::new(),
        }
    }
}
