//! # DAG 编排策略实现
//!
//! 实现了基于 DAG 的任务编排策略。
//!
//! ## 策略特点
//!
//! - **拓扑排序**: 使用 Kahn 算法确定任务执行顺序
//! - **层级并行**: 同一层级的任务可以并行执行
//! - **循环检测**: 自动检测并拒绝包含循环的 DAG
//! - **配置灵活**: 支持自定义最大并发数和超时设置
//!
//! ## 执行流程
//!
//! 1. **验证阶段**: 检查 DAG 结构有效性
//! 2. **计划阶段**: 计算拓扑排序和执行层级
//! 3. **执行阶段**: 按层级顺序执行任务
//! 4. **汇总阶段**: 聚合所有节点执行结果

use crate::context::ExecutionContext;
use crate::dag::DAG;
use crate::error::Result;
use crate::execution::ExecutionPlan;
use crate::strategy::{ExecutionResult, OrchestrationStrategy, StrategyConfig};
use async_trait::async_trait;
use std::sync::Arc;
use std::time::Instant;

/// DAG 编排策略
///
/// 基本的 DAG 编排策略，按照拓扑排序顺序执行任务，
/// 同一层级的任务可以并行执行。
///
/// # 示例
///
/// ```rust,ignore
/// use ai_collector::strategy::{DAGStrategy, StrategyConfig};
///
/// let strategy = DAGStrategy::new(
///     StrategyConfig::new("my-dag")
///         .with_parallelism(4)
///         .with_timeout(60000)
/// );
/// ```
pub struct DAGStrategy {
    /// 策略配置
    config: StrategyConfig,
    /// 最大并发数
    max_parallelism: usize,
}

impl DAGStrategy {
    /// 创建新的 DAG 策略
    ///
    /// # 参数
    ///
    /// - `config`: 策略配置
    pub fn new(config: StrategyConfig) -> Self {
        let max_parallelism = config.max_parallelism.unwrap_or(10);
        Self {
            config,
            max_parallelism,
        }
    }

    /// 设置最大并发数
    ///
    /// 覆盖配置中的并发数设置。
    pub fn with_max_parallelism(mut self, parallelism: usize) -> Self {
        self.max_parallelism = parallelism;
        self
    }
}

impl Default for DAGStrategy {
    fn default() -> Self {
        Self::new(StrategyConfig::default())
    }
}

#[async_trait]
impl OrchestrationStrategy for DAGStrategy {
    /// 获取策略名称
    fn name(&self) -> &str {
        &self.config.name
    }

    /// 获取策略配置
    fn config(&self) -> &StrategyConfig {
        &self.config
    }

    /// 执行 DAG
    ///
    /// 按照 DAG 的拓扑排序执行所有节点。
    /// 同一层级的节点会并行执行。
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

        // 创建 DAG 执行器
        let mut executor = dag.executor();

        // 执行所有节点
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
    ///
    /// 检查 DAG 是否有效：
    /// - 非空
    /// - 无循环
    /// - 有根节点
    /// - 所有边的节点都存在
    fn validate(&self, dag: &DAG) -> Result<()> {
        dag.validate()
    }

    /// 创建执行计划
    ///
    /// 计算 DAG 的拓扑排序和执行层级。
    fn plan(&self, dag: &DAG) -> Result<ExecutionPlan> {
        dag.validate()?;

        // 计算拓扑排序
        let execution_order = dag.topological_sort()?;

        // 计算执行层级（同一层可并行）
        let layers = dag.compute_layers()?;

        // 估算执行时间
        // 注意：这是粗略估算，假设每层平均执行时间为 1000ms
        let estimated_duration_ms = if !layers.is_empty() {
            Some(layers.len() as u64 * 1000)
        } else {
            None
        };

        Ok(ExecutionPlan::new(
            Vec::new(), // Plan 本身不包含完整节点信息
            dag.roots().iter().map(|n| n.id.clone()).collect(),
            execution_order,
        )
        .with_dag(Arc::new(dag.clone()))
        .with_layers(layers)
        .with_estimated_duration(estimated_duration_ms.unwrap_or(0)))
    }
}
