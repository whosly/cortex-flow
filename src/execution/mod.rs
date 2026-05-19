//! # 执行模块
//!
//! 定义执行计划、引擎、状态管理和报告生成。

pub use engine::ExecutionEngine;
pub use plan::{ExecutionNode, ExecutionPlan};
pub use report::{ExecutionReport, ReportGenerator, TaskReport};
pub use state::{ExecutionPhase, ExecutionState, ExecutionStatus, TaskState};

mod engine;
mod plan;
mod report;
mod state;
