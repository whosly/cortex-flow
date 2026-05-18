//! # SimpleTask 实现
//!
//! 提供简化版任务定义，无需泛型参数。
//!
//! ## 模块概述
//!
//! SimpleTask 是 Task trait 的简化实现，提供了：
//!
//! - **无需泛型**: 直接使用 `serde_json::Value` 作为输入输出类型
//! - **函数式创建**: 通过闭包创建任务
//! - **便捷宏**: 提供 `simple_task!` 和 `named_task!` 宏简化创建
//!
//! ## 使用示例
//!
//! ```rust,ignore
//! use ai_collector::task::SimpleTask;
//!
//! let task = SimpleTask::new("fetch", "Fetch Data", |_input, _ctx| {
//!     Box::pin(async move {
//!         Ok(serde_json::json!({ "data": "fetched" }))
//!     })
//! });
//!
//! // 或使用宏
//! let task = simple_task!("process", "Process Data", {
//!     Ok(serde_json::json!({ "processed": true }))
//! });
//! ```

use async_trait::async_trait;
use serde_json::Value;
use std::sync::Arc;
use std::pin::Pin;
use std::future::Future;
use crate::context::ExecutionContext;
use crate::error::Result;
use crate::task::TaskExecutor;

/// 简单任务函数类型
pub type SimpleTaskFn = Arc<
    dyn Fn(Value, &ExecutionContext) -> Pin<Box<dyn Future<Output = Result<Value>> + Send>>
        + Send + Sync,
>;

/// 简单任务 - 无需定义泛型类型的任务实现
#[derive(Clone)]
pub struct SimpleTask {
    /// 任务ID
    id: String,
    /// 任务名称
    name: String,
    /// 任务描述
    description: Option<String>,
    /// 执行函数
    execute_fn: SimpleTaskFn,
}

impl SimpleTask {
    /// 创建新任务
    pub fn new<I, N, F>(id: I, name: N, execute_fn: F) -> Self
    where
        I: Into<String>,
        N: Into<String>,
        F: Fn(Value, &ExecutionContext) -> Pin<Box<dyn Future<Output = Result<Value>> + Send>>
            + Send + Sync + 'static,
    {
        Self {
            id: id.into(),
            name: name.into(),
            description: None,
            execute_fn: Arc::new(execute_fn),
        }
    }

    /// 创建带描述的任务（构造函数）
    pub fn with_description<I, N, D, F>(id: I, name: N, description: D, execute_fn: F) -> Self
    where
        I: Into<String>,
        N: Into<String>,
        D: Into<String>,
        F: Fn(Value, &ExecutionContext) -> Pin<Box<dyn Future<Output = Result<Value>> + Send>>
            + Send + Sync + 'static,
    {
        Self {
            id: id.into(),
            name: name.into(),
            description: Some(description.into()),
            execute_fn: Arc::new(execute_fn),
        }
    }

    /// 执行任务
    pub async fn execute(&self, input: Value, ctx: &ExecutionContext) -> Result<Value> {
        (self.execute_fn)(input, ctx).await
    }

    /// 获取任务ID
    pub fn id(&self) -> &str {
        &self.id
    }

    /// 获取任务名称
    pub fn name(&self) -> &str {
        &self.name
    }

    /// 获取任务描述
    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    /// 设置任务名称
    pub fn with_name<N: Into<String>>(mut self, name: N) -> Self {
        self.name = name.into();
        self
    }
}

#[async_trait]
impl TaskExecutor for SimpleTask {
    async fn execute_task(&self, input: Value, ctx: &ExecutionContext) -> Result<Value> {
        self.execute(input, ctx).await
    }

    fn task_id(&self) -> &str {
        &self.id
    }

    fn task_name(&self) -> &str {
        &self.name
    }

    fn task_description(&self) -> Option<&str> {
        self.description.as_deref()
    }
}

impl std::fmt::Debug for SimpleTask {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SimpleTask")
            .field("id", &self.id)
            .field("name", &self.name)
            .field("description", &self.description)
            .finish()
    }
}

impl PartialEq for SimpleTask {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for SimpleTask {}

impl std::hash::Hash for SimpleTask {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl From<SimpleTask> for Arc<dyn TaskExecutor> {
    fn from(task: SimpleTask) -> Self {
        Arc::new(task)
    }
}

/// 宏：简化创建同步任务
#[macro_export]
macro_rules! simple_task {
    ($id:expr, $name:expr, $body:expr) => {
        $crate::task::SimpleTask::new($id, $name, |_input, _ctx| {
            Box::pin(async move {
                $body
            })
        })
    };
}

/// 宏：简化创建带描述的任务
#[macro_export]
macro_rules! named_task {
    ($id:expr, $name:expr, $desc:expr, $body:expr) => {
        $crate::task::SimpleTask::with_description($id, $name, $desc, |_input, _ctx| {
            Box::pin(async move {
                $body
            })
        })
    };
}
