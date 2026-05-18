//! # 执行模块
//!
//! 定义执行计划、引擎和状态管理。

pub use engine::ExecutionEngine;
pub use plan::{ExecutionPlan, ExecutionNode};
pub use state::{ExecutionState, ExecutionStatus, ExecutionPhase, TaskState};

mod engine;
mod plan;
mod state;
