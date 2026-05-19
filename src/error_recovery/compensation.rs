//! # 补偿操作
//!
//! 定义任务执行失败时的补偿机制。

#![allow(dead_code)]

use crate::context::ExecutionContext;
use async_trait::async_trait;

/// 补偿操作结果
#[derive(Debug, Clone)]
pub enum CompensationResult {
    /// 补偿成功
    Success,
    /// 补偿失败
    Failed(String),
    /// 跳过补偿
    Skipped,
}

/// 补偿操作 trait
///
/// 定义任务失败时的回滚/补偿逻辑。
#[async_trait]
pub trait Compensation: Send + Sync {
    /// 执行补偿操作
    async fn compensate(&self, ctx: &ExecutionContext) -> CompensationResult;

    /// 获取补偿操作名称
    fn name(&self) -> &str;
}

/// 空补偿操作（不做任何事）
pub struct NoOpCompensation;

#[async_trait]
impl Compensation for NoOpCompensation {
    async fn compensate(&self, _ctx: &ExecutionContext) -> CompensationResult {
        CompensationResult::Skipped
    }

    fn name(&self) -> &str {
        "noop"
    }
}

/// 函数式补偿操作
pub struct FnCompensation {
    name: String,
    f: Box<dyn Fn(&ExecutionContext) -> CompensationResult + Send + Sync>,
}

impl FnCompensation {
    /// 创建函数式补偿操作
    pub fn new(
        name: impl Into<String>,
        f: impl Fn(&ExecutionContext) -> CompensationResult + Send + Sync + 'static,
    ) -> Self {
        Self {
            name: name.into(),
            f: Box::new(f),
        }
    }
}

#[async_trait]
impl Compensation for FnCompensation {
    async fn compensate(&self, ctx: &ExecutionContext) -> CompensationResult {
        (self.f)(ctx)
    }

    fn name(&self) -> &str {
        &self.name
    }
}

/// 异步补偿函数类型
type AsyncCompensationFn = Box<
    dyn Fn(
            &ExecutionContext,
        )
            -> std::pin::Pin<Box<dyn std::future::Future<Output = CompensationResult> + Send>>
        + Send
        + Sync,
>;

/// 异步函数式补偿操作
pub struct AsyncFnCompensation {
    name: String,
    f: AsyncCompensationFn,
}

impl AsyncFnCompensation {
    /// 创建异步函数式补偿操作
    pub fn new<F, Fut>(name: impl Into<String>, f: F) -> Self
    where
        F: Fn(&ExecutionContext) -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = CompensationResult> + Send + 'static,
    {
        Self {
            name: name.into(),
            f: Box::new(move |ctx| Box::pin(f(ctx))),
        }
    }
}

#[async_trait]
impl Compensation for AsyncFnCompensation {
    async fn compensate(&self, ctx: &ExecutionContext) -> CompensationResult {
        (self.f)(ctx).await
    }

    fn name(&self) -> &str {
        &self.name
    }
}

/// 补偿链
///
/// 按顺序执行多个补偿操作。任一失败则停止后续补偿。
pub struct CompensationChain {
    compensations: Vec<Box<dyn Compensation>>,
}

impl CompensationChain {
    /// 创建空补偿链
    pub fn new() -> Self {
        Self {
            compensations: Vec::new(),
        }
    }

    /// 添加补偿操作
    #[allow(clippy::should_implement_trait)]
    pub fn add<C: Compensation + 'static>(mut self, compensation: C) -> Self {
        self.compensations.push(Box::new(compensation));
        self
    }

    /// 执行所有补偿操作
    pub async fn execute_all(&self, ctx: &ExecutionContext) -> Vec<CompensationResult> {
        let mut results = Vec::with_capacity(self.compensations.len());
        for comp in &self.compensations {
            let result = comp.compensate(ctx).await;
            let should_continue = matches!(
                result,
                CompensationResult::Success | CompensationResult::Skipped
            );
            results.push(result);
            if !should_continue {
                break;
            }
        }
        results
    }

    /// 获取补偿操作数量
    pub fn len(&self) -> usize {
        self.compensations.len()
    }

    /// 是否为空
    pub fn is_empty(&self) -> bool {
        self.compensations.is_empty()
    }
}

impl Default for CompensationChain {
    fn default() -> Self {
        Self::new()
    }
}
