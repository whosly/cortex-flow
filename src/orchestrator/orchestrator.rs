//! # Orchestrator 核心实现
//!
//! Orchestrator 是整个编排框架的门面（Facade），提供统一的任务编排入口。
//!
//! ## 功能特性
//!
//! - **DAG执行**: 支持通过 DAG 构建器创建和执行复杂的工作流
//! - **任务管理**: 支持单一任务执行和 DAG 批量执行
//! - **上下文管理**: 提供执行上下文用于数据传递和状态共享
//! - **可观测性**: 集成追踪和指标收集
//! - **LLM集成**: 可选的 LLM 客户端支持
//!
//! ## 使用示例
//!
//! ```rust,ignore
//! use ai_collector::Orchestrator;
//!
//! let orchestrator = Orchestrator::builder()
//!     .with_max_parallelism(4)
//!     .build()
//!     .await?;
//!
//! let result = orchestrator.execute_dag(|dag| {
//!     dag.add_task("step1", || async { Ok("result1") });
//!     dag.add_task("step2", || async { Ok("result2") });
//!     dag.add_dependency("step1", "step2");
//! }).await?;
//! ```

use super::OrchestratorBuilder;
use crate::config::{ConfigManager, FrameworkConfig};
use crate::context::ExecutionContext;
use crate::dag::{DAGBuilder, ExecutionProgress, DAG};
use crate::error::Result;
use crate::llm::{LLMClientRegistry, LLMClientTrait};
use crate::observability::{MetricsCollector, TracerImpl};
use crate::strategy::ExecutionResult;
use crate::strategy::StrategyRegistry;
use crate::task::TaskExecutor;
use std::sync::Arc;

/// Orchestrator 是编排框架的核心门面类
///
/// 提供统一的任务编排入口，管理执行引擎、配置、追踪和指标收集。
///
/// # 设计原则
///
/// - **门面模式**: 隐藏内部复杂性，提供简洁的API
/// - **不可变配置**: 配置一旦构建就不可修改
/// - **异步优先**: 所有执行操作都是异步的
///
/// # 示例
///
/// ```rust,ignore
/// let orchestrator = Orchestrator::builder()
///     .with_max_parallelism(4)
///     .build()
///     .await?;
/// ```
pub struct Orchestrator {
    /// 配置管理器
    pub(crate) config: ConfigManager,
    /// 追踪器
    pub(crate) tracer: TracerImpl,
    /// 指标收集器
    pub(crate) metrics: MetricsCollector,
    /// LLM客户端（可选，向后兼容）
    pub(crate) llm_client: Option<Arc<dyn LLMClientTrait>>,
    /// LLM 客户端注册表（多模型管理）
    pub(crate) llm_registry: LLMClientRegistry,
    /// 策略注册表
    pub(crate) strategy_registry: StrategyRegistry,
    /// 最大并发数
    #[allow(dead_code)]
    pub(crate) max_parallelism: usize,
}

impl Orchestrator {
    /// 创建新的 Orchestrator 构建器
    ///
    /// 这是创建 Orchestrator 实例的推荐方式。
    pub fn builder() -> OrchestratorBuilder {
        OrchestratorBuilder::new()
    }

    /// 执行 DAG（通过闭包构建）
    ///
    /// 这是一个便捷方法，允许通过闭包直接构建 DAG 并执行。
    /// 闭包接收一个 `DAGBuilder` 引用，可以添加任务和依赖关系。
    pub async fn execute_dag<F>(&self, dag_builder: F) -> Result<ExecutionResult>
    where
        F: FnOnce(&mut DAGBuilder),
    {
        let mut builder = DAGBuilder::new();
        dag_builder(&mut builder);
        let dag = builder.build()?;
        self.execute_dag_instance(dag).await
    }

    /// 执行 DAG（通过闭包构建，带进度回调）
    ///
    /// 与 `execute_dag` 类似，但额外支持进度回调。
    /// 进度回调会在每个任务完成后被调用。
    pub async fn execute_dag_with_progress<F, P>(
        &self,
        dag_builder: F,
        progress_callback: P,
    ) -> Result<ExecutionResult>
    where
        F: FnOnce(&mut DAGBuilder),
        P: Fn(ExecutionProgress) + Send + Sync + 'static,
    {
        let mut builder = DAGBuilder::new();
        dag_builder(&mut builder);
        let dag = builder.build()?;
        self.execute_dag_instance_with_progress(dag, progress_callback)
            .await
    }

    /// 执行已构建的 DAG 实例（带进度回调）
    pub async fn execute_dag_instance_with_progress<P>(
        &self,
        dag: DAG,
        progress_callback: P,
    ) -> Result<ExecutionResult>
    where
        P: Fn(ExecutionProgress) + Send + Sync + 'static,
    {
        dag.validate()?;
        let mut ctx = self.create_context_with_llm();
        ctx.set("__dag__", &dag).ok();

        let mut executor = dag.executor().with_progress_callback(progress_callback);
        let results = executor.execute_all(&ctx).await?;

        let success_count = results.iter().filter(|r| r.success).count();
        let failure_count = results.len() - success_count;
        let total_duration = results.iter().map(|r| r.duration_ms).sum::<u64>();

        Ok(ExecutionResult {
            success: failure_count == 0,
            node_results: results,
            total_duration_ms: total_duration,
            start_time: chrono::Utc::now().timestamp_millis() - total_duration as i64,
            end_time: chrono::Utc::now().timestamp_millis(),
            error: if failure_count > 0 {
                Some(format!("{} tasks failed", failure_count))
            } else {
                None
            },
        })
    }

    /// 执行已构建的 DAG 实例
    pub async fn execute_dag_instance(&self, dag: DAG) -> Result<ExecutionResult> {
        dag.validate()?;
        let mut ctx = self.create_context_with_llm();
        ctx.set("__dag__", &dag).ok();

        let mut executor = dag.executor();
        let results = executor.execute_all(&ctx).await?;

        let success_count = results.iter().filter(|r| r.success).count();
        let failure_count = results.len() - success_count;
        let total_duration = results.iter().map(|r| r.duration_ms).sum::<u64>();

        Ok(ExecutionResult {
            success: failure_count == 0,
            node_results: results,
            total_duration_ms: total_duration,
            start_time: chrono::Utc::now().timestamp_millis() - total_duration as i64,
            end_time: chrono::Utc::now().timestamp_millis(),
            error: if failure_count > 0 {
                Some(format!("{} tasks failed", failure_count))
            } else {
                None
            },
        })
    }

    /// 执行单个任务
    ///
    /// 便捷方法，用于执行不需要 DAG 的简单任务。
    pub async fn execute_task<S>(&self, task: S) -> Result<serde_json::Value>
    where
        S: Into<Arc<dyn TaskExecutor>>,
    {
        let task = task.into();
        let ctx = ExecutionContext::new();
        task.execute_task(serde_json::Value::Null, &ctx).await
    }

    /// 创建新的执行上下文
    pub fn create_context(&self) -> ExecutionContext {
        ExecutionContext::new()
    }

    /// 创建带 LLM 注册表的执行上下文
    ///
    /// 内部方法：在 DAG 执行前调用，将 LLM 注册表注入到上下文中，
    /// 使得任务闭包中可以通过 `ctx.get_llm()` 获取 LLM 客户端。
    fn create_context_with_llm(&self) -> ExecutionContext {
        let mut ctx = ExecutionContext::new();
        if !self.llm_registry.is_empty() {
            ctx.attach_llm_registry(Arc::new(self.llm_registry.clone()));
        }
        ctx
    }

    /// 获取框架配置
    pub fn config(&self) -> &FrameworkConfig {
        self.config.framework()
    }

    /// 获取追踪器
    pub fn tracer(&self) -> &TracerImpl {
        &self.tracer
    }

    /// 获取指标收集器
    pub fn metrics(&self) -> &MetricsCollector {
        &self.metrics
    }

    /// 获取 LLM 客户端（如果配置了）
    pub fn llm_client(&self) -> Option<&Arc<dyn LLMClientTrait>> {
        self.llm_client.as_ref()
    }

    /// 获取 LLM 客户端注册表
    pub fn llm_registry(&self) -> &LLMClientRegistry {
        &self.llm_registry
    }

    /// 通过名称获取 LLM 客户端
    pub fn get_llm(&self, name: &str) -> Option<Arc<dyn LLMClientTrait>> {
        self.llm_registry.get(name)
    }

    /// 获取默认 LLM 客户端（优先从注册表获取，否则回退到单客户端）
    pub fn default_llm(&self) -> Option<Arc<dyn LLMClientTrait>> {
        self.llm_registry
            .default_client()
            .or_else(|| self.llm_client.clone())
    }

    /// 注册 LLM 客户端
    pub fn register_llm(
        &self,
        name: impl Into<String>,
        client: impl Into<Arc<dyn LLMClientTrait>>,
    ) {
        self.llm_registry.register(name, client);
    }

    /// 获取 Token 使用快照
    pub fn token_snapshot(&self) -> crate::llm::token_tracker::TokenTrackerSnapshot {
        self.llm_registry.token_snapshot()
    }

    /// 获取策略注册表
    pub fn strategy_registry(&self) -> &StrategyRegistry {
        &self.strategy_registry
    }

    /// 获取策略注册表（可变引用）
    pub fn strategy_registry_mut(&mut self) -> &mut StrategyRegistry {
        &mut self.strategy_registry
    }

    /// 设置 LLM 客户端
    pub fn with_llm_client(mut self, client: Arc<dyn LLMClientTrait>) -> Self {
        self.llm_client = Some(client);
        self
    }
}
