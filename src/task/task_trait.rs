//! # Task Trait 定义
//!
//! 定义任务执行的核心接口和数据结构。
//!
//! ## 模块概述
//!
//! 本模块定义了任务编排系统的核心接口：
//!
//! - [`Task`] - 泛型任务接口，支持自定义输入输出类型
//! - [`TaskExecutor`] - 任务执行器接口，用于 DAG 节点执行
//! - [`TaskValidator`] - 任务验证器接口
//! - [`PostProcessor`] - 后处理器接口
//!
//! ## 设计原则
//!
//! - **泛型支持**: 使用 Rust 的泛型系统支持任意输入输出类型
//! - **异步优先**: 所有执行操作都是异步的
//! - **可验证**: 支持可选的输入验证
//! - **可后处理**: 支持可选的输出后处理
//!
//! ## 使用示例
//!
//! ```rust,ignore
//! use async_trait::async_trait;
//! use serde::{Deserialize, Serialize};
//!
//! struct MyTask;
//!
//! #[async_trait]
//! impl Task for MyTask {
//!     type Input = String;
//!     type Output = String;
//!
//!     fn id(&self) -> &str { "my_task" }
//!     fn name(&self) -> &str { "My Task" }
//!
//!     async fn execute(&self, input: String, ctx: &ExecutionContext) -> Result<String> {
//!         Ok(format!("Hello, {}", input))
//!     }
//! }
//! ```

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use crate::error::Result;
use crate::context::ExecutionContext;
use crate::task::{TaskId, TaskOutput};

/// 任务执行结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskExecutionResult {
    /// 任务ID
    pub task_id: TaskId,
    /// 任务名称
    pub name: String,
    /// 执行状态
    pub status: TaskStatus,
    /// 输出数据
    pub output: Option<TaskOutput>,
    /// 错误信息
    pub error: Option<String>,
    /// 执行时间（毫秒）
    pub duration_ms: u64,
    /// 开始时间戳
    pub start_time: Option<i64>,
    /// 结束时间戳
    pub end_time: Option<i64>,
}

impl TaskExecutionResult {
    /// 创建成功结果
    pub fn success(task_id: TaskId, name: String, output: TaskOutput, duration_ms: u64) -> Self {
        let now = chrono::Utc::now().timestamp_millis();
        Self {
            task_id,
            name,
            status: TaskStatus::Completed,
            output: Some(output),
            error: None,
            duration_ms,
            start_time: Some(now - duration_ms as i64),
            end_time: Some(now),
        }
    }

    /// 创建失败结果
    pub fn failure(task_id: TaskId, name: String, error: String, duration_ms: u64) -> Self {
        let now = chrono::Utc::now().timestamp_millis();
        Self {
            task_id,
            name,
            status: TaskStatus::Failed,
            output: None,
            error: Some(error),
            duration_ms,
            start_time: Some(now - duration_ms as i64),
            end_time: Some(now),
        }
    }

    /// 检查是否成功
    pub fn is_success(&self) -> bool {
        self.status == TaskStatus::Completed
    }
}

/// 任务执行状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Skipped,
    Cancelled,
}

impl Default for TaskStatus {
    fn default() -> Self {
        Self::Pending
    }
}

impl std::fmt::Display for TaskStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TaskStatus::Pending => write!(f, "pending"),
            TaskStatus::Running => write!(f, "running"),
            TaskStatus::Completed => write!(f, "completed"),
            TaskStatus::Failed => write!(f, "failed"),
            TaskStatus::Skipped => write!(f, "skipped"),
            TaskStatus::Cancelled => write!(f, "cancelled"),
        }
    }
}

/// Task trait - 所有任务必须实现的接口
#[async_trait]
pub trait Task: Send + Sync {
    /// 泛型输入类型
    type Input: Send + Sync + serde::Serialize + serde::de::DeserializeOwned + Default;
    /// 泛型输出类型
    type Output: Send + Sync + serde::Serialize + serde::de::DeserializeOwned;

    /// 获取任务ID
    fn id(&self) -> &str;
    
    /// 获取任务名称
    fn name(&self) -> &str;
    
    /// 获取任务描述
    fn description(&self) -> Option<&str> {
        None
    }

    /// 执行任务
    async fn execute(
        &self,
        input: Self::Input,
        ctx: &ExecutionContext,
    ) -> std::result::Result<Self::Output, Box<dyn std::error::Error + Send + Sync>>;

    /// 获取验证器（可选）
    fn validator(&self) -> Option<Box<dyn TaskValidator<Input = Self::Input>>> {
        None
    }

    /// 获取后处理器（可选）
    fn post_processor(&self) -> Option<Box<dyn PostProcessor<Output = Self::Output>>> {
        None
    }
}

/// 任务验证器trait
pub trait TaskValidator: Send + Sync {
    type Input: Send + Sync + serde::Serialize + serde::de::DeserializeOwned;
    
    fn validate(&self, input: &Self::Input) -> Result<()>;
}

/// 后处理器trait
pub trait PostProcessor: Send + Sync {
    type Output: Send + Sync + serde::Serialize + serde::de::DeserializeOwned;
    
    fn process(&self, output: Self::Output) -> Result<Self::Output>;
}

/// 任务执行器包装 trait - 用于DAG节点
#[async_trait]
pub trait TaskExecutor: Send + Sync {
    /// 执行任务
    async fn execute_task(
        &self,
        input: serde_json::Value,
        ctx: &ExecutionContext,
    ) -> Result<serde_json::Value>;

    /// 获取任务ID
    fn task_id(&self) -> &str;
    
    /// 获取任务名称
    fn task_name(&self) -> &str;
    
    /// 获取任务描述
    fn task_description(&self) -> Option<&str> {
        None
    }
}

/// 空任务执行器 - 用于反序列化时的占位符
///
/// 当从 JSON 反序列化 `DAGNode` 或 `ExecutionNode` 时，
/// `task` 字段无法被反序列化（因为 trait 对象无法被反序列化），
/// 因此使用这个空实现作为占位符。
///
/// **警告**: 这个执行器不能被实际执行，只能用于结构恢复。
pub struct EmptyTaskExecutor;

#[async_trait]
impl TaskExecutor for EmptyTaskExecutor {
    async fn execute_task(
        &self,
        _input: serde_json::Value,
        _ctx: &ExecutionContext,
    ) -> Result<serde_json::Value> {
        Err(crate::error::Error::Internal(
            "EmptyTaskExecutor cannot be executed. This is a placeholder.".to_string(),
        ))
    }

    fn task_id(&self) -> &str {
        "<placeholder>"
    }
    
    fn task_name(&self) -> &str {
        "<Placeholder Task>"
    }
    
    fn task_description(&self) -> Option<&str> {
        Some("Placeholder task executor for deserialization")
    }
}
