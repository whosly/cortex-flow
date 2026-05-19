//! 顺序策略集成测试

use crate::common;
use cortex_flow::context::ExecutionContext;
use cortex_flow::strategy::{OrchestrationStrategy, SequentialStrategy, StrategyConfig};

/// 顺序策略执行线性 DAG，验证逐个执行成功
#[tokio::test]
async fn test_sequential_strategy_execute_linear() {
    let strategy = SequentialStrategy::default();
    let dag = common::build_linear_dag();
    let ctx = ExecutionContext::new();

    let result = strategy.execute(&dag, &ctx).await.unwrap();
    assert!(result.is_success());
    assert_eq!(result.total_count(), 3);
}

/// 顺序策略执行菱形 DAG，验证 4 个任务全部成功
#[tokio::test]
async fn test_sequential_strategy_execute_diamond() {
    let strategy = SequentialStrategy::default();
    let dag = common::build_diamond_dag();
    let ctx = ExecutionContext::new();

    let result = strategy.execute(&dag, &ctx).await.unwrap();
    assert!(result.is_success());
    assert_eq!(result.total_count(), 4);
}

/// 顺序策略执行计划，验证每层仅 1 个节点（严格串行）
#[test]
fn test_sequential_strategy_plan() {
    let strategy = SequentialStrategy::default();
    let dag = common::build_linear_dag();
    let plan = strategy.plan(&dag).unwrap();

    // 顺序策略中每个层级只有一个节点
    assert_eq!(plan.layers.len(), 3);
    for layer in &plan.layers {
        assert_eq!(layer.len(), 1);
    }
}

/// 顺序策略自定义配置，验证 name() 返回自定义名称
#[test]
fn test_sequential_strategy_with_config() {
    let config = StrategyConfig::new("test-seq").with_parallelism(1);
    let strategy = SequentialStrategy::new(config);
    assert_eq!(strategy.name(), "test-seq");
}
