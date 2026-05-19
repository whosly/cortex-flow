//! # Simple Example
//!
//! 展示最基本的 DAG 任务编排

use cortex_flow::{task::SimpleTask, Orchestrator};
use serde_json::json;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let orchestrator = Orchestrator::builder().build().await?;

    // 定义三个顺序执行的任务
    let task1 = SimpleTask::new("step1", "First Step", |_input, _ctx| {
        Box::pin(async move { Ok(json!({"value": 1})) })
    });

    let task2 = SimpleTask::new("step2", "Second Step", |input, _ctx| {
        Box::pin(async move {
            let v = input.get("value").and_then(|x| x.as_i64()).unwrap_or(0);
            Ok(json!({"value": v + 10}))
        })
    });

    let task3 = SimpleTask::new("step3", "Third Step", |input, _ctx| {
        Box::pin(async move {
            let v = input.get("value").and_then(|x| x.as_i64()).unwrap_or(0);
            Ok(json!({"result": v * 2 }))
        })
    });

    // 构建并执行DAG
    let result = orchestrator
        .execute_dag(|dag| {
            dag.add_node(task1)
                .add_node(task2)
                .add_node(task3)
                .add_dependency("step1", "step2")
                .add_dependency("step2", "step3");
        })
        .await?;

    println!("DAG execution completed: {:?}", result);
    Ok(())
}
