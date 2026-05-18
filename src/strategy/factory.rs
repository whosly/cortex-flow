//! # 策略工厂

use crate::error::{Error, Result};
use crate::strategy::{OrchestrationStrategy, StrategyType, StrategyConfig, dag_strategy::DAGStrategy};

/// 策略工厂 - 用于创建策略实例
pub struct StrategyFactory;

impl StrategyFactory {
    /// 创建指定类型的策略
    pub fn create(strategy_type: StrategyType) -> Result<Box<dyn OrchestrationStrategy>> {
        match strategy_type {
            StrategyType::Dag => Ok(Box::new(DAGStrategy::default())),
            _ => Err(Error::Strategy(format!("Strategy {:?} not yet implemented", strategy_type))),
        }
    }

    /// 创建指定类型的策略（带配置）
    pub fn create_with_config(
        strategy_type: StrategyType,
        config: StrategyConfig,
    ) -> Result<Box<dyn OrchestrationStrategy>> {
        match strategy_type {
            StrategyType::Dag => Ok(Box::new(DAGStrategy::new(config))),
            _ => Err(Error::Strategy(format!("Strategy {:?} not yet implemented", strategy_type))),
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
}
