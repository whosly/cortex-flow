//! # 错误恢复管理器
//!
//! 整合重试策略、错误处理器和补偿操作的统一管理器。

use async_trait::async_trait;
use crate::error::{Error, Result};
use crate::context::ExecutionContext;
use super::retry::RetryPolicy;
use super::handler::{ErrorHandler, ErrorAction};
use super::compensation::{Compensation, CompensationResult, NoOpCompensation};

/// 错误恢复管理器
///
/// 提供统一的错误恢复机制，包括重试、错误处理和补偿。
pub struct ErrorRecovery {
    /// 重试策略
    retry_policy: RetryPolicy,
    /// 错误处理器
    error_handler: ErrorHandler,
    /// 补偿操作
    compensation: Box<dyn Compensation>,
}

impl ErrorRecovery {
    /// 创建新的错误恢复管理器
    pub fn new(retry_policy: RetryPolicy) -> Self {
        Self {
            retry_policy,
            error_handler: ErrorHandler::default(),
            compensation: Box::new(NoOpCompensation),
        }
    }

    /// 创建带自定义错误处理器的恢复管理器
    pub fn with_handler(retry_policy: RetryPolicy, error_handler: ErrorHandler) -> Self {
        Self {
            retry_policy,
            error_handler,
            compensation: Box::new(NoOpCompensation),
        }
    }

    /// 设置补偿操作
    pub fn with_compensation<C: Compensation + 'static>(mut self, compensation: C) -> Self {
        self.compensation = Box::new(compensation);
        self
    }

    /// 获取重试策略引用
    pub fn retry_policy(&self) -> &RetryPolicy {
        &self.retry_policy
    }

    /// 获取错误处理器可变引用
    pub fn error_handler_mut(&mut self) -> &mut ErrorHandler {
        &mut self.error_handler
    }

    /// 带重试执行异步操作
    ///
    /// 根据重试策略自动重试失败的操作。
    pub async fn execute_with_retry<F, Fut, T>(&mut self, operation: F) -> Result<T>
    where
        F: Fn() -> Fut,
        Fut: std::future::Future<Output = Result<T>>,
    {
        let mut attempt = 0u32;

        loop {
            match operation().await {
                Ok(value) => {
                    self.error_handler.reset_consecutive_errors();
                    return Ok(value);
                }
                Err(error) => {
                    let handle_result = self.error_handler.handle(&error);

                    match handle_result.action {
                        ErrorAction::Retry => {
                            if self.retry_policy.should_retry(attempt, Some(error.error_code())) {
                                let wait = self.retry_policy.wait_duration(attempt + 1);
                                attempt += 1;
                                tokio::time::sleep(wait).await;
                                continue;
                            }
                            return Err(error);
                        }
                        ErrorAction::Skip => {
                            // 跳过：无法返回有效值，返回原始错误
                            return Err(error);
                        }
                        ErrorAction::Abort => {
                            return Err(error);
                        }
                        ErrorAction::Compensate => {
                            let ctx = ExecutionContext::new();
                            let _ = self.compensation.compensate(&ctx).await;
                            return Err(error);
                        }
                        ErrorAction::Ignore => {
                            // 忽略错误：同样无法返回有效值
                            return Err(error);
                        }
                    }
                }
            }
        }
    }

    /// 带重试和上下文执行异步操作
    pub async fn execute_with_retry_ctx<F, Fut, T>(
        &mut self,
        ctx: &ExecutionContext,
        operation: F,
    ) -> Result<T>
    where
        F: Fn(&ExecutionContext) -> Fut,
        Fut: std::future::Future<Output = Result<T>>,
    {
        let mut attempt = 0u32;

        loop {
            match operation(ctx).await {
                Ok(value) => {
                    self.error_handler.reset_consecutive_errors();
                    return Ok(value);
                }
                Err(error) => {
                    let handle_result = self.error_handler.handle(&error);

                    match handle_result.action {
                        ErrorAction::Retry => {
                            if self.retry_policy.should_retry(attempt, Some(error.error_code())) {
                                let wait = self.retry_policy.wait_duration(attempt + 1);
                                attempt += 1;
                                tokio::time::sleep(wait).await;
                                continue;
                            }
                            return Err(error);
                        }
                        ErrorAction::Compensate => {
                            let _ = self.compensation.compensate(ctx).await;
                            return Err(error);
                        }
                        ErrorAction::Abort | ErrorAction::Skip | ErrorAction::Ignore => {
                            return Err(error);
                        }
                    }
                }
            }
        }
    }

    /// 执行补偿操作
    pub async fn compensate(&self, ctx: &ExecutionContext) -> CompensationResult {
        self.compensation.compensate(ctx).await
    }

    /// 处理错误（不重试，仅分类和决策）
    pub fn handle_error(&mut self, error: &Error) -> ErrorAction {
        let result = self.error_handler.handle(error);
        result.action
    }
}

impl Default for ErrorRecovery {
    fn default() -> Self {
        Self::new(RetryPolicy::default())
    }
}

/// 可恢复执行 trait
///
/// 为任务执行器提供错误恢复能力。
#[async_trait]
pub trait RecoverableExecutor: Send + Sync {
    /// 带恢复机制的执行
    async fn execute_with_recovery(
        &self,
        input: serde_json::Value,
        ctx: &ExecutionContext,
        recovery: &mut ErrorRecovery,
    ) -> Result<serde_json::Value>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::sync::Arc;

    #[tokio::test]
    async fn test_retry_success_on_second_attempt() {
        let policy = RetryPolicy::fixed(3, 10);
        let mut recovery = ErrorRecovery::new(policy);
        let counter = Arc::new(AtomicU32::new(0));

        let result: Result<String> = recovery
            .execute_with_retry(|| {
                let counter = counter.clone();
                async move {
                    let count = counter.fetch_add(1, Ordering::SeqCst);
                    if count == 0 {
                        Err(Error::TaskExecution("first attempt fails".to_string()))
                    } else {
                        Ok("success".to_string())
                    }
                }
            })
            .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "success");
        assert_eq!(counter.load(Ordering::SeqCst), 2);
    }

    #[tokio::test]
    async fn test_retry_exhausted() {
        let policy = RetryPolicy::fixed(2, 10);
        let mut recovery = ErrorRecovery::new(policy);

        let result: Result<String> = recovery
            .execute_with_retry(|| async {
                Err(Error::TaskExecution("always fails".to_string()))
            })
            .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_no_retry_for_unrecoverable() {
        let mut handler = ErrorHandler::new(ErrorAction::Retry);
        handler.register_action("CFG_001", ErrorAction::Abort);
        let policy = RetryPolicy::fixed(3, 10);
        let mut recovery = ErrorRecovery::with_handler(policy, handler);

        let counter = Arc::new(AtomicU32::new(0));
        let result: Result<String> = recovery
            .execute_with_retry(|| {
                let counter = counter.clone();
                async move {
                    counter.fetch_add(1, Ordering::SeqCst);
                    Err(Error::Config("bad config".to_string()))
                }
            })
            .await;

        assert!(result.is_err());
        // 应该只执行一次（配置错误不重试）
        assert_eq!(counter.load(Ordering::SeqCst), 1);
    }
}
