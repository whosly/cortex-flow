//! # 执行模块
//!
//! 定义执行计划、引擎、状态管理和报告生成。

pub use engine::ExecutionEngine;
pub use plan::{ExecutionPlan, ExecutionNode};
pub use state::{ExecutionState, ExecutionStatus, ExecutionPhase, TaskState};
pub use report::{ExecutionReport, TaskReport, ReportGenerator};

mod engine;
mod plan;
mod state;
mod report;
