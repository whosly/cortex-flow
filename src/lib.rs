//! # CortexFlow
//!
//! 基于Rust实现的AI智能体调度框架，核心目标是将复杂目标拆解为一系列有序、可控、可评估的子任务。

pub mod error;
pub mod observability;
pub mod config;
pub mod task;
pub mod context;
pub mod execution;
pub mod strategy;
pub mod dag;
pub mod llm;
pub mod orchestrator;

// 重新导出常用类型
pub use error::{Error, Result};
pub use task::{Task, TaskResult, TaskInput, TaskOutput, TaskExecutor};
pub use context::ExecutionContext;
pub use execution::ExecutionEngine;
pub use strategy::{OrchestrationStrategy, StrategyFactory, StrategyType, ExecutionResult, TaskExecutionResult};
pub use dag::{DAGBuilder, DAGNode, DAGEdge};
pub use orchestrator::Orchestrator;
