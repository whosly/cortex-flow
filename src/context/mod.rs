//! # 执行上下文模块

pub use context::ContextLog;
pub use context::ContextSnapshot;
pub use context::ExecutionContext;

#[allow(clippy::module_inception)]
mod context;
mod scope;

pub use scope::ContextScope;
