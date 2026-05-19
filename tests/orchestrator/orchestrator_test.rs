//! 编排器集成测试

use crate::common;
use cortex_flow::Orchestrator;

#[tokio::test]
async fn test_orchestrator_build() {
    let orchestrator = Orchestrator::builder().build().await;
    assert!(orchestrator.is_ok());
}

#[tokio::test]
async fn test_orchestrator_execute_dag() {
    let orchestrator = Orchestrator::builder()
        .with_max_parallelism(4)
        .build()
        .await
        .unwrap();

    let result = orchestrator
        .execute_dag(|dag| {
            dag.add_node(common::simple_task("step1", "Step 1"))
                .add_node(common::simple_task("step2", "Step 2"))
                .add_dependency("step1", "step2");
        })
        .await;

    assert!(result.is_ok());
    let exec_result = result.unwrap();
    assert!(exec_result.is_success());
    assert_eq!(exec_result.total_count(), 2);
}

#[tokio::test]
async fn test_orchestrator_execute_parallel_dag() {
    let orchestrator = Orchestrator::builder()
        .with_max_parallelism(4)
        .build()
        .await
        .unwrap();

    let result = orchestrator
        .execute_dag(|dag| {
            dag.add_node(common::simple_task("a", "Task A"))
                .add_node(common::simple_task("b", "Task B"))
                .add_node(common::simple_task("c", "Task C"))
                .add_dependency("a", "b")
                .add_dependency("a", "c");
        })
        .await;

    assert!(result.is_ok());
    let exec_result = result.unwrap();
    assert!(exec_result.is_success());
    assert_eq!(exec_result.total_count(), 3);
}

#[tokio::test]
async fn test_orchestrator_execute_with_progress() {
    let orchestrator = Orchestrator::builder().build().await.unwrap();

    let progress_calls = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let progress_calls_clone = progress_calls.clone();

    let result = orchestrator
        .execute_dag_with_progress(
            |dag| {
                dag.add_node(common::simple_task("a", "Task A"))
                    .add_node(common::simple_task("b", "Task B"))
                    .add_dependency("a", "b");
            },
            move |_progress| {
                progress_calls_clone.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            },
        )
        .await;

    assert!(result.is_ok());
    assert!(progress_calls.load(std::sync::atomic::Ordering::SeqCst) > 0);
}

#[tokio::test]
async fn test_orchestrator_strategy_registry() {
    let orchestrator = Orchestrator::builder().build().await.unwrap();

    // 验证内置策略已注册
    let registry = orchestrator.strategy_registry();
    assert!(registry.is_registered(&cortex_flow::strategy::StrategyType::Dag));
    assert!(registry.is_registered(&cortex_flow::strategy::StrategyType::Sequential));
}

#[tokio::test]
async fn test_orchestrator_config() {
    let orchestrator = Orchestrator::builder()
        .with_max_parallelism(8)
        .build()
        .await
        .unwrap();

    let config = orchestrator.config();
    assert_eq!(config.execution.max_workers, 8);
}

#[tokio::test]
async fn test_orchestrator_single_task() {
    let orchestrator = Orchestrator::builder().build().await.unwrap();

    let task = cortex_flow::task::SimpleTask::new("single", "Single Task", |_input, _ctx| {
        Box::pin(async move { Ok(serde_json::json!({"done": true})) })
    });

    let result = orchestrator.execute_task(task).await;
    assert!(result.is_ok());
    let value = result.unwrap();
    assert_eq!(value["done"], true);
}
