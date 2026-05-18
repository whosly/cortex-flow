//! # 编排器模块
//!
//! 框架的门面组件，协调各组件工作。

pub use orchestrator::Orchestrator;
pub use builder::OrchestratorBuilder;

mod orchestrator;
mod builder;
