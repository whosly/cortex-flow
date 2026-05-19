//! # 任务元数据定义

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// 任务元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskMetadata {
    /// 任务ID
    pub id: String,
    /// 任务名称
    pub name: String,
    /// 任务描述
    pub description: Option<String>,
    /// 创建时间
    pub created_at: DateTime<Utc>,
    /// 标签
    pub tags: Vec<String>,
    /// 自定义属性
    pub properties: std::collections::HashMap<String, serde_json::Value>,
}

impl TaskMetadata {
    pub fn new(id: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            description: None,
            created_at: Utc::now(),
            tags: Vec::new(),
            properties: std::collections::HashMap::new(),
        }
    }

    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = tags;
        self
    }

    pub fn add_tag(&mut self, tag: impl Into<String>) {
        self.tags.push(tag.into());
    }

    pub fn with_property(
        mut self,
        key: impl Into<String>,
        value: impl Into<serde_json::Value>,
    ) -> Self {
        self.properties.insert(key.into(), value.into());
        self
    }

    pub fn get_property(&self, key: &str) -> Option<&serde_json::Value> {
        self.properties.get(key)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        self.tags.iter().any(|t| t == tag)
    }
}

impl Default for TaskMetadata {
    fn default() -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name: String::new(),
            description: None,
            created_at: Utc::now(),
            tags: Vec::new(),
            properties: std::collections::HashMap::new(),
        }
    }
}
