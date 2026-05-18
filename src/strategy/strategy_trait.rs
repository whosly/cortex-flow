//! # 编排策略 Trait 定义
//!
//! 定义编排引擎的核心接口和数据结构。
//!
//! ## 模块概述
//!
//! 本模块定义了编排策略的核心 Trait 接口，包括：
//!
//! - [`OrchestrationStrategy`] - 编排策略的核心接口
//! - [`StrategyConfig`] - 策略配置
//! - [`ExecutionResult`] - 执行结果
//! - [`TaskExecutionResult`] - 任务执行结果

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use crate::error::Result;
use crate::context::ExecutionContext;
use crate::dag::{DAG, NodeExecutionResult};
use crate::task::TaskOutput;
use crate::execution::ExecutionPlan;

/// 策略配置
///
/// 配置编排策略的行为参数。
///
/// # 字段说明
///
/// - `name`: 策略名称
/// - `max_parallelism`: 最大并发数（None 表示无限制）
/// - `timeout_ms`: 任务超时时间（毫秒）
/// - `enable_retry`: 是否启用重试
/// - `max_retries`: 最大重试次数
/// - `retry_interval_ms`: 重试间隔（毫秒）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyConfig {
    /// 策略名称
    pub name: String,
    /// 最大并发数（None 表示无限制）
    pub max_parallelism: Option<usize>,
    /// 超时时间（毫秒）
    pub timeout_ms: Option<u64>,
    /// 是否启用重试
    pub enable_retry: bool,
    /// 最大重试次数
    pub max_retries: u32,
    /// 重试间隔（毫秒）
    pub retry_interval_ms: u64,
}

impl Default for StrategyConfig {
    fn default() -> Self {
        Self {
            name: "default".to_string(),
            max_parallelism: Some(10),
            timeout_ms: Some(60000),
            enable_retry: true,
            max_retries: 3,
            retry_interval_ms: 1000,
        }
    }
}

impl StrategyConfig {
    /// 创建新的策略配置
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            ..Default::default()
        }
    }

    /// 设置最大并发数
    pub fn with_parallelism(mut self, parallelism: usize) -> Self {
        self.max_parallelism = Some(parallelism);
        self
    }

    /// 设置超时时间
    pub fn with_timeout(mut self, timeout_ms: u64) -> Self {
        self.timeout_ms = Some(timeout_ms);
        self
    }

    /// 配置重试策略
    pub fn with_retry(mut self, max_retries: u32, interval_ms: u64) -> Self {
        self.enable_retry = true;
        self.max_retries = max_retries;
        self.retry_interval_ms = interval_ms;
        self
    }

    /// 禁用重试
    pub fn without_retry(mut self) -> Self {
        self.enable_retry = false;
        self
    }
}

/// 编排策略 Trait - 定义编排引擎的核心接口
///
/// 编排策略定义了如何执行 DAG 中的任务节点。
///
/// # 设计原则
///
/// - **策略模式**: 不同策略实现不同执行语义
/// - **异步优先**: 所有执行操作都是异步的
/// - **可组合**: 支持嵌套和组合策略
///
/// # 实现要求
///
/// 实现此 Trait 需要提供：
///
/// - `name()` - 返回策略名称
/// - `config()` - 返回策略配置
/// - `execute()` - 执行编排逻辑
/// - `validate()` - 验证 DAG 结构
/// - `plan()` - 创建执行计划
#[async_trait]
pub trait OrchestrationStrategy: Send + Sync {
    /// 获取策略名称
    fn name(&self) -> &str;
    
    /// 获取策略配置
    fn config(&self) -> &StrategyConfig;
    
    /// 执行编排
    ///
    /// 根据 DAG 和执行上下文执行编排。
    ///
    /// # 参数
    ///
    /// - `dag`: 要执行的 DAG
    /// - `ctx`: 执行上下文
    ///
    /// # 返回
    ///
    /// - `Result<ExecutionResult>` - 执行结果
    async fn execute(&self, dag: &DAG, ctx: &ExecutionContext) -> Result<ExecutionResult>;
    
    /// 验证 DAG 结构
    ///
    /// 检查 DAG 是否满足策略的执行要求。
    ///
    /// # 参数
    ///
    /// - `dag`: 要验证的 DAG
    ///
    /// # 返回
    ///
    /// - `Result<()>` - 验证成功返回 Ok
    fn validate(&self, dag: &DAG) -> Result<()>;
    
    /// 创建执行计划
    ///
    /// 根据 DAG 创建执行计划，描述执行顺序和并行关系。
    ///
    /// # 参数
    ///
    /// - `dag`: 要规划的 DAG
    ///
    /// # 返回
    ///
    /// - `Result<ExecutionPlan>` - 执行计划
    fn plan(&self, dag: &DAG) -> Result<ExecutionPlan>;
    
    /// 回滚执行（可选）
    ///
    /// 当执行失败时，回滚已完成的操作。
    ///
    /// # 参数
    ///
    /// - `dag`: DAG 引用（如果可用）
    /// - `ctx`: 执行上下文
    ///
    /// # 默认实现
    ///
    /// 默认实现为空操作。
    async fn rollback(&self, dag: Option<&DAG>, ctx: &ExecutionContext) -> Result<()> {
        let _ = (dag, ctx);
        Ok(())
    }
}

/// 执行结果
///
/// 包含整个编排的执行结果。
///
/// # 字段说明
///
/// - `success`: 是否全部成功
/// - `node_results`: 每个节点的执行结果
/// - `total_duration_ms`: 总执行时间（毫秒）
/// - `start_time`: 开始时间戳
/// - `end_time`: 结束时间戳
/// - `error`: 错误信息（如果有）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResult {
    /// 是否成功
    pub success: bool,
    /// 节点执行结果
    pub node_results: Vec<NodeExecutionResult>,
    /// 总执行时间（毫秒）
    pub total_duration_ms: u64,
    /// 开始时间戳
    pub start_time: i64,
    /// 结束时间戳
    pub end_time: i64,
    /// 错误信息
    pub error: Option<String>,
}

impl ExecutionResult {
    /// 创建成功结果
    ///
    /// # 参数
    ///
    /// - `node_results`: 节点执行结果列表
    /// - `total_duration_ms`: 总执行时间
    pub fn success(node_results: Vec<NodeExecutionResult>, total_duration_ms: u64) -> Self {
        let now = chrono::Utc::now().timestamp_millis();
        Self {
            success: true,
            node_results,
            total_duration_ms,
            start_time: now - total_duration_ms as i64,
            end_time: now,
            error: None,
        }
    }

    /// 创建失败结果
    ///
    /// # 参数
    ///
    /// - `error`: 错误信息
    /// - `node_results`: 节点执行结果列表
    /// - `total_duration_ms`: 总执行时间
    pub fn failure(error: String, node_results: Vec<NodeExecutionResult>, total_duration_ms: u64) -> Self {
        let now = chrono::Utc::now().timestamp_millis();
        Self {
            success: false,
            node_results,
            total_duration_ms,
            start_time: now - total_duration_ms as i64,
            end_time: now,
            error: Some(error),
        }
    }

    /// 检查是否成功
    pub fn is_success(&self) -> bool {
        self.success
    }

    /// 获取失败节点数量
    pub fn failed_count(&self) -> usize {
        self.node_results.iter().filter(|r| !r.success).count()
    }

    /// 获取成功节点数量
    pub fn succeeded_count(&self) -> usize {
        self.node_results.iter().filter(|r| r.success).count()
    }

    /// 获取总节点数量
    pub fn total_count(&self) -> usize {
        self.node_results.len()
    }
}

/// 任务执行结果（用于策略接口）
///
/// 描述单个任务的执行结果。
///
/// # 字段说明
///
/// - `task_id`: 任务ID
/// - `name`: 任务名称
/// - `success`: 是否成功
/// - `output`: 输出数据
/// - `error`: 错误信息
/// - `duration_ms`: 执行时间
#[derive(Debug, Clone)]
pub struct TaskExecutionResult {
    /// 任务ID
    pub task_id: String,
    /// 任务名称
    pub name: String,
    /// 是否成功
    pub success: bool,
    /// 输出数据
    pub output: Option<TaskOutput>,
    /// 错误信息
    pub error: Option<String>,
    /// 执行时间（毫秒）
    pub duration_ms: u64,
}

impl TaskExecutionResult {
    /// 创建成功结果
    pub fn success(task_id: String, name: String, output: TaskOutput, duration_ms: u64) -> Self {
        Self {
            task_id,
            name,
            success: true,
            output: Some(output),
            error: None,
            duration_ms,
        }
    }

    /// 创建失败结果
    pub fn failure(task_id: String, name: String, error: String, duration_ms: u64) -> Self {
        Self {
            task_id,
            name,
            success: false,
            output: None,
            error: Some(error),
            duration_ms,
        }
    }
}
