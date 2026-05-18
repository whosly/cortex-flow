//! # 顺序编排策略实现
//!
//! 按照给定顺序逐个执行任务，不进行并行。
//!
//! ## 策略特点
//!
//! - **顺序执行**: 按拓扑排序逐个执行任务
//! - **简单可靠**: 无并发问题，适合调试和简单场景
//! - **错误即停**: 任一任务失败则停止后续执行

use std::sync::Arc;
use std::time::Instant;
use async_trait::async_trait;
use crate::error::Result;
use crate::context::ExecutionContext;
use crate::dag::{DAG, DAGExecutor};
use crate::strategy::{OrchestrationStrategy, StrategyConfig, ExecutionResult};
use crate::execution::ExecutionPlan;

/// 顺序编排策略
///
/// 按照 DAG 的拓扑排序逐个执行任务，不进行任何并行。
/// 适用于调试、简单工作流或需要严格顺序保证的场景。
///
/// # 示例
///
/// ```rust,ignore
/// use cortex_flow::strategy::SequentialStrategy;
///
/// let strategy = SequentialStrategy::new(StrategyConfig::new("sequential"));
/// ```
pub struct SequentialStrategy {
    /// 策略配置
    config: StrategyConfig,
}

impl SequentialStrategy {
    /// 创建新的顺序策略
    pub fn new(config: StrategyConfig) -> Self {
        Self { config }
    }
}

impl Default for SequentialStrategy {
    fn default() -> Self {
        Self::new(StrategyConfig::new("sequential").with_parallelism(1))
    }
}

#[async_trait]
impl OrchestrationStrategy for SequentialStrategy {
    fn name(&self) -> &str {
        &self.config.name
    }

    fn config(&self) -> &StrategyConfig {
        &self.config
    }

    /// 顺序执行 DAG 中的所有任务
    ///
    /// 使用最大并发数为 1 的执行器，确保所有任务按顺序执行。
    async fn execute(&self, dag: &DAG, ctx: &ExecutionContext) -> Result<ExecutionResult> {
        let start = Instant::now();

        // 验证 DAG 有效性
        if let Err(e) = self.validate(dag) {
            return Ok(ExecutionResult::failure(
                e.to_string(),
                vec![],
                start.elapsed().as_millis() as u64,
            ));
        }

        // 创建并发数为 1 的执行器
        let mut executor = DAGExecutor::with_parallelism(dag.clone(), 1);

        // 顺序执行所有节点
        let node_results = match executor.execute_all(ctx).await {
            Ok(results) => results,
            Err(e) => {
                return Ok(ExecutionResult::failure(
                    e.to_string(),
                    vec![],
                    start.elapsed().as_millis() as u64,
                ));
            }
        };

        // 检查是否有失败
        let success = node_results.iter().all(|r| r.success);
        let total_duration = start.elapsed().as_millis() as u64;

        if success {
            Ok(ExecutionResult::success(node_results, total_duration))
        } else {
            let failed: Vec<_> = node_results
                .iter()
                .filter(|r| !r.success)
                .map(|r| r.error.clone().unwrap_or_default())
                .collect();
            Ok(ExecutionResult::failure(
                format!("{} tasks failed: {:?}", failed.len(), failed),
                node_results,
                total_duration,
            ))
        }
    }

    /// 验证 DAG 结构
    fn validate(&self, dag: &DAG) -> Result<()> {
        dag.validate()
    }

    /// 创建执行计划
    fn plan(&self, dag: &DAG) -> Result<ExecutionPlan> {
        dag.validate()?;

        let execution_order = dag.topological_sort()?;
        let root_nodes = dag.roots().iter().map(|n| n.id.clone()).collect();

        // 顺序策略中每个层级只有一个节点
        let layers: Vec<Vec<String>> = execution_order
            .iter()
            .map(|id| vec![id.clone()])
            .collect();

        let estimated_duration_ms = execution_order.len() as u64 * 1000;

        Ok(ExecutionPlan::new(
            Vec::new(),
            root_nodes,
            execution_order,
        )
        .with_dag(Arc::new(dag.clone()))
        .with_layers(layers)
        .with_estimated_duration(estimated_duration_ms))
    }
}
