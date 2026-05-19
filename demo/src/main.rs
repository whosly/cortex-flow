//! # CortexFlow 数据分析管道 Demo
//!
//! 演示如何使用 CortexFlow 构建一个数据分析管道：
//! - 数据获取 → 数据清洗 → 数据验证 (并行)
//! - 数据清洗 + 数据验证 → LLM 分析
//! - LLM 分析 → 报告生成 + 可视化导出 (并行)
//!
//! DAG 结构图:
//! ```
//!                    ┌──> [数据清洗] ──┐
//! [数据获取] ────────┤                 ├──> [LLM 分析] ──┬──> [报告生成]
//!                    └──> [数据验证] ──┘                  └──> [可视化导出]
//! ```

use chrono::Utc;
use cortex_flow::context::ExecutionContext;
use cortex_flow::error_recovery::{ErrorRecovery, RetryPolicy};
use cortex_flow::llm::{ChatRequest, LLMClientRegistry, LLMConfig, MockLLMClient};
use cortex_flow::task::SimpleTask;
use cortex_flow::{ExecutionProgress, Orchestrator};
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;

// ============================================================================
// 数据模型
// ============================================================================

/// 原始销售数据
#[derive(Debug, Clone, Serialize, Deserialize)]
struct SalesRecord {
    id: u32,
    product: String,
    amount: f64,
    region: String,
    date: String,
}

/// 清洗后的数据
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct CleanedData {
    total_records: usize,
    total_amount: f64,
    regions: Vec<String>,
    products: Vec<String>,
    monthly_trend: Vec<MonthlyData>,
}

/// 月度数据
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct MonthlyData {
    month: String,
    amount: f64,
}

/// 分析结果
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct AnalysisResult {
    summary: String,
    trend: String,
    top_region: String,
    growth_rate: f64,
    recommendations: Vec<String>,
}

/// 最终报告
#[derive(Debug, Clone, Serialize, Deserialize)]
struct AnalysisReport {
    title: String,
    generated_at: String,
    data_summary: String,
    analysis: AnalysisResult,
    visual_chart: String,
}

// ============================================================================
// 主函数
// ============================================================================

#[tokio::main]
async fn main() -> cortex_flow::Result<()> {
    println!("========================================================");
    println!("  CortexFlow 数据分析管道 Demo");
    println!("========================================================\n");

    let mode = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "basic".to_string());

    match mode.as_str() {
        "basic" => run_basic_pipeline().await?,
        "llm" => run_llm_pipeline().await?,
        "error-recovery" => run_error_recovery_demo().await?,
        "context" => run_context_demo().await?,
        "progress" => run_progress_demo().await?,
        "multi-model" => run_multi_model_demo().await?,
        "all" => {
            run_basic_pipeline().await?;
            println!("\n");
            run_llm_pipeline().await?;
            println!("\n");
            run_error_recovery_demo().await?;
            println!("\n");
            run_context_demo().await?;
            println!("\n");
            run_progress_demo().await?;
        }
        _ => {
            println!(
                "用法: cargo run -- [basic|llm|error-recovery|context|progress|multi-model|all]"
            );
            println!();
            println!("  basic           - 基础 DAG 数据管道（无 LLM）");
            println!("  llm             - 带 LLM 分析的数据管道");
            println!("  error-recovery  - 错误恢复与重试演示");
            println!("  context         - 执行上下文与数据传递演示");
            println!("  progress        - 带进度回调的管道演示");
            println!("  multi-model     - 多模型 LLM 配置与切换（P0-P5 全特性）");
            println!("  all             - 运行所有演示（不含 multi-model）");
        }
    }

    Ok(())
}

// ============================================================================
// Demo 1: 基础 DAG 数据管道
// ============================================================================

async fn run_basic_pipeline() -> cortex_flow::Result<()> {
    println!("--- Demo 1: 基础 DAG 数据管道 ---\n");

    let orchestrator = Orchestrator::builder()
        .with_max_parallelism(4)
        .build()
        .await?;

    println!("DAG 结构:");
    println!("  [数据获取] ──┬──> [数据清洗] ──┬──> [LLM 分析] ──┬──> [报告生成]");
    println!("               │                  │                  └──> [可视化导出]");
    println!("               └──> [数据验证] ──┘");
    println!();

    let result = orchestrator
        .execute_dag(|dag| {
            // 节点1: 数据获取
            dag.add_node(SimpleTask::new(
                "fetch_data",
                "获取销售数据",
                |_input, _ctx| {
                    Box::pin(async move {
                        let records = generate_sample_data();
                        println!("  [fetch_data] 获取到 {} 条销售记录", records.len());
                        Ok(serde_json::to_value(records).unwrap_or_default())
                    })
                },
            ));

            // 节点2: 数据清洗
            dag.add_node(SimpleTask::new(
                "clean_data",
                "清洗数据",
                |input, _ctx| {
                    Box::pin(async move {
                        let records: Vec<SalesRecord> =
                            serde_json::from_value(input).unwrap_or_default();
                        let cleaned = clean_data(&records);
                        println!(
                            "  [clean_data] 清洗完成: {} 条记录, {} 个区域",
                            cleaned.total_records,
                            cleaned.regions.len()
                        );
                        Ok(serde_json::to_value(&cleaned).unwrap_or_default())
                    })
                },
            ));

            // 节点3: 数据验证
            dag.add_node(SimpleTask::new(
                "validate_data",
                "验证数据",
                |input, _ctx| {
                    Box::pin(async move {
                        let records: Vec<SalesRecord> =
                            serde_json::from_value(input).unwrap_or_default();
                        let is_valid = validate_data(&records);
                        println!(
                            "  [validate_data] 数据验证: {}",
                            if is_valid { "通过" } else { "失败" }
                        );
                        Ok(serde_json::json!({ "is_valid": is_valid, "record_count": records.len() }))
                    })
                },
            ));

            // 节点4: 分析数据 (依赖清洗和验证)
            dag.add_node(SimpleTask::new(
                "analyze_data",
                "分析数据",
                |input, _ctx| {
                    Box::pin(async move {
                        // 多上游输入为 JSON 对象，key 为上游节点 ID
                        let cleaned: CleanedData = if let Some(cleaned_val) =
                            input.get("clean_data")
                        {
                            serde_json::from_value(cleaned_val.clone()).unwrap_or_default()
                        } else {
                            serde_json::from_value(input).unwrap_or_default()
                        };

                        let analysis = analyze_data(&cleaned);
                        println!(
                            "  [analyze_data] 分析完成: 趋势={}, 增长率={:.1}%",
                            analysis.trend, analysis.growth_rate * 100.0
                        );
                        Ok(serde_json::to_value(&analysis).unwrap_or_default())
                    })
                },
            ));

            // 节点5: 报告生成
            dag.add_node(SimpleTask::new(
                "generate_report",
                "生成报告",
                |input, _ctx| {
                    Box::pin(async move {
                        let analysis: AnalysisResult =
                            serde_json::from_value(input).unwrap_or_default();
                        let report = generate_report(&analysis, "无（基础模式）");
                        println!("  [generate_report] 报告已生成: {}", report.title);
                        Ok(serde_json::to_value(&report).unwrap_or_default())
                    })
                },
            ));

            // 节点6: 可视化导出
            dag.add_node(SimpleTask::new(
                "export_visual",
                "可视化导出",
                |input, _ctx| {
                    Box::pin(async move {
                        let analysis: AnalysisResult =
                            serde_json::from_value(input).unwrap_or_default();
                        let chart = generate_visual(&analysis);
                        println!("  [export_visual] 可视化图表已导出 ({} 字符)", chart.len());
                        Ok(serde_json::json!({
                            "chart": chart,
                            "format": "mermaid"
                        }))
                    })
                },
            ));

            // 定义依赖关系
            dag.add_dependency("fetch_data", "clean_data");
            dag.add_dependency("fetch_data", "validate_data");
            dag.add_dependency("clean_data", "analyze_data");
            dag.add_dependency("validate_data", "analyze_data");
            dag.add_dependency("analyze_data", "generate_report");
            dag.add_dependency("analyze_data", "export_visual");
        })
        .await?;

    print_result(&result);
    Ok(())
}

// ============================================================================
// Demo 2: 带 LLM 分析的数据管道
// ============================================================================

async fn run_llm_pipeline() -> cortex_flow::Result<()> {
    println!("--- Demo 2: 带 LLM 分析的数据管道 ---\n");

    // 使用 MockLLMClient 演示（无需真实 API Key）
    // 生产环境替换为: LLMConfig::openai("sk-xxx", "gpt-4")
    let mock_llm = MockLLMClient::new(
        r#"{"summary":"销售数据整体呈上升趋势","trend":"上升","top_region":"华东","growth_rate":0.15,"recommendations":["加大华东区投入","关注产品B库存","拓展华南市场"]}"#,
    );

    let orchestrator = Orchestrator::builder()
        .with_max_parallelism(4)
        .with_llm_config(LLMConfig::ollama("demo-model"))
        .build()
        .await?
        .with_llm_client(mock_llm.into_trait());

    let result = orchestrator
        .execute_dag(|dag| {
            dag.add_node(SimpleTask::new(
                "fetch_data",
                "获取销售数据",
                |_input, _ctx| {
                    Box::pin(async move {
                        let records = generate_sample_data();
                        Ok(serde_json::to_value(records).unwrap_or_default())
                    })
                },
            ));

            dag.add_node(SimpleTask::new(
                "clean_data",
                "清洗数据",
                |input, _ctx| {
                    Box::pin(async move {
                        let records: Vec<SalesRecord> =
                            serde_json::from_value(input).unwrap_or_default();
                        let cleaned = clean_data(&records);
                        Ok(serde_json::to_value(&cleaned).unwrap_or_default())
                    })
                },
            ));

            // LLM 分析节点
            dag.add_node(SimpleTask::new(
                "llm_analyze",
                "LLM 智能分析",
                |input, _ctx| {
                    Box::pin(async move {
                        let cleaned: CleanedData =
                            serde_json::from_value(input).unwrap_or_default();
                        println!(
                            "  [llm_analyze] 发送数据摘要到 LLM: {} 条记录, 总金额 {:.2}",
                            cleaned.total_records, cleaned.total_amount
                        );

                        // 在实际使用中，这里通过 orchestrator.llm_client() 调用 LLM
                        // 此处用本地模拟代替
                        let analysis = AnalysisResult {
                            summary: "销售数据整体呈上升趋势".to_string(),
                            trend: "上升".to_string(),
                            top_region: "华东".to_string(),
                            growth_rate: 0.15,
                            recommendations: vec![
                                "加大华东区投入".to_string(),
                                "关注产品B库存".to_string(),
                                "拓展华南市场".to_string(),
                            ],
                        };
                        Ok(serde_json::to_value(&analysis).unwrap_or_default())
                    })
                },
            ));

            dag.add_node(SimpleTask::new(
                "generate_report",
                "生成智能报告",
                |input, _ctx| {
                    Box::pin(async move {
                        let analysis: AnalysisResult =
                            serde_json::from_value(input).unwrap_or_default();
                        let report = generate_report(&analysis, "LLM 智能分析");
                        println!("  [generate_report] 智能报告已生成: {}", report.title);
                        Ok(serde_json::to_value(&report).unwrap_or_default())
                    })
                },
            ));

            dag.add_dependency("fetch_data", "clean_data");
            dag.add_dependency("clean_data", "llm_analyze");
            dag.add_dependency("llm_analyze", "generate_report");
        })
        .await?;

    print_result(&result);
    Ok(())
}

// ============================================================================
// Demo 3: 错误恢复与重试
// ============================================================================

async fn run_error_recovery_demo() -> cortex_flow::Result<()> {
    println!("--- Demo 3: 错误恢复与重试 ---\n");

    // 演示1: 指数退避重试
    println!("场景1: 指数退避重试（模拟第2次成功）");
    let policy = RetryPolicy::exponential(3, 100, 2.0).with_jitter(false);
    let mut recovery = ErrorRecovery::new(policy);
    let attempt_counter = Arc::new(AtomicU32::new(0));

    let result: cortex_flow::Result<String> = recovery
        .execute_with_retry(|| {
            let counter = attempt_counter.clone();
            async move {
                let count = counter.fetch_add(1, Ordering::SeqCst);
                if count == 0 {
                    println!("  第 {} 次尝试: 失败", count + 1);
                    Err(cortex_flow::Error::TaskExecution(
                        "模拟网络超时".to_string(),
                    ))
                } else {
                    println!("  第 {} 次尝试: 成功!", count + 1);
                    Ok("数据获取成功".to_string())
                }
            }
        })
        .await;

    println!("  重试结果: {:?}\n", result);

    // 演示2: 重试耗尽
    println!("场景2: 重试耗尽（始终失败）");
    let policy = RetryPolicy::fixed(2, 50);
    let mut recovery = ErrorRecovery::new(policy);
    let fail_counter = Arc::new(AtomicU32::new(0));

    let result: cortex_flow::Result<String> = recovery
        .execute_with_retry(|| {
            let counter = fail_counter.clone();
            async move {
                let count = counter.fetch_add(1, Ordering::SeqCst);
                println!("  第 {} 次尝试: 失败", count + 1);
                Err(cortex_flow::Error::LLMCall("API rate limit".to_string()))
            }
        })
        .await;

    println!("  重试结果: {}\n", result.unwrap_err());

    // 演示3: DAG 中的错误恢复
    println!("场景3: DAG 中某个节点失败");
    let orchestrator = Orchestrator::builder().build().await?;
    let result = orchestrator
        .execute_dag(|dag| {
            dag.add_node(SimpleTask::new("step1", "正常步骤", |_input, _ctx| {
                Box::pin(async move {
                    println!("  [step1] 执行成功");
                    Ok(serde_json::json!({ "data": "ok" }))
                })
            }));
            dag.add_node(SimpleTask::new("step2", "失败步骤", |_input, _ctx| {
                Box::pin(async move {
                    println!("  [step2] 执行失败!");
                    Err(cortex_flow::Error::TaskExecution(
                        "数据格式错误".to_string(),
                    ))
                })
            }));
            dag.add_node(SimpleTask::new("step3", "依赖步骤", |_input, _ctx| {
                Box::pin(async move {
                    println!("  [step3] 执行成功");
                    Ok(serde_json::json!({ "result": "done" }))
                })
            }));
            dag.add_dependency("step1", "step2");
            dag.add_dependency("step2", "step3");
        })
        .await?;

    println!("  DAG 执行结果: success={}", result.success);
    for node in &result.node_results {
        println!(
            "    节点 {}: {} - {:?}",
            node.task_id,
            if node.success { "成功" } else { "失败" },
            node.error.as_deref().unwrap_or("无错误")
        );
    }

    Ok(())
}

// ============================================================================
// Demo 4: 执行上下文与数据传递
// ============================================================================

async fn run_context_demo() -> cortex_flow::Result<()> {
    println!("--- Demo 4: 执行上下文与数据传递 ---\n");

    // 演示1: 基本上下文操作（在 DAG 外部使用，拥有 &mut 引用）
    let mut ctx = ExecutionContext::new();
    println!("Session ID: {}", ctx.session_id());

    ctx.set("pipeline_name", &"数据分析管道")?;
    ctx.set(
        "config",
        &serde_json::json!({ "threshold": 0.8, "max_retries": 3 }),
    )?;
    ctx.set("batch_size", &100usize)?;

    println!("设置上下文数据:");
    println!(
        "  pipeline_name = {}",
        ctx.get::<String>("pipeline_name").unwrap()
    );
    println!(
        "  config = {:?}",
        ctx.get::<serde_json::Value>("config").unwrap()
    );
    println!("  batch_size = {}", ctx.get::<usize>("batch_size").unwrap());

    // 日志
    ctx.log("info", "管道启动");
    ctx.log_with_task("debug", "数据获取完成", "fetch_data");
    ctx.log_with_task("warn", "数据质量较低", "validate_data");

    println!("\n上下文日志:");
    for log in ctx.logs() {
        let task_info = log
            .task_id
            .as_ref()
            .map(|t| format!(" [task: {}]", t))
            .unwrap_or_default();
        println!("  [{}] {}{}", log.level, log.message, task_info);
    }

    // 快照与恢复
    let snapshot = ctx.snapshot();
    ctx.set("temporary", &"临时数据")?;
    println!("\n添加临时数据后: {} 个键", ctx.len());

    ctx.restore(&snapshot);
    println!("恢复快照后: {} 个键", ctx.len());
    println!("临时数据是否存在: {}", ctx.contains("temporary"));

    // 演示2: DAG 中的数据传递
    // 注意：在 DAG 任务闭包中，ctx 是 &ExecutionContext（不可变引用）
    // 因此只能在闭包中读取 ctx，不能调用 ctx.set()
    // 数据传递方式：通过 DAG 依赖自动传递 input
    println!("\n--- DAG 中的数据传递 ---");
    let orchestrator = Orchestrator::builder().build().await?;
    let result = orchestrator
        .execute_dag(|dag| {
            dag.add_node(SimpleTask::new(
                "producer",
                "数据生产者",
                |_input, _ctx| {
                    Box::pin(async move {
                        println!("  [producer] 生成数据");
                        // 数据通过返回值传递给下游
                        Ok(serde_json::json!({
                            "direct_output": "通过依赖传递",
                            "value": 42
                        }))
                    })
                },
            ));
            dag.add_node(SimpleTask::new(
                "consumer",
                "数据消费者",
                |input, _ctx| {
                    Box::pin(async move {
                        // 通过 DAG 依赖传递接收上游输出
                        let direct = input
                            .get("direct_output")
                            .and_then(|v| v.as_str())
                            .unwrap_or("无");
                        let value = input.get("value").and_then(|v| v.as_u64()).unwrap_or(0);
                        println!("  [consumer] 依赖传递: direct={}, value={}", direct, value);
                        Ok(serde_json::json!({ "consumed": true }))
                    })
                },
            ));
            dag.add_dependency("producer", "consumer");
        })
        .await?;

    print_result(&result);
    Ok(())
}

// ============================================================================
// Demo 5: 带进度回调的管道
// ============================================================================

async fn run_progress_demo() -> cortex_flow::Result<()> {
    println!("--- Demo 5: 带进度回调的管道 ---\n");

    let orchestrator = Orchestrator::builder()
        .with_max_parallelism(2)
        .build()
        .await?;

    let completed_tasks = Arc::new(AtomicU32::new(0));
    let completed_clone = completed_tasks.clone();

    let result = orchestrator
        .execute_dag_with_progress(
            |dag| {
                dag.add_node(SimpleTask::new("fetch", "获取数据", |_input, _ctx| {
                    Box::pin(async move {
                        tokio::time::sleep(std::time::Duration::from_millis(200)).await;
                        println!("  [fetch] 完成");
                        Ok(serde_json::json!({ "data": "raw" }))
                    })
                }));
                dag.add_node(SimpleTask::new("clean", "清洗数据", |input, _ctx| {
                    Box::pin(async move {
                        tokio::time::sleep(std::time::Duration::from_millis(300)).await;
                        println!("  [clean] 完成");
                        Ok(serde_json::json!({ "data": "clean", "input_was": input }))
                    })
                }));
                dag.add_node(SimpleTask::new(
                    "validate",
                    "验证数据",
                    |input, _ctx| {
                        Box::pin(async move {
                            tokio::time::sleep(std::time::Duration::from_millis(150)).await;
                            println!("  [validate] 完成");
                            Ok(serde_json::json!({ "valid": true, "input_was": input }))
                        })
                    },
                ));
                dag.add_node(SimpleTask::new("analyze", "分析数据", |input, _ctx| {
                    Box::pin(async move {
                        tokio::time::sleep(std::time::Duration::from_millis(400)).await;
                        println!("  [analyze] 完成");
                        Ok(serde_json::json!({ "insight": "trend", "input_was": input }))
                    })
                }));
                dag.add_node(SimpleTask::new("report", "生成报告", |input, _ctx| {
                    Box::pin(async move {
                        tokio::time::sleep(std::time::Duration::from_millis(200)).await;
                        println!("  [report] 完成");
                        Ok(serde_json::json!({ "report": "done", "input_was": input }))
                    })
                }));
                dag.add_node(SimpleTask::new("export", "导出结果", |input, _ctx| {
                    Box::pin(async move {
                        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                        println!("  [export] 完成");
                        Ok(serde_json::json!({ "exported": true, "input_was": input }))
                    })
                }));

                dag.add_dependency("fetch", "clean");
                dag.add_dependency("fetch", "validate");
                dag.add_dependency("clean", "analyze");
                dag.add_dependency("validate", "analyze");
                dag.add_dependency("analyze", "report");
                dag.add_dependency("analyze", "export");
            },
            move |progress: ExecutionProgress| {
                let prev = completed_clone.load(Ordering::SeqCst);
                if progress.completed > prev as usize {
                    completed_clone.store(progress.completed as u32, Ordering::SeqCst);
                    let bar_width = 30;
                    let filled = (progress.percent as usize * bar_width) / 100;
                    let bar: String = "█".repeat(filled) + &"░".repeat(bar_width - filled);
                    print!(
                        "\r  进度: [{}] {}% ({}/{} 成功:{} 失败:{})",
                        bar,
                        progress.percent,
                        progress.completed,
                        progress.total,
                        progress.success_count,
                        progress.failure_count
                    );
                    use std::io::Write;
                    std::io::stdout().flush().ok();
                    if progress.completed == progress.total {
                        println!();
                    }
                }
            },
        )
        .await?;

    print_result(&result);
    Ok(())
}

// ============================================================================
// Demo 6: 多模型 LLM 配置与切换
// ============================================================================

async fn run_multi_model_demo() -> cortex_flow::Result<()> {
    println!("--- Demo 6: 多模型 LLM 配置与切换 ---\n");

    // ========================================================================
    // 1. LLMConfig::custom() 快捷构造（P0）
    // ========================================================================
    println!("=== 1. LLMConfig::custom() 快捷构造 ===\n");

    // 火山引擎豆包 - 使用 custom() 快捷构造，替代手动构建 struct
    let doubao_config = LLMConfig::custom(
        std::env::var("VOLCENGINE_API_KEY")
            .unwrap_or_else(|_| "your-volcengine-api-key".to_string()),
        "doubao-pro-32k",
        "https://ark.cn-beijing.volces.com/api/v3",
    )
    .with_temperature(0.7)
    .with_max_tokens(4096);

    println!("[火山引擎豆包] 使用 LLMConfig::custom() 构造");
    println!("  provider: {}", doubao_config.provider);
    println!("  model: {}", doubao_config.model);
    println!("  base_url: {:?}", doubao_config.base_url);
    println!();

    // DeepSeek - 同样使用 custom()
    let deepseek_config = LLMConfig::custom(
        std::env::var("DEEPSEEK_API_KEY").unwrap_or_else(|_| "your-deepseek-api-key".to_string()),
        "deepseek-chat",
        "https://api.deepseek.com/v1",
    );
    println!("[DeepSeek] 使用 LLMConfig::custom() 构造");
    println!("  provider: {}", deepseek_config.provider);
    println!("  model: {}", deepseek_config.model);
    println!();

    // ========================================================================
    // 2. ChatRequest 请求级参数覆盖（P1）
    // ========================================================================
    println!("=== 2. ChatRequest 请求级参数覆盖 ===\n");

    // 创建一个默认 GPT-4 客户端
    let default_config = LLMConfig::openai("sk-demo", "gpt-4");
    let client = cortex_flow::llm::LLMClient::new(default_config);

    println!("默认配置 model: {}", client.config().model);

    // 使用 ChatRequest 临时切换模型，无需创建新客户端
    let request = ChatRequest::new(vec![cortex_flow::llm::Message::user("快速分析")])
        .with_model("gpt-3.5-turbo") // 临时切换到更便宜的模型
        .with_temperature(0.3) // 降低温度获取确定性结果
        .with_max_tokens(512); // 限制输出长度

    println!(
        "ChatRequest 覆盖: model={}, temperature={:?}, max_tokens={:?}",
        request.model.as_deref().unwrap_or("default"),
        request.temperature,
        request.max_tokens
    );
    println!();

    // ========================================================================
    // 3. Orchestrator Builder 多模型注册（P2+P3）
    // ========================================================================
    println!("=== 3. Orchestrator Builder 多模型注册 ===\n");

    // 使用 Mock 演示，生产环境替换为真实配置
    let orchestrator = Orchestrator::builder()
        .with_max_parallelism(4)
        // 使用 with_llm() 注册多个命名模型
        .with_llm("gpt4", LLMConfig::openai("sk-demo-key", "gpt-4"))
        .with_llm("llama3", LLMConfig::ollama("llama3"))
        .with_llm(
            "doubao",
            LLMConfig::custom(
                "demo-key",
                "doubao-pro-32k",
                "https://ark.cn-beijing.volces.com/api/v3",
            ),
        )
        .with_default_llm("gpt4") // 设置默认模型
        .build()
        .await?;

    // 运行时注册额外模型
    orchestrator.register_llm("mock", MockLLMClient::new(
        r#"{"summary":"销售数据整体呈上升趋势","trend":"上升","top_region":"华东","growth_rate":0.15}"#,
    ).into_trait());

    // 通过名称获取客户端
    println!("已注册模型: {:?}", orchestrator.llm_registry().names());
    println!("默认模型: {:?}", orchestrator.llm_registry().default_name());
    println!("获取 gpt4: {}", orchestrator.get_llm("gpt4").is_some());
    println!("获取 doubao: {}", orchestrator.get_llm("doubao").is_some());
    println!();

    // ========================================================================
    // 4. ExecutionContext LLM 访问（P5） - 推荐方式
    // ========================================================================
    println!("=== 4. ExecutionContext LLM 访问（推荐方式）===\n");

    println!("DAG 结构（任务通过 ctx.get_llm() 获取模型）:");
    println!("  [数据获取] ──> [数据清洗] ──┬──> [GPT-4 深度分析]  ──┐");
    println!("                             ├──> [Llama3 本地校验]  ──┼──> [豆包报告润色] ──> [Token 统计]");
    println!("                             └──> [数据统计]         ──┘");
    println!();

    let result = orchestrator
        .execute_dag(|dag| {
            // 节点1: 数据获取
            dag.add_node(SimpleTask::new(
                "fetch_data",
                "获取销售数据",
                |_input, _ctx| {
                    Box::pin(async move {
                        let records = generate_sample_data();
                        println!("  [fetch_data] 获取 {} 条销售记录", records.len());
                        Ok(serde_json::to_value(records).unwrap_or_default())
                    })
                },
            ));

            // 节点2: 数据清洗
            dag.add_node(SimpleTask::new(
                "clean_data",
                "清洗数据",
                |input, _ctx| {
                    Box::pin(async move {
                        let records: Vec<SalesRecord> =
                            serde_json::from_value(input).unwrap_or_default();
                        let cleaned = clean_data(&records);
                        println!(
                            "  [clean_data] 清洗完成: {} 条记录, 总额 {:.2}",
                            cleaned.total_records, cleaned.total_amount
                        );
                        Ok(serde_json::to_value(&cleaned).unwrap_or_default())
                    })
                },
            ));

            // 节点3: GPT-4 深度分析 - 通过 ctx.get_llm("gpt4") 获取
            dag.add_node(SimpleTask::new(
                "gpt4_analysis",
                "GPT-4 深度分析",
                |input, ctx| {
                    // P5 特性：在 async 块之前提取 Arc（'static，可安全 move）
                    let client = ctx.get_llm("gpt4").expect("gpt4 未注册");
                    Box::pin(async move {
                        let cleaned: CleanedData =
                            serde_json::from_value(input).unwrap_or_default();
                        println!(
                            "  [gpt4_analysis] 使用 GPT-4 分析 ({} 条记录)...",
                            cleaned.total_records
                        );

                        // 使用 ChatRequest 覆盖参数（P1）
                        let request = ChatRequest::new(vec![
                            cortex_flow::llm::Message::system("你是数据分析专家"),
                            cortex_flow::llm::Message::user(format!(
                                "分析以下销售数据: {} 条记录, 总额 {:.2}",
                                cleaned.total_records, cleaned.total_amount
                            )),
                        ])
                        .with_temperature(0.7);

                        // Mock 模式下使用默认 chat
                        let _ = client;
                        let _ = request;
                        Ok(serde_json::json!({
                            "model": "gpt-4",
                            "insight": "深度分析: 销售趋势上升，华东区领先",
                            "confidence": 0.92
                        }))
                    })
                },
            ));

            // 节点4: Llama3 本地校验 - 通过 ctx.get_llm("llama3") 获取
            dag.add_node(SimpleTask::new(
                "llama3_validate",
                "Llama3 本地校验",
                |input, ctx| {
                    let client = ctx.get_llm("llama3").expect("llama3 未注册");
                    Box::pin(async move {
                        let cleaned: CleanedData =
                            serde_json::from_value(input).unwrap_or_default();
                        println!(
                            "  [llama3_validate] 使用本地 Llama3 校验 ({} 条)...",
                            cleaned.total_records
                        );
                        let _ = client;
                        Ok(serde_json::json!({
                            "model": "llama3",
                            "validation": "数据质量良好",
                            "anomalies": 0
                        }))
                    })
                },
            ));

            // 节点5: 数据统计（无需 LLM）
            dag.add_node(SimpleTask::new(
                "statistics",
                "统计计算",
                |input, _ctx| {
                    Box::pin(async move {
                        let cleaned: CleanedData =
                            serde_json::from_value(input).unwrap_or_default();
                        println!("  [statistics] 计算统计指标...");
                        Ok(serde_json::json!({
                            "avg_amount": cleaned.total_amount / cleaned.total_records as f64,
                            "region_count": cleaned.regions.len(),
                            "product_count": cleaned.products.len()
                        }))
                    })
                },
            ));

            // 节点6: 豆包报告润色 - 通过 ctx.get_llm("doubao") 获取
            dag.add_node(SimpleTask::new(
                "doubao_report",
                "豆包报告润色",
                |_input, ctx| {
                    let client = ctx.get_llm("doubao").expect("doubao 未注册");
                    Box::pin(async move {
                        println!("  [doubao_report] 使用豆包润色报告...");
                        let _ = client;
                        Ok(serde_json::json!({
                            "model": "doubao-pro-32k",
                            "polished_report": "2024年度销售分析报告：整体增长稳健，华东区域表现突出，建议持续投入。",
                            "language_quality": "excellent"
                        }))
                    })
                },
            ));

            // 节点7: Token 统计（P4 特性）
            dag.add_node(SimpleTask::new(
                "token_summary",
                "Token 使用统计",
                |_input, ctx| {
                    // P5 特性：在 async 块之前提取快照数据
                    let snapshot = ctx.token_snapshot();
                    let llm_names = ctx.llm_names();
                    Box::pin(async move {
                        if let Some(snap) = snapshot {
                            println!("  [token_summary] Token 使用统计:");
                            println!("    调用次数: {}", snap.call_count);
                            println!("    总 Token: {}", snap.total_tokens);
                            println!("    估算成本: ${:.4}", snap.total_cost);
                        }
                        println!("  [token_summary] 可用模型: {:?}", llm_names);
                        Ok(serde_json::json!({
                            "token_tracking": "enabled"
                        }))
                    })
                },
            ));

            // 定义依赖
            dag.add_dependency("fetch_data", "clean_data");
            dag.add_dependency("clean_data", "gpt4_analysis");
            dag.add_dependency("clean_data", "llama3_validate");
            dag.add_dependency("clean_data", "statistics");
            dag.add_dependency("gpt4_analysis", "doubao_report");
            dag.add_dependency("llama3_validate", "doubao_report");
            dag.add_dependency("statistics", "doubao_report");
            dag.add_dependency("doubao_report", "token_summary");
        })
        .await?;

    print_result(&result);

    // ========================================================================
    // 5. TokenTracker 聚合统计（P4）
    // ========================================================================
    println!("\n=== 5. TokenTracker 聚合统计 ===\n");

    // Orchestrator 级别的 Token 快照（所有模型聚合）
    let snapshot = orchestrator.token_snapshot();
    println!("所有模型聚合 Token 使用:");
    println!("{}", snapshot);

    // ========================================================================
    // 6. LLMClientRegistry 独立使用（P2）
    // ========================================================================
    println!("=== 6. LLMClientRegistry 独立使用 ===\n");

    let registry = LLMClientRegistry::new();

    // 方式一：通过配置自动创建（自动关联 TokenTracker）
    registry.register_with_config("gpt4", LLMConfig::openai("sk-xxx", "gpt-4"));
    registry.register_with_config("llama3", LLMConfig::ollama("llama3"));
    registry.register_with_config(
        "doubao",
        LLMConfig::custom(
            "key",
            "doubao-pro-32k",
            "https://ark.cn-beijing.volces.com/api/v3",
        ),
    );

    // 方式二：传入已有客户端
    registry.register("mock", MockLLMClient::new("mock response").into_trait());

    println!("注册表中的模型: {:?}", registry.names());
    println!("默认模型: {:?}", registry.default_name());
    println!("包含 gpt4: {}", registry.contains("gpt4"));
    println!("模型数量: {}", registry.len());
    println!();

    // ========================================================================
    // 7. 完整特性对比总结
    // ========================================================================
    println!("=== 7. CortexFlow 多模型特性总结 ===\n");
    println!("P0: LLMConfig::custom()      - 豆包/DeepSeek 等 OpenAI 兼容 API 快捷构造");
    println!("P1: ChatRequest               - 请求级别覆盖 model/temperature/max_tokens");
    println!("P2: LLMClientRegistry         - HashMap 注册表，按名称管理多模型");
    println!("P3: Orchestrator 多模型集成    - Builder.with_llm()/with_default_llm() 链式注册");
    println!("P4: TokenTracker 自动集成     - 调用后自动记录，注册表级聚合统计");
    println!("P5: ExecutionContext LLM 访问 - ctx.get_llm()/ctx.default_llm() 零 Arc 克隆");
    println!();
    println!("--- 推荐用法 ---");
    println!("1. Orchestrator::builder().with_llm(\"name\", config).build()");
    println!("2. DAG 任务中: ctx.get_llm(\"name\") 获取客户端");
    println!("3. 临时切换: ChatRequest::new(msg).with_model(\"other-model\")");
    println!("4. 查看成本: orchestrator.token_snapshot()");

    Ok(())
}

// ============================================================================
// 辅助函数
// ============================================================================

fn generate_sample_data() -> Vec<SalesRecord> {
    vec![
        SalesRecord {
            id: 1,
            product: "产品A".into(),
            amount: 1500.0,
            region: "华东".into(),
            date: "2024-01".into(),
        },
        SalesRecord {
            id: 2,
            product: "产品B".into(),
            amount: 2300.0,
            region: "华北".into(),
            date: "2024-01".into(),
        },
        SalesRecord {
            id: 3,
            product: "产品A".into(),
            amount: 1800.0,
            region: "华东".into(),
            date: "2024-02".into(),
        },
        SalesRecord {
            id: 4,
            product: "产品C".into(),
            amount: 950.0,
            region: "华南".into(),
            date: "2024-02".into(),
        },
        SalesRecord {
            id: 5,
            product: "产品B".into(),
            amount: 2800.0,
            region: "华东".into(),
            date: "2024-03".into(),
        },
        SalesRecord {
            id: 6,
            product: "产品A".into(),
            amount: 2100.0,
            region: "华北".into(),
            date: "2024-03".into(),
        },
        SalesRecord {
            id: 7,
            product: "产品C".into(),
            amount: 1200.0,
            region: "华南".into(),
            date: "2024-03".into(),
        },
        SalesRecord {
            id: 8,
            product: "产品B".into(),
            amount: 3100.0,
            region: "华东".into(),
            date: "2024-04".into(),
        },
        SalesRecord {
            id: 9,
            product: "产品A".into(),
            amount: 2400.0,
            region: "华北".into(),
            date: "2024-04".into(),
        },
        SalesRecord {
            id: 10,
            product: "产品C".into(),
            amount: 1500.0,
            region: "华南".into(),
            date: "2024-04".into(),
        },
    ]
}

fn clean_data(records: &[SalesRecord]) -> CleanedData {
    let total_amount: f64 = records.iter().map(|r| r.amount).sum();
    let regions: Vec<String> = records
        .iter()
        .map(|r| r.region.clone())
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();
    let products: Vec<String> = records
        .iter()
        .map(|r| r.product.clone())
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();

    let mut monthly: std::collections::HashMap<String, f64> = std::collections::HashMap::new();
    for r in records {
        *monthly.entry(r.date.clone()).or_insert(0.0) += r.amount;
    }
    let monthly_trend: Vec<MonthlyData> = monthly
        .into_iter()
        .map(|(month, amount)| MonthlyData { month, amount })
        .collect();

    CleanedData {
        total_records: records.len(),
        total_amount,
        regions,
        products,
        monthly_trend,
    }
}

fn validate_data(records: &[SalesRecord]) -> bool {
    !records.is_empty()
        && records
            .iter()
            .all(|r| r.amount > 0.0 && !r.product.is_empty())
}

fn analyze_data(cleaned: &CleanedData) -> AnalysisResult {
    let top_region = cleaned
        .regions
        .first()
        .cloned()
        .unwrap_or_else(|| "未知".to_string());

    let growth_rate = if cleaned.monthly_trend.len() >= 2 {
        let first = cleaned
            .monthly_trend
            .first()
            .map(|m| m.amount)
            .unwrap_or(1.0);
        let last = cleaned
            .monthly_trend
            .last()
            .map(|m| m.amount)
            .unwrap_or(1.0);
        (last - first) / first
    } else {
        0.0
    };

    let trend = if growth_rate > 0.0 {
        "上升".to_string()
    } else {
        "下降".to_string()
    };

    AnalysisResult {
        summary: format!(
            "共 {} 条记录，总金额 {:.2}，覆盖 {} 个区域",
            cleaned.total_records,
            cleaned.total_amount,
            cleaned.regions.len()
        ),
        trend,
        top_region,
        growth_rate,
        recommendations: vec![
            format!(
                "重点关注 {} 区域",
                cleaned.regions.first().unwrap_or(&"未知".to_string())
            ),
            "建议增加产品多样性".to_string(),
            "关注月度销售波动".to_string(),
        ],
    }
}

fn generate_report(analysis: &AnalysisResult, source: &str) -> AnalysisReport {
    AnalysisReport {
        title: format!(
            "数据分析报告 - {} ({})",
            Utc::now().format("%Y-%m-%d"),
            source
        ),
        generated_at: Utc::now().to_rfc3339(),
        data_summary: analysis.summary.clone(),
        analysis: analysis.clone(),
        visual_chart: generate_visual(analysis),
    }
}

fn generate_visual(analysis: &AnalysisResult) -> String {
    format!(
        "graph TD\n  A[数据源] --> B[分析引擎]\n  B --> C{{趋势: {}}}\n  B --> D{{区域: {}}}\n  B --> E{{增长: {:.1}%}}",
        analysis.trend, analysis.top_region, analysis.growth_rate * 100.0
    )
}

fn print_result(result: &cortex_flow::strategy::ExecutionResult) {
    println!("\n--- 执行结果 ---");
    println!("整体状态: {}", if result.success { "成功" } else { "失败" });
    println!("总耗时: {}ms", result.total_duration_ms);
    println!("节点结果:");
    for node in &result.node_results {
        let status = if node.success { "OK" } else { "FAIL" };
        println!(
            "  [{}] {} ({}) - {}ms",
            status, node.task_id, node.name, node.duration_ms
        );
        if let Some(ref err) = node.error {
            println!("         错误: {}", err);
        }
    }
}
