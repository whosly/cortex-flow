//! # 策略注册表
//!
//! 支持运行时动态注册和查找编排策略。
//!
//! ## 使用示例
//!
//! ```rust,ignore
//! use cortex_flow::strategy::StrategyRegistry;
//! use cortex_flow::strategy::{OrchestrationStrategy, StrategyType};
//!
//! let mut registry = StrategyRegistry::new();
//! registry.register(StrategyType::Dag, || Box::new(DAGStrategy::default()));
//! let strategy = registry.create(&StrategyType::Dag)?;
//! ```

use super::dag_strategy::DAGStrategy;
use super::sequential_strategy::SequentialStrategy;
use super::{OrchestrationStrategy, StrategyConfig, StrategyType};
use crate::error::{Error, Result};
use std::collections::HashMap;

/// 策略工厂函数类型
type StrategyFactoryFn = Box<dyn Fn() -> Box<dyn OrchestrationStrategy> + Send + Sync>;

/// 策略注册表
///
/// 管理编排策略的注册和创建。支持运行时动态注册新的策略类型。
pub struct StrategyRegistry {
    /// 已注册的策略工厂
    factories: HashMap<StrategyType, StrategyFactoryFn>,
}

impl StrategyRegistry {
    /// 创建新的策略注册表
    ///
    /// 默认注册 DAG 和 Sequential 两种策略。
    pub fn new() -> Self {
        let mut registry = Self {
            factories: HashMap::new(),
        };
        registry.register(StrategyType::Dag, || Box::new(DAGStrategy::default()));
        registry.register(StrategyType::Sequential, || {
            Box::new(SequentialStrategy::default())
        });
        registry
    }

    /// 创建空的策略注册表（不注册任何内置策略）
    pub fn empty() -> Self {
        Self {
            factories: HashMap::new(),
        }
    }

    /// 注册策略
    ///
    /// # 参数
    ///
    /// - `strategy_type`: 策略类型
    /// - `factory`: 策略工厂函数
    ///
    /// # 注意
    ///
    /// 如果已存在相同类型的策略，会被覆盖。
    pub fn register<F>(&mut self, strategy_type: StrategyType, factory: F)
    where
        F: Fn() -> Box<dyn OrchestrationStrategy> + Send + Sync + 'static,
    {
        self.factories.insert(strategy_type, Box::new(factory));
    }

    /// 创建策略实例
    ///
    /// # 错误
    ///
    /// 如果策略类型未注册，返回 `Error::Strategy`。
    pub fn create(&self, strategy_type: &StrategyType) -> Result<Box<dyn OrchestrationStrategy>> {
        self.factories
            .get(strategy_type)
            .ok_or_else(|| Error::Strategy(format!("Strategy {:?} not registered", strategy_type)))
            .map(|f| f())
    }

    /// 创建带配置的策略实例
    ///
    /// 如果是内置策略类型，使用给定配置创建。
    /// 如果是自定义策略类型，使用工厂创建（忽略配置）。
    pub fn create_with_config(
        &self,
        strategy_type: &StrategyType,
        config: StrategyConfig,
    ) -> Result<Box<dyn OrchestrationStrategy>> {
        match strategy_type {
            StrategyType::Dag => Ok(Box::new(DAGStrategy::new(config))),
            StrategyType::Sequential => Ok(Box::new(SequentialStrategy::new(config))),
            other => self
                .factories
                .get(other)
                .ok_or_else(|| Error::Strategy(format!("Strategy {:?} not registered", other)))
                .map(|f| f()),
        }
    }

    /// 检查策略是否已注册
    pub fn is_registered(&self, strategy_type: &StrategyType) -> bool {
        self.factories.contains_key(strategy_type)
    }

    /// 获取已注册的策略类型列表
    pub fn registered_types(&self) -> Vec<StrategyType> {
        self.factories.keys().copied().collect()
    }

    /// 注销策略
    pub fn unregister(&mut self, strategy_type: &StrategyType) -> bool {
        self.factories.remove(strategy_type).is_some()
    }

    /// 获取已注册策略数量
    pub fn len(&self) -> usize {
        self.factories.len()
    }

    /// 检查注册表是否为空
    pub fn is_empty(&self) -> bool {
        self.factories.is_empty()
    }

    /// 按名称创建策略
    ///
    /// 支持的名称：`"dag"`, `"sequential"`（不区分大小写）
    pub fn create_by_name(&self, name: &str) -> Result<Box<dyn OrchestrationStrategy>> {
        let strategy_type = Self::parse_type_name(name)?;
        self.create(&strategy_type)
    }

    /// 按名称创建策略（带配置）
    pub fn create_by_name_with_config(
        &self,
        name: &str,
        config: StrategyConfig,
    ) -> Result<Box<dyn OrchestrationStrategy>> {
        let strategy_type = Self::parse_type_name(name)?;
        self.create_with_config(&strategy_type, config)
    }

    /// 验证注册的策略完整性
    ///
    /// 检查每个已注册的策略工厂是否能正常创建实例。
    /// 返回验证失败的策略类型列表。
    pub fn validate_registered(&self) -> Vec<(StrategyType, String)> {
        let mut errors = Vec::new();
        for strategy_type in self.factories.keys() {
            if let Err(e) = self.create(strategy_type) {
                errors.push((*strategy_type, e.to_string()));
            }
        }
        errors
    }

    /// 解析策略类型名称
    fn parse_type_name(name: &str) -> Result<StrategyType> {
        match name.to_lowercase().as_str() {
            "dag" => Ok(StrategyType::Dag),
            "sequential" | "seq" => Ok(StrategyType::Sequential),
            "consensus" => Ok(StrategyType::Consensus),
            "mapreduce" | "map_reduce" => Ok(StrategyType::MapReduce),
            _ => Err(Error::Strategy(format!(
                "Unknown strategy name: '{}'. Available: dag, sequential",
                name
            ))),
        }
    }
}

impl Default for StrategyRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_registry_has_builtin_strategies() {
        let registry = StrategyRegistry::new();
        assert!(registry.is_registered(&StrategyType::Dag));
        assert!(registry.is_registered(&StrategyType::Sequential));
        assert_eq!(registry.len(), 2);
    }

    #[test]
    fn test_create_dag_strategy() {
        let registry = StrategyRegistry::new();
        let strategy = registry.create(&StrategyType::Dag).unwrap();
        assert_eq!(strategy.name(), "default");
    }

    #[test]
    fn test_create_unregistered_strategy() {
        let registry = StrategyRegistry::new();
        let result = registry.create(&StrategyType::Consensus);
        assert!(result.is_err());
    }

    #[test]
    fn test_register_custom_strategy() {
        let mut registry = StrategyRegistry::new();
        registry.register(StrategyType::Consensus, || Box::new(DAGStrategy::default()));
        assert!(registry.is_registered(&StrategyType::Consensus));
        let strategy = registry.create(&StrategyType::Consensus).unwrap();
        assert_eq!(strategy.name(), "default");
    }

    #[test]
    fn test_unregister_strategy() {
        let mut registry = StrategyRegistry::new();
        assert!(registry.unregister(&StrategyType::Sequential));
        assert!(!registry.is_registered(&StrategyType::Sequential));
    }
}
