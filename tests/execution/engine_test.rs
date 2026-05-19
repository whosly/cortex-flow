//! 执行引擎集成测试

use crate::common;
use cortex_flow::context::ExecutionContext;
use cortex_flow::execution::ExecutionEngine;
use cortex_flow::strategy::DAGStrategy;
use std::sync::Arc;

/// 执行引擎执行线性 DAG，验证成功
#[tokio::test]
async fn test_engine_execute_dag() {
    let strategy = Arc::new(DAGStrategy::default());
    let mut engine = ExecutionEngine::new(strategy);
    let dag = common::build_linear_dag();
    let ctx = ExecutionContext::new();

    let result = engine.execute_dag(&dag, &ctx).await.unwrap();
    assert!(result.is_success());
}

/// 执行引擎状态追踪，验证执行前后 is_completed 状态变化
#[tokio::test]
async fn test_engine_state_tracking() {
    let strategy = Arc::new(DAGStrategy::default());
    let mut engine = ExecutionEngine::new(strategy);
    let dag = common::build_linear_dag();
    let ctx = ExecutionContext::new();

    assert!(!engine.is_completed());

    let _ = engine.execute_dag(&dag, &ctx).await;

    assert!(engine.is_completed());
}

/// 执行引擎 with_dag 构造，验证未启动时 total_count 为 0
#[tokio::test]
async fn test_engine_with_dag() {
    let strategy = Arc::new(DAGStrategy::default());
    let dag = common::build_linear_dag();
    let engine = ExecutionEngine::with_dag(strategy, dag.clone());

    assert_eq!(engine.total_count(), 0); // not started yet
}
