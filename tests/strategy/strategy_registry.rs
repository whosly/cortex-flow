//! 策略注册表测试

use cortex_flow::strategy::{StrategyRegistry, StrategyType};

#[test]
fn test_registry_default_has_builtin_strategies() {
    let registry = StrategyRegistry::new();
    assert!(registry.is_registered(&StrategyType::Dag));
    assert!(registry.is_registered(&StrategyType::Sequential));
    assert_eq!(registry.len(), 2);
}

#[test]
fn test_registry_create_dag_strategy() {
    let registry = StrategyRegistry::new();
    let strategy = registry.create(&StrategyType::Dag).unwrap();
    assert_eq!(strategy.name(), "default");
}

#[test]
fn test_registry_create_sequential_strategy() {
    let registry = StrategyRegistry::new();
    let strategy = registry.create(&StrategyType::Sequential).unwrap();
    assert_eq!(strategy.name(), "sequential");
}

#[test]
fn test_registry_unregistered_returns_error() {
    let registry = StrategyRegistry::new();
    let result = registry.create(&StrategyType::Consensus);
    assert!(result.is_err());
}

#[test]
fn test_registry_register_custom_strategy() {
    let mut registry = StrategyRegistry::new();
    let initial_count = registry.len();

    registry.register(StrategyType::Consensus, || {
        Box::new(cortex_flow::strategy::DAGStrategy::default())
    });

    assert!(registry.is_registered(&StrategyType::Consensus));
    assert_eq!(registry.len(), initial_count + 1);

    let strategy = registry.create(&StrategyType::Consensus).unwrap();
    assert_eq!(strategy.name(), "default");
}

#[test]
fn test_registry_unregister() {
    let mut registry = StrategyRegistry::new();
    assert!(registry.unregister(&StrategyType::Sequential));
    assert!(!registry.is_registered(&StrategyType::Sequential));
}

#[test]
fn test_registry_registered_types() {
    let registry = StrategyRegistry::new();
    let types = registry.registered_types();
    assert!(types.contains(&StrategyType::Dag));
    assert!(types.contains(&StrategyType::Sequential));
}

#[test]
fn test_registry_empty() {
    let registry = StrategyRegistry::empty();
    assert!(registry.is_empty());
    assert_eq!(registry.len(), 0);
}

#[test]
fn test_registry_create_with_config() {
    use cortex_flow::strategy::StrategyConfig;

    let registry = StrategyRegistry::new();
    let config = StrategyConfig::new("custom-dag").with_parallelism(4);
    let strategy = registry
        .create_with_config(&StrategyType::Dag, config)
        .unwrap();
    assert_eq!(strategy.name(), "custom-dag");
}
