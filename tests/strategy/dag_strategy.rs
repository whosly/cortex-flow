//! DAG 策略集成测试

use crate::common;
use cortex_flow::context::ExecutionContext;
use cortex_flow::strategy::{DAGStrategy, OrchestrationStrategy, StrategyConfig};

#[tokio::test]
async fn test_dag_strategy_execute_linear() {
    let strategy = DAGStrategy::default();
    let dag = common::build_linear_dag();
    let ctx = ExecutionContext::new();

    let result = strategy.execute(&dag, &ctx).await.unwrap();
    assert!(result.is_success());
    assert_eq!(result.total_count(), 3);
}

#[tokio::test]
async fn test_dag_strategy_execute_diamond() {
    let strategy = DAGStrategy::default();
    let dag = common::build_diamond_dag();
    let ctx = ExecutionContext::new();

    let result = strategy.execute(&dag, &ctx).await.unwrap();
    assert!(result.is_success());
    assert_eq!(result.total_count(), 4);
}

#[test]
fn test_dag_strategy_validate_valid_dag() {
    let strategy = DAGStrategy::default();
    let dag = common::build_linear_dag();
    assert!(strategy.validate(&dag).is_ok());
}

#[test]
fn test_dag_strategy_plan_linear() {
    let strategy = DAGStrategy::default();
    let dag = common::build_linear_dag();
    let plan = strategy.plan(&dag).unwrap();

    assert_eq!(plan.execution_order.len(), 3);
    assert!(!plan.layers.is_empty());
}

#[test]
fn test_dag_strategy_plan_diamond() {
    let strategy = DAGStrategy::default();
    let dag = common::build_diamond_dag();
    let plan = strategy.plan(&dag).unwrap();

    assert_eq!(plan.execution_order.len(), 4);
    // Diamond: A -> B, C -> D, so at least 3 layers
    assert!(plan.layers.len() >= 3);
}

#[tokio::test]
async fn test_dag_strategy_with_config() {
    let config = StrategyConfig::new("test-dag").with_parallelism(2);
    let strategy = DAGStrategy::new(config);
    let dag = common::build_linear_dag();
    let ctx = ExecutionContext::new();

    let result = strategy.execute(&dag, &ctx).await.unwrap();
    assert!(result.is_success());
    assert_eq!(strategy.name(), "test-dag");
}
