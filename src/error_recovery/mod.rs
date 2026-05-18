//! # 错误恢复模块
//!
//! 提供任务执行过程中的错误处理和恢复机制。
//!
//! ## 模块概述
//!
//! - [`RetryPolicy`] - 重试策略配置
//! - [`ErrorHandler`] - 错误处理器
//! - [`Compensation`] - 补偿操作 trait
//! - [`ErrorRecovery`] - 错误恢复管理器
//!
//! ## 使用示例
//!
//! ```rust,ignore
//! use cortex_flow::error_recovery::{RetryPolicy, ErrorRecovery};
//!
//! let policy = RetryPolicy::exponential(3, 1000, 2.0);
//! let recovery = ErrorRecovery::new(policy);
//!
//! let result = recovery.execute_with_retry(|| async {
//!     // 可能失败的操作
//!     Ok("success")
//! }).await;
//! ```

pub use retry::RetryPolicy;
pub use handler::{ErrorHandler, ErrorAction};
pub use compensation::{Compensation, CompensationResult};
pub use recovery::ErrorRecovery;

mod retry;
mod handler;
mod compensation;
mod recovery;
