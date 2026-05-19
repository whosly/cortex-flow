//! # 编排器模块
//!
//! 框架的门面组件，协调各组件工作。

pub use builder::OrchestratorBuilder;
pub use orchestrator::Orchestrator;

mod builder;
#[allow(clippy::module_inception)]
mod orchestrator;
