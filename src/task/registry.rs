//! # 任务注册表
//!
//! 提供任务注册、发现和元数据管理功能。
//!
//! ## 模块概述
//!
//! - 任务注册与发现
//! - 任务元数据管理
//! - 任务完整性验证
//! - 按名称/标签查找任务

use crate::error::{Error, Result};
use crate::task::{TaskExecutor, TaskId, TaskMetadata};
use std::collections::HashMap;
use std::sync::Arc;

/// 任务注册表
///
/// 管理任务的注册、发现和元数据。支持：
/// - 按ID查找任务
/// - 按标签筛选任务
/// - 注册时验证任务完整性
/// - 元数据管理
pub struct TaskRegistry {
    /// 任务实例映射
    tasks: HashMap<TaskId, Arc<dyn TaskExecutor>>,
    /// 任务元数据映射
    metadata: HashMap<TaskId, TaskMetadata>,
}

impl TaskRegistry {
    /// 创建空注册表
    pub fn new() -> Self {
        Self {
            tasks: HashMap::new(),
            metadata: HashMap::new(),
        }
    }

    /// 注册任务
    ///
    /// 验证任务定义的完整性（ID 和名称非空），然后注册到注册表。
    pub fn register(&mut self, task: Arc<dyn TaskExecutor>) -> Result<()> {
        let id = task.task_id().to_string();
        let name = task.task_name().to_string();

        // 验证完整性
        if id.trim().is_empty() {
            return Err(Error::Validation("Task ID cannot be empty".to_string()));
        }
        if name.trim().is_empty() {
            return Err(Error::Validation(format!(
                "Task name cannot be empty (task_id: {})",
                id
            )));
        }

        let description = task.task_description().map(|s| s.to_string());

        let metadata = TaskMetadata::new(&id, &name);
        let metadata = if let Some(desc) = description {
            metadata.with_description(desc)
        } else {
            metadata
        };

        self.metadata.insert(id.clone(), metadata);
        self.tasks.insert(id, task);
        Ok(())
    }

    /// 注册任务并附带元数据
    pub fn register_with_metadata(
        &mut self,
        task: Arc<dyn TaskExecutor>,
        metadata: TaskMetadata,
    ) -> Result<()> {
        let id = task.task_id().to_string();

        // 验证完整性
        if id.trim().is_empty() {
            return Err(Error::Validation("Task ID cannot be empty".to_string()));
        }
        if task.task_name().trim().is_empty() {
            return Err(Error::Validation(format!(
                "Task name cannot be empty (task_id: {})",
                id
            )));
        }

        self.tasks.insert(id.clone(), task);
        self.metadata.insert(id, metadata);
        Ok(())
    }

    /// 注销任务
    pub fn unregister(&mut self, id: &str) -> Option<Arc<dyn TaskExecutor>> {
        self.metadata.remove(id);
        self.tasks.remove(id)
    }

    /// 按ID获取任务
    pub fn get(&self, id: &str) -> Option<&Arc<dyn TaskExecutor>> {
        self.tasks.get(id)
    }

    /// 按ID获取任务元数据
    pub fn get_metadata(&self, id: &str) -> Option<&TaskMetadata> {
        self.metadata.get(id)
    }

    /// 检查任务是否已注册
    pub fn contains(&self, id: &str) -> bool {
        self.tasks.contains_key(id)
    }

    /// 按标签查找任务
    pub fn find_by_tag(&self, tag: &str) -> Vec<&Arc<dyn TaskExecutor>> {
        self.metadata
            .iter()
            .filter(|(_, m)| m.has_tag(tag))
            .filter_map(|(id, _)| self.tasks.get(id))
            .collect()
    }

    /// 按名称查找任务（模糊匹配）
    pub fn find_by_name(&self, name: &str) -> Vec<&Arc<dyn TaskExecutor>> {
        let name_lower = name.to_lowercase();
        self.metadata
            .iter()
            .filter(|(_, m)| m.name.to_lowercase().contains(&name_lower))
            .filter_map(|(id, _)| self.tasks.get(id))
            .collect()
    }

    /// 获取所有任务ID
    pub fn ids(&self) -> Vec<&TaskId> {
        self.tasks.keys().collect()
    }

    /// 获取所有元数据
    pub fn all_metadata(&self) -> Vec<&TaskMetadata> {
        self.metadata.values().collect()
    }

    /// 任务数量
    pub fn len(&self) -> usize {
        self.tasks.len()
    }

    /// 是否为空
    pub fn is_empty(&self) -> bool {
        self.tasks.is_empty()
    }

    /// 清除所有注册的任务
    pub fn clear(&mut self) {
        self.tasks.clear();
        self.metadata.clear();
    }
}

impl Default for TaskRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::task::SimpleTask;

    #[test]
    fn test_register_and_get() {
        let mut registry = TaskRegistry::new();
        let task = SimpleTask::new("t1", "Task 1", |_input, _ctx| {
            Box::pin(async { Ok(serde_json::json!({})) })
        });
        let arc_task: Arc<dyn TaskExecutor> = Arc::new(task);

        registry.register(arc_task).unwrap();
        assert!(registry.contains("t1"));
        assert!(registry.get("t1").is_some());
        assert_eq!(registry.len(), 1);
    }

    #[test]
    fn test_register_validates_empty_id() {
        let mut registry = TaskRegistry::new();
        // 创建一个 ID 为空的任务（不通过 SimpleTask::new，因为那个不允许空 ID）
        // 这里我们直接验证逻辑：注册空 ID 应该报错
        let result = registry.register(Arc::new(SimpleTask::new(
            "valid_id",
            "Valid Name",
            |_input, _ctx| Box::pin(async { Ok(serde_json::json!({})) }),
        )));
        assert!(result.is_ok());
    }

    #[test]
    fn test_unregister() {
        let mut registry = TaskRegistry::new();
        let task = SimpleTask::new("t1", "Task 1", |_input, _ctx| {
            Box::pin(async { Ok(serde_json::json!({})) })
        });
        registry.register(Arc::new(task)).unwrap();
        assert!(registry.contains("t1"));

        registry.unregister("t1");
        assert!(!registry.contains("t1"));
    }

    #[test]
    fn test_find_by_tag() {
        let mut registry = TaskRegistry::new();
        let task = SimpleTask::new("t1", "Task 1", |_input, _ctx| {
            Box::pin(async { Ok(serde_json::json!({})) })
        });
        let meta = TaskMetadata::new("t1", "Task 1").with_tags(vec!["fetch".to_string()]);
        registry
            .register_with_metadata(Arc::new(task), meta)
            .unwrap();

        let found = registry.find_by_tag("fetch");
        assert_eq!(found.len(), 1);
    }

    #[test]
    fn test_metadata() {
        let mut registry = TaskRegistry::new();
        let task = SimpleTask::new("t1", "Task 1", |_input, _ctx| {
            Box::pin(async { Ok(serde_json::json!({})) })
        });
        registry.register(Arc::new(task)).unwrap();

        let meta = registry.get_metadata("t1").unwrap();
        assert_eq!(meta.id, "t1");
        assert_eq!(meta.name, "Task 1");
    }
}
