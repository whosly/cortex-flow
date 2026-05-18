//! # 并行执行示例
//!
//! 展示DAG中的并行执行能力

use cortex_flow::{
    Orchestrator,
    task::SimpleTask,
};
use serde_json::json;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let orchestrator = Orchestrator::builder()
        .with_tracing(true)
        .with_metrics(true)
        .build()
        .await?;

    // 任务A: 收集数据
    let task_a = SimpleTask::new("A", "Collect Data", |_input, ctx| {
        let ctx = ctx.clone();
        Box::pin(async move {
            println!("[A] Collecting data...");
            ctx.log_with_task("info", "Starting data collection", "A");
            tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
            Ok(json!({"data": vec![1, 2, 3]}))
        })
    });

    // 任务B: 处理数据 (依赖A)
    let task_b = SimpleTask::new("B", "Process Data", |input, ctx| {
        let ctx = ctx.clone();
        Box::pin(async move {
            let data = input.get("data").map(|d| d.clone()).unwrap_or_default();
            println!("[B] Processing data: {:?}", data);
            ctx.log_with_task("info", "Processing data", "B");
            tokio::time::sleep(tokio::time::Duration::from_millis(150)).await;
            Ok(json!({"processed": data}))
        })
    });

    // 任务C: 生成报告 (依赖B)
    let task_c = SimpleTask::new("C", "Generate Report", |input, ctx| {
        let ctx = ctx.clone();
        Box::pin(async move {
            let processed = input.get("processed").map(|d| d.clone()).unwrap_or_default();
            println!("[C] Generating report from: {:?}", processed);
            ctx.log_with_task("info", "Generating report", "C");
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            Ok(json!({"report": "Summary Report"}))
        })
    });

    // 任务D: 发送通知 (依赖A) - 与B并行
    let task_d = SimpleTask::new("D", "Send Notification", |input, ctx| {
        let ctx = ctx.clone();
        Box::pin(async move {
            println!("[D] Sending notification...");
            ctx.log_with_task("info", "Sending notification", "D");
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            Ok(json!({"notification": "sent"}))
        })
    });

    // 构建DAG: A -> B -> C, A -> D (B和D并行)
    let result = orchestrator
        .execute_dag(|dag| {
            dag.add_node(task_a)
                .add_node(task_b)
                .add_node(task_c)
                .add_node(task_d)
                .add_dependency("A", "B")
                .add_dependency("B", "C")
                .add_dependency("A", "D");
        })
        .await?;

    println!("\n执行结果:");
    println!("成功: {}", result.success);
    println!("执行任务数: {}", result.node_results.len());
    
    for task in &result.node_results {
        println!("  - {}: success={}, output={:?}", task.task_id, task.success, task.output);
    }

    Ok(())
}
