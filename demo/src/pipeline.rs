//! 管道定义与执行逻辑

use crate::models::{NodeResult, NodeSpec, PipelineResult, PipelineSpec};
use crate::state::ProgressEvent;
use cortex_flow::llm::MockLLMClient;
use cortex_flow::task::SimpleTask;
use cortex_flow::Orchestrator;
use std::sync::Arc;
use tokio::sync::watch;

/// 获取所有可用的管道定义
pub fn get_pipelines() -> Vec<PipelineSpec> {
    vec![
        PipelineSpec {
            id: "basic".into(),
            name: "基础 DAG 数据管道".into(),
            description: "数据获取 → 数据清洗 → 数据验证 (并行) → 数据分析 → 报告生成 + 可视化导出 (并行)".into(),
            nodes: vec![
                NodeSpec { id: "fetch_data".into(), name: "获取销售数据".into(), node_type: "data".into() },
                NodeSpec { id: "clean_data".into(), name: "清洗数据".into(), node_type: "transform".into() },
                NodeSpec { id: "validate_data".into(), name: "验证数据".into(), node_type: "transform".into() },
                NodeSpec { id: "analyze_data".into(), name: "分析数据".into(), node_type: "transform".into() },
                NodeSpec { id: "generate_report".into(), name: "报告生成".into(), node_type: "transform".into() },
                NodeSpec { id: "export_visual".into(), name: "可视化导出".into(), node_type: "transform".into() },
            ],
            edges: vec![
                ("fetch_data".into(), "clean_data".into()),
                ("fetch_data".into(), "validate_data".into()),
                ("clean_data".into(), "analyze_data".into()),
                ("validate_data".into(), "analyze_data".into()),
                ("analyze_data".into(), "generate_report".into()),
                ("analyze_data".into(), "export_visual".into()),
            ],
            dag_mermaid: r#"flowchart TD
    fetch_data[获取销售数据] --> clean_data[清洗数据]
    fetch_data --> validate_data[验证数据]
    clean_data --> analyze_data[分析数据]
    validate_data --> analyze_data
    analyze_data --> generate_report[报告生成]
    analyze_data --> export_visual[可视化导出]"#.into(),
        },
        PipelineSpec {
            id: "llm".into(),
            name: "LLM 智能分析管道".into(),
            description: "数据获取 → 数据清洗 → LLM 智能分析 → 生成智能报告".into(),
            nodes: vec![
                NodeSpec { id: "fetch_data".into(), name: "获取销售数据".into(), node_type: "data".into() },
                NodeSpec { id: "clean_data".into(), name: "清洗数据".into(), node_type: "transform".into() },
                NodeSpec { id: "llm_analyze".into(), name: "LLM 智能分析".into(), node_type: "llm".into() },
                NodeSpec { id: "generate_report".into(), name: "生成智能报告".into(), node_type: "transform".into() },
            ],
            edges: vec![
                ("fetch_data".into(), "clean_data".into()),
                ("clean_data".into(), "llm_analyze".into()),
                ("llm_analyze".into(), "generate_report".into()),
            ],
            dag_mermaid: r#"flowchart TD
    fetch_data[获取销售数据] --> clean_data[清洗数据]
    clean_data --> llm_analyze[LLM 智能分析]
    llm_analyze --> generate_report[生成智能报告]"#.into(),
        },
        PipelineSpec {
            id: "multi-model".into(),
            name: "多模型协作管道".into(),
            description: "数据获取 → 数据清洗 → GPT-4 分析 / Llama3 校验 / 统计计算 (并行) → 豆包报告润色 → Token 统计".into(),
            nodes: vec![
                NodeSpec { id: "fetch_data".into(), name: "获取销售数据".into(), node_type: "data".into() },
                NodeSpec { id: "clean_data".into(), name: "清洗数据".into(), node_type: "transform".into() },
                NodeSpec { id: "gpt4_analysis".into(), name: "GPT-4 深度分析".into(), node_type: "llm".into() },
                NodeSpec { id: "llama3_validate".into(), name: "Llama3 本地校验".into(), node_type: "llm".into() },
                NodeSpec { id: "statistics".into(), name: "统计计算".into(), node_type: "transform".into() },
                NodeSpec { id: "doubao_report".into(), name: "豆包报告润色".into(), node_type: "llm".into() },
                NodeSpec { id: "token_summary".into(), name: "Token 使用统计".into(), node_type: "transform".into() },
            ],
            edges: vec![
                ("fetch_data".into(), "clean_data".into()),
                ("clean_data".into(), "gpt4_analysis".into()),
                ("clean_data".into(), "llama3_validate".into()),
                ("clean_data".into(), "statistics".into()),
                ("gpt4_analysis".into(), "doubao_report".into()),
                ("llama3_validate".into(), "doubao_report".into()),
                ("statistics".into(), "doubao_report".into()),
                ("doubao_report".into(), "token_summary".into()),
            ],
            dag_mermaid: r#"flowchart TD
    fetch_data[获取销售数据] --> clean_data[清洗数据]
    clean_data --> gpt4_analysis[GPT-4 深度分析]
    clean_data --> llama3_validate[Llama3 本地校验]
    clean_data --> statistics[统计计算]
    gpt4_analysis --> doubao_report[豆包报告润色]
    llama3_validate --> doubao_report
    statistics --> doubao_report
    doubao_report --> token_summary[Token 使用统计]"#.into(),
        },
        PipelineSpec {
            id: "context".into(),
            name: "执行上下文演示".into(),
            description: "展示上下文读写、日志、快照恢复、DAG 数据传递".into(),
            nodes: vec![
                NodeSpec { id: "producer".into(), name: "数据生产者".into(), node_type: "data".into() },
                NodeSpec { id: "consumer".into(), name: "数据消费者".into(), node_type: "transform".into() },
            ],
            edges: vec![
                ("producer".into(), "consumer".into()),
            ],
            dag_mermaid: r#"flowchart TD
    producer[数据生产者] --> consumer[数据消费者]"#.into(),
        },
        PipelineSpec {
            id: "error-recovery".into(),
            name: "错误恢复演示".into(),
            description: "模拟网络超时，演示指数退避重试和 DAG 节点失败处理".into(),
            nodes: vec![
                NodeSpec { id: "step1".into(), name: "正常步骤".into(), node_type: "data".into() },
                NodeSpec { id: "step2".into(), name: "失败步骤".into(), node_type: "transform".into() },
                NodeSpec { id: "step3".into(), name: "依赖步骤".into(), node_type: "transform".into() },
            ],
            edges: vec![
                ("step1".into(), "step2".into()),
                ("step2".into(), "step3".into()),
            ],
            dag_mermaid: r#"flowchart TD
    step1[正常步骤] --> step2[失败步骤]
    step2 --> step3[依赖步骤]"#.into(),
        },
    ]
}

/// 执行管道，发送 SSE 进度事件
pub async fn execute_pipeline(
    pipeline_id: &str,
    progress_tx: Arc<watch::Sender<Option<ProgressEvent>>>,
) -> PipelineResult {
    let pipelines = get_pipelines();
    let spec = pipelines.iter().find(|p| p.id == pipeline_id).cloned();

    let Some(_spec) = spec else {
        return PipelineResult {
            pipeline_id: pipeline_id.into(),
            success: false,
            total_duration_ms: 0,
            node_results: vec![NodeResult {
                task_id: "unknown".into(),
                name: "未知管道".into(),
                success: false,
                duration_ms: 0,
                error: Some(format!("Pipeline '{}' not found", pipeline_id)),
            }],
        };
    };

    // 创建 Orchestrator
    let orchestrator = if pipeline_id == "multi-model" {
        Orchestrator::builder()
            .with_max_parallelism(4)
            .with_llm("gpt4", cortex_flow::llm::LLMConfig::openai("sk-demo", "gpt-4"))
            .with_llm("llama3", cortex_flow::llm::LLMConfig::ollama("llama3"))
            .with_llm("doubao", cortex_flow::llm::LLMConfig::custom("demo-key", "doubao-pro-32k", "https://ark.cn-beijing.volces.com/api/v3"))
            .with_default_llm("gpt4")
            .build()
            .await
            .unwrap()
    } else if pipeline_id == "llm" {
        let mock = MockLLMClient::new(
            r#"{"summary":"销售数据整体呈上升趋势","trend":"上升","top_region":"华东","growth_rate":0.15,"recommendations":["加大华东区投入","关注产品B库存","拓展华南市场"]}"#,
        );
        Orchestrator::builder()
            .with_max_parallelism(4)
            .with_llm_config(cortex_flow::llm::LLMConfig::ollama("demo-model"))
            .build()
            .await
            .unwrap()
            .with_llm_client(mock.into_trait())
    } else {
        Orchestrator::builder()
            .with_max_parallelism(4)
            .build()
            .await
            .unwrap()
    };

    let start = std::time::Instant::now();
    let result = match pipeline_id {
        "basic" => run_basic(&orchestrator, progress_tx).await,
        "llm" => run_llm(&orchestrator, progress_tx).await,
        "context" => run_context(&orchestrator, progress_tx).await,
        "multi-model" => run_multi_model(&orchestrator, progress_tx).await,
        "error-recovery" => run_error_recovery(&orchestrator, progress_tx).await,
        _ => Err(cortex_flow::Error::TaskExecution("Unknown pipeline".into())),
    };

    let total_duration_ms = start.elapsed().as_millis() as u64;

    match result {
        Ok(r) => PipelineResult {
            pipeline_id: pipeline_id.into(),
            success: r.success,
            total_duration_ms,
            node_results: r.node_results.iter().map(|n| NodeResult {
                task_id: n.task_id.clone(),
                name: n.name.clone(),
                success: n.success,
                duration_ms: n.duration_ms,
                error: n.error.clone(),
            }).collect(),
        },
        Err(e) => PipelineResult {
            pipeline_id: pipeline_id.into(),
            success: false,
            total_duration_ms,
            node_results: vec![NodeResult {
                task_id: "error".into(),
                name: "执行错误".into(),
                success: false,
                duration_ms: total_duration_ms,
                error: Some(e.to_string()),
            }],
        },
    }
}

macro_rules! progress_send {
    ($tx:expr, $pipeline:expr, $node_id:expr, $node_name:expr, $status:expr, $completed:expr, $total:expr, $msg:expr) => {
        let _ = $tx.send(Some(ProgressEvent {
            pipeline: $pipeline.into(),
            node_id: $node_id.into(),
            node_name: $node_name.into(),
            status: $status.into(),
            percent: if $total > 0 { ($completed * 100) / $total } else { 0 },
            completed: $completed,
            total: $total,
            message: $msg.into(),
        }));
    };
}

/// 通用进度回调，创建一个 'static 闭包
fn make_progress_callback(
    tx: Arc<watch::Sender<Option<ProgressEvent>>>,
    pipeline_name: &'static str,
) -> impl Fn(cortex_flow::ExecutionProgress) + Send + Sync + 'static {
    move |progress: cortex_flow::ExecutionProgress| {
        let completed = progress.completed;
        let total_nodes = progress.total;
        let success = progress.success_count;
        let failed = progress.failure_count;
        let status = if failed > 0 { "failed" } else { "completed" };
        let msg = format!("已完成 {}/{} (成功:{} 失败:{})", completed, total_nodes, success, failed);
        progress_send!(tx, pipeline_name, "", "", status, completed, total_nodes, msg);
    }
}

async fn run_basic(
    orchestrator: &Orchestrator,
    progress_tx: Arc<watch::Sender<Option<ProgressEvent>>>,
) -> cortex_flow::Result<cortex_flow::ExecutionResult> {
    let total = 6;
    progress_send!(progress_tx, "basic", "", "", "started", 0, total, "管道启动");

    let result = orchestrator
        .execute_dag_with_progress(
            |dag| {
                dag.add_node(SimpleTask::new("fetch_data", "获取销售数据", |_input, _ctx| {
                    Box::pin(async move {
                        tokio::time::sleep(std::time::Duration::from_millis(300)).await;
                        let records = generate_sample_data();
                        Ok(serde_json::to_value(records).unwrap_or_default())
                    })
                }));
                dag.add_node(SimpleTask::new("clean_data", "清洗数据", |input, _ctx| {
                    Box::pin(async move {
                        tokio::time::sleep(std::time::Duration::from_millis(400)).await;
                        let records: Vec<SalesRecord> = serde_json::from_value(input).unwrap_or_default();
                        let cleaned = clean_data(&records);
                        Ok(serde_json::to_value(&cleaned).unwrap_or_default())
                    })
                }));
                dag.add_node(SimpleTask::new("validate_data", "验证数据", |input, _ctx| {
                    Box::pin(async move {
                        tokio::time::sleep(std::time::Duration::from_millis(200)).await;
                        let records: Vec<SalesRecord> = serde_json::from_value(input).unwrap_or_default();
                        let is_valid = validate_data(&records);
                        Ok(serde_json::json!({ "is_valid": is_valid, "record_count": records.len() }))
                    })
                }));
                dag.add_node(SimpleTask::new("analyze_data", "分析数据", |input, _ctx| {
                    Box::pin(async move {
                        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                        let cleaned: CleanedData = if let Some(v) = input.get("clean_data") {
                            serde_json::from_value(v.clone()).unwrap_or_default()
                        } else {
                            serde_json::from_value(input).unwrap_or_default()
                        };
                        let analysis = analyze_data(&cleaned);
                        Ok(serde_json::to_value(&analysis).unwrap_or_default())
                    })
                }));
                dag.add_node(SimpleTask::new("generate_report", "报告生成", |input, _ctx| {
                    Box::pin(async move {
                        tokio::time::sleep(std::time::Duration::from_millis(300)).await;
                        let analysis: AnalysisResult = serde_json::from_value(input).unwrap_or_default();
                        let report = generate_report(&analysis, "基础模式");
                        Ok(serde_json::to_value(&report).unwrap_or_default())
                    })
                }));
                dag.add_node(SimpleTask::new("export_visual", "可视化导出", |input, _ctx| {
                    Box::pin(async move {
                        tokio::time::sleep(std::time::Duration::from_millis(200)).await;
                        let analysis: AnalysisResult = serde_json::from_value(input).unwrap_or_default();
                        let chart = generate_visual(&analysis);
                        Ok(serde_json::json!({ "chart": chart, "format": "mermaid" }))
                    })
                }));

                dag.add_dependency("fetch_data", "clean_data");
                dag.add_dependency("fetch_data", "validate_data");
                dag.add_dependency("clean_data", "analyze_data");
                dag.add_dependency("validate_data", "analyze_data");
                dag.add_dependency("analyze_data", "generate_report");
                dag.add_dependency("analyze_data", "export_visual");
            },
            make_progress_callback(progress_tx.clone(), "basic"),
        )
        .await?;

    progress_send!(progress_tx, "basic", "", "", "done", total, total, "管道执行完成");
    Ok(result)
}

async fn run_llm(
    orchestrator: &Orchestrator,
    progress_tx: Arc<watch::Sender<Option<ProgressEvent>>>,
) -> cortex_flow::Result<cortex_flow::ExecutionResult> {
    let total = 4;
    progress_send!(progress_tx, "llm", "", "", "started", 0, total, "LLM 管道启动");

    let result = orchestrator
        .execute_dag_with_progress(
            |dag| {
                dag.add_node(SimpleTask::new("fetch_data", "获取销售数据", |_input, _ctx| {
                    Box::pin(async move {
                        tokio::time::sleep(std::time::Duration::from_millis(200)).await;
                        let records = generate_sample_data();
                        Ok(serde_json::to_value(records).unwrap_or_default())
                    })
                }));
                dag.add_node(SimpleTask::new("clean_data", "清洗数据", |input, _ctx| {
                    Box::pin(async move {
                        tokio::time::sleep(std::time::Duration::from_millis(300)).await;
                        let records: Vec<SalesRecord> = serde_json::from_value(input).unwrap_or_default();
                        let cleaned = clean_data(&records);
                        Ok(serde_json::to_value(&cleaned).unwrap_or_default())
                    })
                }));
                dag.add_node(SimpleTask::new("llm_analyze", "LLM 智能分析", |input, _ctx| {
                    Box::pin(async move {
                        tokio::time::sleep(std::time::Duration::from_millis(600)).await;
                        let cleaned: CleanedData = serde_json::from_value(input).unwrap_or_default();
                        let analysis = AnalysisResult {
                            summary: format!("共 {} 条记录，总额 {:.2}", cleaned.total_records, cleaned.total_amount),
                            trend: "上升".into(),
                            top_region: "华东".into(),
                            growth_rate: 0.15,
                            recommendations: vec!["加大华东区投入".into(), "关注产品B库存".into(), "拓展华南市场".into()],
                        };
                        Ok(serde_json::to_value(&analysis).unwrap_or_default())
                    })
                }));
                dag.add_node(SimpleTask::new("generate_report", "生成智能报告", |input, _ctx| {
                    Box::pin(async move {
                        tokio::time::sleep(std::time::Duration::from_millis(200)).await;
                        let analysis: AnalysisResult = serde_json::from_value(input).unwrap_or_default();
                        let report = generate_report(&analysis, "LLM 智能分析");
                        Ok(serde_json::to_value(&report).unwrap_or_default())
                    })
                }));

                dag.add_dependency("fetch_data", "clean_data");
                dag.add_dependency("clean_data", "llm_analyze");
                dag.add_dependency("llm_analyze", "generate_report");
            },
            make_progress_callback(progress_tx.clone(), "llm"),
        )
        .await?;

    progress_send!(progress_tx, "llm", "", "", "done", total, total, "LLM 管道执行完成");
    Ok(result)
}

async fn run_context(
    orchestrator: &Orchestrator,
    progress_tx: Arc<watch::Sender<Option<ProgressEvent>>>,
) -> cortex_flow::Result<cortex_flow::ExecutionResult> {
    let total = 2;
    progress_send!(progress_tx, "context", "", "", "started", 0, total, "上下文演示管道启动");

    let result = orchestrator
        .execute_dag_with_progress(
            |dag| {
                dag.add_node(SimpleTask::new("producer", "数据生产者", |_input, ctx| {
                    let session_id = ctx.session_id();
                    Box::pin(async move {
                        // DAG 任务中通过 ctx 记录日志（同步调用，不进入 async block）
                        println!("  [producer] session_id={}", session_id);
                        Ok(serde_json::json!({ "direct": "通过依赖传递", "value": 42 }))
                    })
                }));
                dag.add_node(SimpleTask::new("consumer", "数据消费者", |input, _ctx| {
                    let direct: Option<String> = input.get("direct").and_then(|v| v.as_str()).map(|s| s.to_string());
                    let value: Option<i64> = input.get("value").and_then(|v| v.as_i64());
                    Box::pin(async move {
                        // DAG 依赖传递：上游输出自动传给下游
                        println!("  [consumer] 依赖传递: direct={}, value={}", direct.unwrap_or_default(), value.unwrap_or_default());
                        Ok(serde_json::json!({ "consumed": true }))
                    })
                }));

                dag.add_dependency("producer", "consumer");
            },
            make_progress_callback(progress_tx.clone(), "context"),
        )
        .await?;

    progress_send!(progress_tx, "context", "", "", "done", total, total, "上下文演示管道执行完成");
    Ok(result)
}

async fn run_multi_model(
    orchestrator: &Orchestrator,
    progress_tx: Arc<watch::Sender<Option<ProgressEvent>>>,
) -> cortex_flow::Result<cortex_flow::ExecutionResult> {
    let total = 7;
    progress_send!(progress_tx, "multi-model", "", "", "started", 0, total, "多模型管道启动");

    let result = orchestrator
        .execute_dag_with_progress(
            |dag| {
                dag.add_node(SimpleTask::new("fetch_data", "获取销售数据", |_input, _ctx| {
                    Box::pin(async move {
                        tokio::time::sleep(std::time::Duration::from_millis(200)).await;
                        let records = generate_sample_data();
                        Ok(serde_json::to_value(records).unwrap_or_default())
                    })
                }));
                dag.add_node(SimpleTask::new("clean_data", "清洗数据", |input, _ctx| {
                    Box::pin(async move {
                        tokio::time::sleep(std::time::Duration::from_millis(300)).await;
                        let records: Vec<SalesRecord> = serde_json::from_value(input).unwrap_or_default();
                        let cleaned = clean_data(&records);
                        Ok(serde_json::to_value(&cleaned).unwrap_or_default())
                    })
                }));
                dag.add_node(SimpleTask::new("gpt4_analysis", "GPT-4 深度分析", |input, ctx| {
                    let _client = ctx.get_llm("gpt4");
                    Box::pin(async move {
                        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                        let _cleaned: CleanedData = serde_json::from_value(input).unwrap_or_default();
                        let _ = _client;
                        Ok(serde_json::json!({
                            "model": "gpt-4",
                            "insight": "深度分析: 销售趋势上升，华东区领先",
                            "confidence": 0.92
                        }))
                    })
                }));
                dag.add_node(SimpleTask::new("llama3_validate", "Llama3 本地校验", |input, ctx| {
                    let _client = ctx.get_llm("llama3");
                    Box::pin(async move {
                        tokio::time::sleep(std::time::Duration::from_millis(400)).await;
                        let _cleaned: CleanedData = serde_json::from_value(input).unwrap_or_default();
                        let _ = _client;
                        Ok(serde_json::json!({
                            "model": "llama3",
                            "validation": "数据质量良好",
                            "anomalies": 0
                        }))
                    })
                }));
                dag.add_node(SimpleTask::new("statistics", "统计计算", |input, _ctx| {
                    Box::pin(async move {
                        tokio::time::sleep(std::time::Duration::from_millis(200)).await;
                        let cleaned: CleanedData = serde_json::from_value(input).unwrap_or_default();
                        Ok(serde_json::json!({
                            "avg_amount": cleaned.total_amount / cleaned.total_records as f64,
                            "region_count": cleaned.regions.len(),
                            "product_count": cleaned.products.len()
                        }))
                    })
                }));
                dag.add_node(SimpleTask::new("doubao_report", "豆包报告润色", |_input, ctx| {
                    let _client = ctx.get_llm("doubao");
                    Box::pin(async move {
                        tokio::time::sleep(std::time::Duration::from_millis(400)).await;
                        let _ = _client;
                        Ok(serde_json::json!({
                            "model": "doubao-pro-32k",
                            "polished_report": "2024年度销售分析报告：整体增长稳健，华东区域表现突出，建议持续投入。",
                            "language_quality": "excellent"
                        }))
                    })
                }));
                dag.add_node(SimpleTask::new("token_summary", "Token 使用统计", |_input, ctx| {
                    let snapshot = ctx.token_snapshot();
                    Box::pin(async move {
                        let tokens = if let Some(s) = snapshot {
                            format!("调用:{} 总Token:{}", s.call_count, s.total_tokens)
                        } else {
                            "无数据".into()
                        };
                        Ok(serde_json::json!({ "token_tracking": tokens }))
                    })
                }));

                dag.add_dependency("fetch_data", "clean_data");
                dag.add_dependency("clean_data", "gpt4_analysis");
                dag.add_dependency("clean_data", "llama3_validate");
                dag.add_dependency("clean_data", "statistics");
                dag.add_dependency("gpt4_analysis", "doubao_report");
                dag.add_dependency("llama3_validate", "doubao_report");
                dag.add_dependency("statistics", "doubao_report");
                dag.add_dependency("doubao_report", "token_summary");
            },
            make_progress_callback(progress_tx.clone(), "multi-model"),
        )
        .await?;

    progress_send!(progress_tx, "multi-model", "", "", "done", total, total, "多模型管道执行完成");
    Ok(result)
}

async fn run_error_recovery(
    orchestrator: &Orchestrator,
    progress_tx: Arc<watch::Sender<Option<ProgressEvent>>>,
) -> cortex_flow::Result<cortex_flow::ExecutionResult> {
    let total = 3;
    progress_send!(progress_tx, "error-recovery", "", "", "started", 0, total, "错误恢复管道启动");

    let result = orchestrator
        .execute_dag_with_progress(
            |dag| {
                dag.add_node(SimpleTask::new("step1", "正常步骤", |_input, _ctx| {
                    Box::pin(async move {
                        tokio::time::sleep(std::time::Duration::from_millis(200)).await;
                        Ok(serde_json::json!({ "data": "ok" }))
                    })
                }));
                dag.add_node(SimpleTask::new("step2", "失败步骤", |_input, _ctx| {
                    Box::pin(async move {
                        tokio::time::sleep(std::time::Duration::from_millis(300)).await;
                        Err(cortex_flow::Error::TaskExecution("数据格式错误".into()))
                    })
                }));
                dag.add_node(SimpleTask::new("step3", "依赖步骤", |_input, _ctx| {
                    Box::pin(async move {
                        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                        Ok(serde_json::json!({ "result": "done" }))
                    })
                }));
                dag.add_dependency("step1", "step2");
                dag.add_dependency("step2", "step3");
            },
            make_progress_callback(progress_tx.clone(), "error-recovery"),
        )
        .await?;

    progress_send!(progress_tx, "error-recovery", "", "", "done", total, total, "错误恢复管道执行完成");
    Ok(result)
}

// ============================================================================
// 数据模型
// ============================================================================

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct SalesRecord {
    id: u32,
    product: String,
    amount: f64,
    region: String,
    date: String,
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
struct CleanedData {
    total_records: usize,
    total_amount: f64,
    regions: Vec<String>,
    products: Vec<String>,
    monthly_trend: Vec<MonthlyData>,
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
struct MonthlyData {
    month: String,
    amount: f64,
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
struct AnalysisResult {
    summary: String,
    trend: String,
    top_region: String,
    growth_rate: f64,
    recommendations: Vec<String>,
}

fn generate_sample_data() -> Vec<SalesRecord> {
    vec![
        SalesRecord { id: 1, product: "产品A".into(), amount: 1500.0, region: "华东".into(), date: "2024-01".into() },
        SalesRecord { id: 2, product: "产品B".into(), amount: 2300.0, region: "华北".into(), date: "2024-01".into() },
        SalesRecord { id: 3, product: "产品A".into(), amount: 1800.0, region: "华东".into(), date: "2024-02".into() },
        SalesRecord { id: 4, product: "产品C".into(), amount: 950.0, region: "华南".into(), date: "2024-02".into() },
        SalesRecord { id: 5, product: "产品B".into(), amount: 2800.0, region: "华东".into(), date: "2024-03".into() },
        SalesRecord { id: 6, product: "产品A".into(), amount: 2100.0, region: "华北".into(), date: "2024-03".into() },
        SalesRecord { id: 7, product: "产品C".into(), amount: 1200.0, region: "华南".into(), date: "2024-03".into() },
        SalesRecord { id: 8, product: "产品B".into(), amount: 3100.0, region: "华东".into(), date: "2024-04".into() },
        SalesRecord { id: 9, product: "产品A".into(), amount: 2400.0, region: "华北".into(), date: "2024-04".into() },
        SalesRecord { id: 10, product: "产品C".into(), amount: 1500.0, region: "华南".into(), date: "2024-04".into() },
    ]
}

fn clean_data(records: &[SalesRecord]) -> CleanedData {
    let total_amount: f64 = records.iter().map(|r| r.amount).sum();
    let regions: Vec<String> = records.iter().map(|r| r.region.clone()).collect::<std::collections::HashSet<_>>().into_iter().collect();
    let products: Vec<String> = records.iter().map(|r| r.product.clone()).collect::<std::collections::HashSet<_>>().into_iter().collect();
    let mut monthly: std::collections::HashMap<String, f64> = std::collections::HashMap::new();
    for r in records {
        *monthly.entry(r.date.clone()).or_insert(0.0) += r.amount;
    }
    let monthly_trend: Vec<MonthlyData> = monthly.into_iter().map(|(month, amount)| MonthlyData { month, amount }).collect();
    CleanedData { total_records: records.len(), total_amount, regions, products, monthly_trend }
}

fn validate_data(records: &[SalesRecord]) -> bool {
    !records.is_empty() && records.iter().all(|r| r.amount > 0.0 && !r.product.is_empty())
}

fn analyze_data(cleaned: &CleanedData) -> AnalysisResult {
    let top_region = cleaned.regions.first().cloned().unwrap_or_else(|| "未知".into());
    let growth_rate = if cleaned.monthly_trend.len() >= 2 {
        let first = cleaned.monthly_trend.first().map(|m| m.amount).unwrap_or(1.0);
        let last = cleaned.monthly_trend.last().map(|m| m.amount).unwrap_or(1.0);
        (last - first) / first
    } else { 0.0 };
    AnalysisResult {
        summary: format!("共 {} 条记录，总金额 {:.2}，覆盖 {} 个区域", cleaned.total_records, cleaned.total_amount, cleaned.regions.len()),
        trend: if growth_rate > 0.0 { "上升".into() } else { "下降".into() },
        top_region,
        growth_rate,
        recommendations: vec!["加大华东区投入".into(), "关注产品B库存".into(), "拓展华南市场".into()],
    }
}

fn generate_report(analysis: &AnalysisResult, source: &str) -> serde_json::Value {
    serde_json::json!({
        "title": format!("数据分析报告 - {} ({})", chrono::Utc::now().format("%Y-%m-%d"), source),
        "generated_at": chrono::Utc::now().to_rfc3339(),
        "data_summary": analysis.summary,
        "trend": analysis.trend,
        "top_region": analysis.top_region,
        "growth_rate": analysis.growth_rate,
    })
}

fn generate_visual(analysis: &AnalysisResult) -> String {
    format!("graph TD\n  A[数据源] --> B[分析引擎]\n  B --> C{{趋势: {}}}\n  B --> D{{区域: {}}}\n  B --> E{{增长: {:.1}%}}",
        analysis.trend, analysis.top_region, analysis.growth_rate * 100.0)
}
