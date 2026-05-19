//! # 策略工厂

use crate::error::{Error, Result};
use crate::strategy::{
    dag_strategy::DAGStrategy, sequential_strategy::SequentialStrategy, OrchestrationStrategy,
    StrategyConfig, StrategyType,
};

/// 策略工厂 - 用于创建策略实例
pub struct StrategyFactory;

impl StrategyFactory {
    /// 创建指定类型的策略
    pub fn create(strategy_type: StrategyType) -> Result<Box<dyn OrchestrationStrategy>> {
        match strategy_type {
            StrategyType::Dag => Ok(Box::new(DAGStrategy::default())),
            StrategyType::Sequential => Ok(Box::new(SequentialStrategy::default())),
            _ => Err(Error::Strategy(format!(
                "Strategy {:?} not yet implemented. Available: Dag, Sequential",
                strategy_type
            ))),
        }
    }

    /// 创建指定类型的策略（带配置）
    pub fn create_with_config(
        strategy_type: StrategyType,
        config: StrategyConfig,
    ) -> Result<Box<dyn OrchestrationStrategy>> {
        match strategy_type {
            StrategyType::Dag => Ok(Box::new(DAGStrategy::new(config))),
            StrategyType::Sequential => Ok(Box::new(SequentialStrategy::new(config))),
            _ => Err(Error::Strategy(format!(
                "Strategy {:?} not yet implemented. Available: Dag, Sequential",
                strategy_type
            ))),
        }
    }

    /// 创建默认DAG策略
    pub fn dag_strategy() -> Box<dyn OrchestrationStrategy> {
        Box::new(DAGStrategy::default())
    }

    /// 创建DAG策略（带配置）
    pub fn dag_strategy_with(config: StrategyConfig) -> Box<dyn OrchestrationStrategy> {
        Box::new(DAGStrategy::new(config))
    }

    /// 创建默认顺序策略
    pub fn sequential_strategy() -> Box<dyn OrchestrationStrategy> {
        Box::new(SequentialStrategy::default())
    }

    /// 创建顺序策略（带配置）
    pub fn sequential_strategy_with(config: StrategyConfig) -> Box<dyn OrchestrationStrategy> {
        Box::new(SequentialStrategy::new(config))
    }
}
