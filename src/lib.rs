//! # CortexFlow
//!
//! 基于Rust实现的AI智能体调度框架，核心目标是将复杂目标拆解为一系列有序、可控、可评估的子任务。

pub mod config;
pub mod context;
pub mod dag;
pub mod error;
pub mod error_recovery;
pub mod execution;
pub mod llm;
pub mod observability;
pub mod orchestrator;
pub mod strategy;
pub mod task;

// 重新导出常用类型
pub use context::ExecutionContext;
pub use dag::{DAGBuilder, DAGEdge, DAGNode, ExecutionProgress, ProgressCallback};
pub use error::{Error, Result};
pub use error_recovery::{ErrorHandler, ErrorRecovery, RetryPolicy};
pub use execution::ExecutionEngine;
pub use llm::LLMClientRegistry;
pub use orchestrator::Orchestrator;
pub use strategy::{
    ExecutionResult, OrchestrationStrategy, StrategyFactory, StrategyRegistry, StrategyType,
    TaskExecutionResult,
};
pub use task::{Task, TaskExecutor, TaskInput, TaskOutput, TaskRegistry, TaskResult};
