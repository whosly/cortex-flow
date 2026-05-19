//! # 任务模块
//!
//! 定义任务相关的核心类型和trait。

pub use registry::TaskRegistry;
pub use simple_task::SimpleTask;
pub use task_input::TaskInput;
pub use task_output::TaskOutput;
pub use task_result::TaskResult;
pub use task_trait::{
    EmptyTaskExecutor, PostProcessor, Task, TaskExecutionResult, TaskExecutor, TaskStatus,
    TaskValidator,
};

mod registry;
mod simple_task;
mod task_input;
mod task_metadata;
mod task_output;
mod task_result;
mod task_trait;

pub use task_metadata::TaskMetadata;

/// 任务ID类型别名
pub type TaskId = String;
