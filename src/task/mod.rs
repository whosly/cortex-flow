//! # 任务模块
//!
//! 定义任务相关的核心类型和trait。

pub use task_trait::{Task, TaskExecutor, TaskExecutionResult, TaskStatus, TaskValidator, PostProcessor, EmptyTaskExecutor};
pub use simple_task::SimpleTask;
pub use task_result::TaskResult;
pub use task_input::TaskInput;
pub use task_output::TaskOutput;

mod task_trait;
mod simple_task;
mod task_result;
mod task_input;
mod task_output;
mod task_metadata;

pub use task_metadata::TaskMetadata;

/// 任务ID类型别名
pub type TaskId = String;

/// 简化的Task trait（用于快速定义任务）
pub trait SimpleTaskTrait: Send + Sync {
    fn id(&self) -> &TaskId;
    fn name(&self) -> &str;
    fn execute(&self) -> std::pin::Pin<Box<dyn std::future::Future<Output = crate::error::Result<serde_json::Value>> + Send>>;
}

/// 任务注册表 - 用于运行时注册和查找任务
pub struct TaskRegistry {
    tasks: std::collections::HashMap<TaskId, Arc<dyn TaskExecutor>>,
}

impl TaskRegistry {
    pub fn new() -> Self {
        Self {
            tasks: std::collections::HashMap::new(),
        }
    }

    pub fn register(&mut self, task: Arc<dyn TaskExecutor>) {
        self.tasks.insert(task.task_id().to_string(), task);
    }

    pub fn get(&self, id: &str) -> Option<&Arc<dyn TaskExecutor>> {
        self.tasks.get(id)
    }

    pub fn contains(&self, id: &str) -> bool {
        self.tasks.contains_key(id)
    }

    pub fn len(&self) -> usize {
        self.tasks.len()
    }

    pub fn is_empty(&self) -> bool {
        self.tasks.is_empty()
    }

    pub fn ids(&self) -> Vec<&TaskId> {
        self.tasks.keys().collect()
    }
}

impl Default for TaskRegistry {
    fn default() -> Self {
        Self::new()
    }
}

use std::sync::Arc;
