//! # 执行引擎
//!
//! 负责管理和执行任务编排的执行引擎。
//!
//! ## 模块概述
//!
//! 执行引擎是任务编排的核心组件，负责：
//!
//! - **生命周期管理**: 管理执行从开始到结束的整个生命周期
//! - **状态跟踪**: 跟踪每个任务的执行状态
//! - **策略执行**: 委托给编排策略执行实际逻辑
//! - **错误处理**: 处理执行过程中的错误并触发回滚
//!
//! ## 执行阶段
//!
//! 执行引擎经历以下阶段：
//!
//! 1. **Validating** - 验证阶段：验证 DAG 和执行计划
//! 2. **Executing** - 执行阶段：执行任务
//! 3. **RollingBack** - 回滚阶段：处理失败回滚
//! 4. **Completed** - 完成阶段：执行完成

use std::sync::Arc;
use std::time::Instant;
use crate::error::{Error, Result};
use crate::context::ExecutionContext;
use crate::strategy::{OrchestrationStrategy, ExecutionResult};
use crate::execution::{ExecutionPlan, ExecutionState, ExecutionPhase};
use crate::dag::DAG;

/// 执行引擎
///
/// 管理任务编排的执行生命周期。
///
/// # 设计特点
///
/// - **单次执行**: 每次执行创建新的状态
/// - **状态隔离**: 使用 ExecutionState 跟踪执行状态
/// - **策略委托**: 将实际执行委托给 OrchestrationStrategy
///
/// # 示例
///
/// ```rust,ignore
/// use ai_collector::execution::ExecutionEngine;
/// use ai_collector::strategy::DAGStrategy;
///
/// let strategy = Arc::new(DAGStrategy::default());
/// let mut engine = ExecutionEngine::new(strategy);
/// let result = engine.execute_dag(&dag, &ctx).await?;
/// ```
pub struct ExecutionEngine {
    /// 编排策略
    strategy: Arc<dyn OrchestrationStrategy>,
    /// 执行状态
    state: ExecutionState,
    /// DAG引用（可选）
    dag: Option<Arc<DAG>>,
}

impl ExecutionEngine {
    /// 创建新的执行引擎
    ///
    /// # 参数
    ///
    /// - `strategy`: 编排策略
    pub fn new(strategy: Arc<dyn OrchestrationStrategy>) -> Self {
        Self {
            strategy,
            state: ExecutionState::new(),
            dag: None,
        }
    }

    /// 创建带 DAG 的执行引擎
    ///
    /// # 参数
    ///
    /// - `strategy`: 编排策略
    /// - `dag`: 要执行的 DAG
    pub fn with_dag(strategy: Arc<dyn OrchestrationStrategy>, dag: DAG) -> Self {
        Self {
            strategy,
            state: ExecutionState::new(),
            dag: Some(Arc::new(dag)),
        }
    }

    /// 执行计划
    ///
    /// 根据执行计划执行任务。
    ///
    /// # 参数
    ///
    /// - `plan`: 执行计划
    /// - `ctx`: 执行上下文
    ///
    /// # 返回
    ///
    /// - `Result<ExecutionResult>` - 执行结果
    ///
    /// # 执行流程
    ///
    /// 1. 更新状态为 Validating
    /// 2. 验证执行计划
    /// 3. 初始化任务状态
    /// 4. 更新状态为 Executing
    /// 5. 调用策略执行
    /// 6. 更新状态为 Completed
    /// 7. 如果失败，触发回滚
    pub async fn execute(
        &mut self,
        plan: ExecutionPlan,
        ctx: &ExecutionContext,
    ) -> Result<ExecutionResult> {
        let start_time = Instant::now();
        self.state.start();
        
        // 验证阶段
        self.state.set_phase(ExecutionPhase::Validating);
        self.validate_plan(&plan).await?;

        // 初始化任务状态
        for node in &plan.nodes {
            self.state.add_task(node.id.clone(), node.name.clone());
        }

        // 从计划中提取 DAG（如果存在）
        let dag_opt = plan.dag().cloned();
        
        // 执行阶段
        self.state.set_phase(ExecutionPhase::Executing);
        let result = if let Some(dag) = dag_opt.as_ref() {
            self.strategy.execute(dag, ctx).await
        } else {
            Ok(ExecutionResult::failure(
                "No DAG in execution plan".to_string(),
                vec![],
                start_time.elapsed().as_millis() as u64,
            ))
        };
        
        // 完成
        self.state.finish();

        match result {
            Ok(exec_result) => Ok(exec_result),
            Err(e) => {
                self.state.set_phase(ExecutionPhase::RollingBack);
                // 转换 Option<&Arc<DAG>> 为 Option<&DAG>
                let dag_ref = dag_opt.as_ref().map(|v| &**v);
                let _ = self.strategy.rollback(dag_ref, ctx).await;
                Ok(ExecutionResult::failure(
                    e.to_string(),
                    vec![],
                    start_time.elapsed().as_millis() as u64,
                ))
            }
        }
    }

    /// 从 DAG 执行
    ///
    /// 直接执行 DAG，自动创建执行计划。
    ///
    /// # 参数
    ///
    /// - `dag`: 要执行的 DAG
    /// - `ctx`: 执行上下文
    pub async fn execute_dag(&mut self, dag: &DAG, ctx: &ExecutionContext) -> Result<ExecutionResult> {
        let start_time = Instant::now();
        self.state.start();
        
        // 验证阶段
        self.state.set_phase(ExecutionPhase::Validating);
        dag.validate()?;

        // 初始化任务状态
        for node in dag.nodes().values() {
            self.state.add_task(node.id.clone(), node.name.clone());
        }

        // 执行阶段
        self.state.set_phase(ExecutionPhase::Executing);
        let result = self.strategy.execute(dag, ctx).await;
        
        // 完成
        self.state.finish();

        match result {
            Ok(exec_result) => Ok(exec_result),
            Err(e) => {
                self.state.set_phase(ExecutionPhase::RollingBack);
                let _ = self.strategy.rollback(Some(dag), ctx).await;
                Ok(ExecutionResult::failure(
                    e.to_string(),
                    vec![],
                    start_time.elapsed().as_millis() as u64,
                ))
            }
        }
    }

    /// 验证执行计划
    async fn validate_plan(&self, plan: &ExecutionPlan) -> Result<()> {
        if plan.is_empty() {
            return Err(Error::Validation("Execution plan is empty".to_string()));
        }
        // 如果计划中有 DAG，验证 DAG
        if let Some(dag) = plan.dag() {
            self.strategy.validate(dag)?;
        }
        Ok(())
    }

    /// 获取执行状态
    pub fn state(&self) -> &ExecutionState {
        &self.state
    }

    /// 检查是否完成
    pub fn is_completed(&self) -> bool {
        self.state.is_completed()
    }

    /// 检查是否失败
    pub fn is_failed(&self) -> bool {
        self.state.is_failed()
    }

    /// 获取已完成的节点数量
    pub fn completed_count(&self) -> usize {
        self.state.completed_count()
    }

    /// 获取失败节点数量
    pub fn failed_count(&self) -> usize {
        self.state.failed_count()
    }

    /// 获取总节点数量
    pub fn total_count(&self) -> usize {
        self.state.total_count()
    }

    /// 获取当前执行阶段
    pub fn phase(&self) -> ExecutionPhase {
        self.state.phase
    }
}

impl Default for ExecutionEngine {
    fn default() -> Self {
        Self {
            strategy: Arc::new(crate::strategy::DAGStrategy::default()),
            state: ExecutionState::new(),
            dag: None,
        }
    }
}
