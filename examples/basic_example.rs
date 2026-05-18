//! # Basic DAG Example
//!
//! 展示带 tracing 和 metrics 的基本 DAG 任务编排

use ai_agent_scheduler::{
    Orchestrator,
    task::SimpleTask,
};
use serde_json::json;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("=== AI Agent Scheduler Demo ===\n");

    // 创建编排器
    let orchestrator = Orchestrator::builder()
        .with_tracing(true)
        .with_metrics(true)
        .build()
        .await?;

    println!("Orchestrator created successfully");

    // 创建任务1: 数据收集
    let collect_task = SimpleTask::new("collect", "Data Collection", |_input, ctx| {
        let ctx = ctx.clone();
        Box::pin(async move {
            println!("[collect] Starting data collection...");
            ctx.log_with_task("info", "Collecting data", "collect");
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            Ok(json!({
                "data": ["item1", "item2", "item3"],
                "count": 3
            }))
        })
    });

    // 创建任务2: 数据处理
    let process_task = SimpleTask::new("process", "Data Processing", |input, ctx| {
        let ctx = ctx.clone();
        Box::pin(async move {
            let data = input.get("data").and_then(|d| d.as_array());
            println!("[process] Processing data: {:?}", data);
            ctx.log_with_task("info", "Processing data", "process");
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            Ok(json!({
                "processed": true,
                "items": data.map(|a| a.len()).unwrap_or(0)
            }))
        })
    });

    // 创建任务3: 结果生成
    let generate_task = SimpleTask::new("generate", "Result Generation", |input, ctx| {
        let ctx = ctx.clone();
        Box::pin(async move {
            let processed = input.get("processed").and_then(|v| v.as_bool()).unwrap_or(false);
            let items = input.get("items").and_then(|v| v.as_u64()).unwrap_or(0);
            println!("[generate] Generating result for {} items, processed={}", items, processed);
            ctx.log_with_task("info", "Generating result", "generate");
            tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
            Ok(json!({
                "summary": format!("Processed {} items successfully", items),
                "status": "completed"
            }))
        })
    });

    // 使用DAGBuilder构建DAG
    println!("\nBuilding DAG...");
    let result = orchestrator
        .execute_dag(|dag| {
            dag.add_node(collect_task)
                .add_node(process_task)
                .add_node(generate_task)
                .add_dependency("collect", "process")
                .add_dependency("process", "generate");
        })
        .await;

    match result {
        Ok(exec_result) => {
            println!("\n=== Execution Result ===");
            println!("Success: {}", exec_result.success);
            let completed = exec_result.node_results.iter().filter(|r| r.success).count();
            let failed = exec_result.node_results.len() - completed;
            println!("Tasks executed: {}", exec_result.node_results.len());
            println!("Tasks completed: {}", completed);
            println!("Tasks failed: {}", failed);
            println!("\nTask Results:");
            for task_result in &exec_result.node_results {
                println!(
                    "  - {} ({}) {:?}",
                    task_result.task_id,
                    task_result.name,
                    if task_result.success { "OK" } else { "FAILED" }
                );
                if let Some(ref output) = task_result.output {
                    println!("    Output: {}", output);
                }
                if let Some(ref error) = task_result.error {
                    println!("    Error: {}", error);
                }
            }
        }
        Err(e) => {
            println!("\nExecution failed: {:?}", e);
        }
    }

    println!("\n=== Demo completed ===");
    Ok(())
}
