//! # 编排策略模块
//!
//! 定义可扩展的编排策略接口和内置实现。
//!
//! ## 模块结构
//!
//! - [`strategy_trait`] - 定义编排策略的核心 Trait 接口
//! - [`factory`] - 策略工厂，用于创建策略实例
//! - [`dag_strategy`] - DAG编排策略的默认实现
//!
//! ## 核心概念
//!
//! ### 编排策略 (OrchestrationStrategy)
//!
//! 编排策略定义了如何执行 DAG 中的任务节点。主要接口包括：
//!
//! - `execute()` - 执行编排
//! - `validate()` - 验证 DAG 结构
//! - `plan()` - 创建执行计划
//! - `rollback()` - 回滚执行（可选）
//!
//! ### 执行计划 (ExecutionPlan)
//!
//! 执行计划描述了任务的执行顺序和并行关系：
//!
//! - `execution_order` - 拓扑排序的执行顺序
//! - `layers` - 可并行执行的层级
//!
//! ### 执行结果 (ExecutionResult)
//!
//! 执行结果包含整个编排的执行结果：
//!
//! - `success` - 是否全部成功
//! - `node_results` - 每个节点的执行结果
//! - `total_duration_ms` - 总执行时间
//!
//! ## 策略类型
//!
//! 框架支持多种编排策略：
//!
//! | 策略类型 | 说明 |
//! |---------|------|
//! | Dag | 默认 DAG 编排策略，按拓扑排序执行 |
//! | Sequential | 顺序执行策略 |
//! | MapReduce | MapReduce 风格的分发收集策略 |
//! | Consensus | 共识策略，用于多节点一致性 |
//! | Hierarchical | 层级策略，支持嵌套 DAG |
//! | ProducerChecker | 生产者-检查者策略 |

use crate::error::Error;
use serde::{Deserialize, Serialize};

// 导出核心类型
pub use strategy_trait::{
    ExecutionResult, OrchestrationStrategy, StrategyConfig, TaskExecutionResult,
};
// 从 execution 模块重新导出 ExecutionPlan
pub use crate::execution::ExecutionPlan;
pub use dag_strategy::DAGStrategy;
pub use factory::StrategyFactory;
pub use registry::StrategyRegistry;
pub use sequential_strategy::SequentialStrategy;

mod dag_strategy;
mod factory;
mod registry;
mod sequential_strategy;
mod strategy_trait;

/// 编排策略类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum StrategyType {
    /// DAG编排策略
    Dag,
    /// 顺序执行策略
    Sequential,
    /// MapReduce策略
    MapReduce,
    /// 共识策略
    Consensus,
    /// 层级策略
    Hierarchical,
    /// 生产者-检查者策略
    ProducerChecker,
}

#[allow(clippy::derivable_impls)]
impl Default for StrategyType {
    fn default() -> Self {
        Self::Dag
    }
}

impl std::fmt::Display for StrategyType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StrategyType::Dag => write!(f, "dag"),
            StrategyType::Sequential => write!(f, "sequential"),
            StrategyType::MapReduce => write!(f, "map_reduce"),
            StrategyType::Consensus => write!(f, "consensus"),
            StrategyType::Hierarchical => write!(f, "hierarchical"),
            StrategyType::ProducerChecker => write!(f, "producer_checker"),
        }
    }
}

impl std::str::FromStr for StrategyType {
    type Err = Error;
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "dag" => Ok(StrategyType::Dag),
            "sequential" => Ok(StrategyType::Sequential),
            "map_reduce" | "mapreduce" => Ok(StrategyType::MapReduce),
            "consensus" => Ok(StrategyType::Consensus),
            "hierarchical" => Ok(StrategyType::Hierarchical),
            "producer_checker" | "producerchecker" => Ok(StrategyType::ProducerChecker),
            _ => Err(Error::Strategy(format!("Unknown strategy type: {}", s))),
        }
    }
}
