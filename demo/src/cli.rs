//! CLI 模式演示：终端输出详细执行过程

use cortex_flow::llm::{LLMClientRegistry, LLMConfig, MockLLMClient};
use cortex_flow::task::SimpleTask;
use cortex_flow::Orchestrator;

pub async fn run_demo(command: &str) {
    match command {
        "basic" => demo_basic().await,
        "llm" => demo_llm().await,
        "error-recovery" => demo_error_recovery().await,
        "context" => demo_context().await,
        "progress" => demo_progress().await,
        "multi-model" => demo_multi_model().await,
        _ => eprintln!("未知演示: {}", command),
    }
}

// ============================================================================
// Demo 1: 基础 DAG 数据管道
// ============================================================================

async fn demo_basic() {
    println!("\n--- Demo 1: 基础 DAG 数据管道 ---\n");
    println!("DAG 结构:");
    println!("  [数据获取] ──┬──> [数据清洗] ──┬──> [数据分析] ──┬──> [报告生成]");
    println!("               └──> [数据验证] ──┘                  └──> [可视化导出]");
    println!();

    let orchestrator = Orchestrator::builder()
        .with_max_parallelism(4)
        .build()
        .await
        .unwrap();

    let result = orchestrator
        .execute_dag(|dag| {
            dag.add_node(SimpleTask::new("fetch_data", "获取销售数据", |_input, _ctx| {
                Box::pin(async move {
                    let records = generate_sample_data();
                    println!("  [fetch_data] 获取到 {} 条销售记录", records.len());
                    Ok(serde_json::to_value(records).unwrap_or_default())
                })
            }));
            dag.add_node(SimpleTask::new("clean_data", "清洗数据", |input, _ctx| {
                Box::pin(async move {
                    let records: Vec<SalesRecord> = serde_json::from_value(input).unwrap_or_default();
                    let cleaned = clean_data(&records);
                    println!("  [clean_data] 清洗完成: {} 条记录, {} 个区域", cleaned.total_records, cleaned.regions.len());
                    Ok(serde_json::to_value(&cleaned).unwrap_or_default())
                })
            }));
            dag.add_node(SimpleTask::new("validate_data", "验证数据", |input, _ctx| {
                Box::pin(async move {
                    let records: Vec<SalesRecord> = serde_json::from_value(input).unwrap_or_default();
                    let is_valid = validate_data(&records);
                    println!("  [validate_data] 数据验证: {}", if is_valid { "通过" } else { "失败" });
                    Ok(serde_json::json!({ "is_valid": is_valid, "record_count": records.len() }))
                })
            }));
            dag.add_node(SimpleTask::new("analyze_data", "分析数据", |input, _ctx| {
                Box::pin(async move {
                    let cleaned: CleanedData = if let Some(v) = input.get("clean_data") {
                        serde_json::from_value(v.clone()).unwrap_or_default()
                    } else {
                        serde_json::from_value(input).unwrap_or_default()
                    };
                    let analysis = analyze_data(&cleaned);
                    println!("  [analyze_data] 分析完成: 趋势={}, 增长率={:.1}%", analysis.trend, analysis.growth_rate * 100.0);
                    Ok(serde_json::to_value(&analysis).unwrap_or_default())
                })
            }));
            dag.add_node(SimpleTask::new("generate_report", "生成报告", |input, _ctx| {
                Box::pin(async move {
                    let analysis: AnalysisResult = serde_json::from_value(input).unwrap_or_default();
                    let report = generate_report(&analysis, "基础模式");
                    let title = report.get("title").and_then(|t| t.as_str()).unwrap_or("");
                    println!("  [generate_report] 报告已生成: {}", title);
                    Ok(serde_json::to_value(&report).unwrap_or_default())
                })
            }));
            dag.add_node(SimpleTask::new("export_visual", "可视化导出", |input, _ctx| {
                Box::pin(async move {
                    let analysis: AnalysisResult = serde_json::from_value(input).unwrap_or_default();
                    let chart = generate_visual(&analysis);
                    println!("  [export_visual] 可视化图表已导出 ({} 字符)", chart.len());
                    Ok(serde_json::json!({ "chart": chart, "format": "mermaid" }))
                })
            }));

            dag.add_dependency("fetch_data", "clean_data");
            dag.add_dependency("fetch_data", "validate_data");
            dag.add_dependency("clean_data", "analyze_data");
            dag.add_dependency("validate_data", "analyze_data");
            dag.add_dependency("analyze_data", "generate_report");
            dag.add_dependency("analyze_data", "export_visual");
        })
        .await
        .unwrap();

    print_execution_result(&result);
}

// ============================================================================
// Demo 2: 带 LLM 分析的管道
// ============================================================================

async fn demo_llm() {
    println!("\n--- Demo 2: 带 LLM 分析的数据管道 ---\n");

    let mock = MockLLMClient::new(
        r#"{"summary":"销售数据整体呈上升趋势","trend":"上升","top_region":"华东","growth_rate":0.15,"recommendations":["加大华东区投入","关注产品B库存","拓展华南市场"]}"#,
    );
    let orchestrator = Orchestrator::builder()
        .with_max_parallelism(4)
        .with_llm_config(LLMConfig::ollama("demo-model"))
        .build()
        .await
        .unwrap()
        .with_llm_client(mock.into_trait());

    let result = orchestrator
        .execute_dag(|dag| {
            dag.add_node(SimpleTask::new("fetch_data", "获取销售数据", |_input, _ctx| {
                Box::pin(async move {
                    let records = generate_sample_data();
                    println!("  [fetch_data] 获取到 {} 条销售记录", records.len());
                    Ok(serde_json::to_value(records).unwrap_or_default())
                })
            }));
            dag.add_node(SimpleTask::new("clean_data", "清洗数据", |input, _ctx| {
                Box::pin(async move {
                    let records: Vec<SalesRecord> = serde_json::from_value(input).unwrap_or_default();
                    let cleaned = clean_data(&records);
                    println!("  [clean_data] 清洗完成: {} 条记录, 总额 {:.2}", cleaned.total_records, cleaned.total_amount);
                    Ok(serde_json::to_value(&cleaned).unwrap_or_default())
                })
            }));
            dag.add_node(SimpleTask::new("llm_analyze", "LLM 智能分析", |input, _ctx| {
                Box::pin(async move {
                    let cleaned: CleanedData = serde_json::from_value(input).unwrap_or_default();
                    println!("  [llm_analyze] 发送数据摘要到 LLM: {} 条记录, 总金额 {:.2}", cleaned.total_records, cleaned.total_amount);
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
                    let analysis: AnalysisResult = serde_json::from_value(input).unwrap_or_default();
                    let report = generate_report(&analysis, "LLM 智能分析");
                    let title = report.get("title").and_then(|t| t.as_str()).unwrap_or("");
                    println!("  [generate_report] 智能报告已生成: {}", title);
                    Ok(serde_json::to_value(&report).unwrap_or_default())
                })
            }));

            dag.add_dependency("fetch_data", "clean_data");
            dag.add_dependency("clean_data", "llm_analyze");
            dag.add_dependency("llm_analyze", "generate_report");
        })
        .await
        .unwrap();

    print_execution_result(&result);
}

// ============================================================================
// Demo 3: 错误恢复与重试
// ============================================================================

async fn demo_error_recovery() {
    println!("\n--- Demo 3: 错误恢复与重试 ---\n");

    // 场景1: 指数退避重试
    println!("场景1: 指数退避重试（模拟第2次成功）");
    let mut recovery = cortex_flow::ErrorRecovery::new(
        cortex_flow::RetryPolicy::exponential(3, 100, 2.0),
    );
    let attempt_count = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let count_clone = attempt_count.clone();
    let result = recovery
        .execute_with_retry(|| {
            let c = count_clone.clone();
            async move {
                let n = c.fetch_add(1, std::sync::atomic::Ordering::SeqCst) + 1;
                println!("  第 {} 次尝试: {}", n, if n == 1 { "失败" } else { "成功!" });
                if n == 1 {
                    Err(cortex_flow::Error::TaskExecution("临时错误".into()))
                } else {
                    Ok("数据获取成功".to_string())
                }
            }
        })
        .await;
    println!("  重试结果: {:?}", result);

    // 场景2: 重试耗尽
    println!("\n场景2: 重试耗尽（始终失败）");
    let mut recovery2 = cortex_flow::ErrorRecovery::new(
        cortex_flow::RetryPolicy::exponential(3, 50, 2.0),
    );
    let attempt_count2 = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let count_clone2 = attempt_count2.clone();
    let result2: Result<String, cortex_flow::Error> = recovery2
        .execute_with_retry(|| {
            let c = count_clone2.clone();
            async move {
                let n = c.fetch_add(1, std::sync::atomic::Ordering::SeqCst) + 1;
                println!("  第 {} 次尝试: 失败", n);
                Err(cortex_flow::Error::LLMCall("API rate limit".into()))
            }
        })
        .await;
    println!("  重试结果: {:?}", result2);

    // 场景3: DAG 中某个节点失败
    println!("\n场景3: DAG 中某个节点失败");
    let orchestrator = Orchestrator::builder()
        .with_max_parallelism(4)
        .build()
        .await
        .unwrap();

    let result = orchestrator
        .execute_dag(|dag| {
            dag.add_node(SimpleTask::new("step1", "正常步骤", |_input, _ctx| {
                Box::pin(async move {
                    println!("  [step1] 执行成功");
                    Ok(serde_json::json!({ "data": "无错误" }))
                })
            }));
            dag.add_node(SimpleTask::new("step2", "失败步骤", |_input, _ctx| {
                Box::pin(async move {
                    println!("  [step2] 执行失败!");
                    Err(cortex_flow::Error::TaskExecution("数据格式错误".into()))
                })
            }));
            dag.add_node(SimpleTask::new("step3", "依赖步骤", |_input, _ctx| {
                Box::pin(async move {
                    println!("  [step3] 执行成功");
                    Ok(serde_json::json!({ "data": "无错误" }))
                })
            }));
            dag.add_dependency("step1", "step2");
            dag.add_dependency("step2", "step3");
        })
        .await
        .unwrap();

    println!("  DAG 执行结果: success={}", result.success);
    for node in &result.node_results {
        let status = if node.success { "成功" } else { "失败" };
        let error = node.error.as_deref().unwrap_or("-");
        println!("    节点 {}: {} - \"{}\"", node.task_id, status, error);
    }
}

// ============================================================================
// Demo 4: 执行上下文
// ============================================================================

async fn demo_context() {
    println!("\n--- Demo 4: 执行上下文与数据传递 ---\n");

    let mut ctx = cortex_flow::ExecutionContext::new();
    println!("Session ID: {}", ctx.session_id());

    // 读写操作
    ctx.set("pipeline_name", "数据分析管道").unwrap();
    ctx.set("config", serde_json::json!({"max_retries": 3, "threshold": 0.8})).unwrap();
    ctx.set("batch_size", 100).unwrap();
    println!("设置上下文数据:");
    let name: Option<String> = ctx.get("pipeline_name");
    let config: Option<serde_json::Value> = ctx.get("config");
    let batch: Option<i32> = ctx.get("batch_size");
    println!("  pipeline_name = {}", name.unwrap_or_default());
    println!("  config = {}", config.unwrap_or_default());
    println!("  batch_size = {}", batch.unwrap_or_default());

    // 日志
    ctx.log("info", "管道启动");
    ctx.log_with_task("debug", "数据获取完成", "fetch_data");
    ctx.log_with_task("warn", "数据质量较低", "validate_data");
    println!("\n上下文日志:");
    for log in ctx.logs() {
        let task_info = log.task_id.as_deref().map(|t| format!(" [task: {}]", t)).unwrap_or_default();
        println!("  [{}] {}{}", log.level, log.message, task_info);
    }

    // 快照与恢复
    let snapshot = ctx.snapshot();
    ctx.set("temp_data", "临时值").unwrap();
    println!("\n添加临时数据后: {} 个键", ctx.len());
    ctx.restore(&snapshot);
    println!("恢复快照后: {} 个键", ctx.len());
    println!("临时数据是否存在: {}", ctx.contains("temp_data"));

    // DAG 中的数据传递
    println!("\n--- DAG 中的数据传递 ---");
    let orchestrator = Orchestrator::builder()
        .with_max_parallelism(4)
        .build()
        .await
        .unwrap();

    let result = orchestrator
        .execute_dag(|dag| {
            dag.add_node(SimpleTask::new("producer", "数据生产者", |_input, _ctx| {
                Box::pin(async move {
                    println!("  [producer] 生成数据");
                    Ok(serde_json::json!({ "direct": "通过依赖传递", "value": 42 }))
                })
            }));
            dag.add_node(SimpleTask::new("consumer", "数据消费者", |input, _ctx| {
                Box::pin(async move {
                    let direct: Option<String> = input.get("direct").and_then(|v| v.as_str()).map(|s| s.to_string());
                    let value: Option<i64> = input.get("value").and_then(|v| v.as_i64());
                    println!("  [consumer] 依赖传递: direct={}, value={}", direct.unwrap_or_default(), value.unwrap_or_default());
                    Ok(serde_json::json!({ "consumed": true }))
                })
            }));
            dag.add_dependency("producer", "consumer");
        })
        .await
        .unwrap();

    print_execution_result(&result);
}

// ============================================================================
// Demo 5: 进度回调
// ============================================================================

async fn demo_progress() {
    println!("\n--- Demo 5: 带进度回调的管道 ---\n");

    let orchestrator = Orchestrator::builder()
        .with_max_parallelism(4)
        .build()
        .await
        .unwrap();

    let result = orchestrator
        .execute_dag_with_progress(
            |dag| {
                dag.add_node(SimpleTask::new("fetch", "获取数据", |_input, _ctx| {
                    Box::pin(async move {
                        tokio::time::sleep(std::time::Duration::from_millis(200)).await;
                        println!("  [fetch] 完成");
                        Ok(serde_json::json!({ "data": "ok" }))
                    })
                }));
                dag.add_node(SimpleTask::new("validate", "验证数据", |_input, _ctx| {
                    Box::pin(async move {
                        tokio::time::sleep(std::time::Duration::from_millis(150)).await;
                        println!("  [validate] 完成");
                        Ok(serde_json::json!({ "valid": true }))
                    })
                }));
                dag.add_node(SimpleTask::new("clean", "清洗数据", |_input, _ctx| {
                    Box::pin(async move {
                        tokio::time::sleep(std::time::Duration::from_millis(300)).await;
                        println!("  [clean] 完成");
                        Ok(serde_json::json!({ "cleaned": true }))
                    })
                }));
                dag.add_node(SimpleTask::new("analyze", "分析数据", |_input, _ctx| {
                    Box::pin(async move {
                        tokio::time::sleep(std::time::Duration::from_millis(400)).await;
                        println!("  [analyze] 完成");
                        Ok(serde_json::json!({ "result": "done" }))
                    })
                }));
                dag.add_node(SimpleTask::new("export", "导出结果", |_input, _ctx| {
                    Box::pin(async move {
                        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                        println!("  [export] 完成");
                        Ok(serde_json::json!({ "exported": true }))
                    })
                }));
                dag.add_node(SimpleTask::new("report", "生成报告", |_input, _ctx| {
                    Box::pin(async move {
                        tokio::time::sleep(std::time::Duration::from_millis(200)).await;
                        println!("  [report] 完成");
                        Ok(serde_json::json!({ "report": "done" }))
                    })
                }));

                dag.add_dependency("fetch", "validate");
                dag.add_dependency("fetch", "clean");
                dag.add_dependency("validate", "analyze");
                dag.add_dependency("clean", "analyze");
                dag.add_dependency("analyze", "export");
                dag.add_dependency("analyze", "report");
            },
            |progress| {
                let completed = progress.completed;
                let total = progress.total;
                let success = progress.success_count;
                let failed = progress.failure_count;
                let pct = progress.percent;
                let filled = pct / 10;
                let bar: String = "█".repeat(filled as usize) + &"░".repeat(10 - filled as usize);
                print!("\r  进度: [{}] {}% ({}/{} 成功:{} 失败:{})  ", bar, pct, completed, total, success, failed);
                use std::io::Write;
                std::io::stdout().flush().ok();
            },
        )
        .await
        .unwrap();

    println!();
    print_execution_result(&result);
}

// ============================================================================
// Demo 6: 多模型 LLM 配置
// ============================================================================

async fn demo_multi_model() {
    println!("\n--- Demo 6: 多模型 LLM 配置与切换 ---\n");

    // 1. LLMConfig::custom() 快捷构造
    println!("=== 1. LLMConfig::custom() 快捷构造 ===\n");
    let doubao_config = LLMConfig::custom("your-key", "doubao-pro-32k", "https://ark.cn-beijing.volces.com/api/v3");
    let deepseek_config = LLMConfig::custom("your-key", "deepseek-chat", "https://api.deepseek.com/v1");
    println!("[火山引擎豆包] 使用 LLMConfig::custom() 构造");
    println!("  provider: {:?}", doubao_config.provider);
    println!("  model: {}", doubao_config.model);
    println!("  base_url: {:?}", doubao_config.base_url);
    println!("\n[DeepSeek] 使用 LLMConfig::custom() 构造");
    println!("  provider: {:?}", deepseek_config.provider);
    println!("  model: {}", deepseek_config.model);

    // 2. ChatRequest 请求级参数覆盖
    println!("\n=== 2. ChatRequest 请求级参数覆盖 ===\n");
    let default_config = LLMConfig::openai("sk-xxx", "gpt-4");
    println!("默认配置 model: {}", default_config.model);
    let chat_req = cortex_flow::llm::ChatRequest::new(vec![cortex_flow::llm::Message::user("分析数据")])
        .with_model("gpt-3.5-turbo")
        .with_temperature(0.3)
        .with_max_tokens(512);
    println!("ChatRequest 覆盖: model={:?}, temperature={:?}, max_tokens={:?}",
        chat_req.model, chat_req.temperature, chat_req.max_tokens);

    // 3. Orchestrator Builder 多模型注册
    println!("\n=== 3. Orchestrator Builder 多模型注册 ===\n");
    let orchestrator = Orchestrator::builder()
        .with_max_parallelism(4)
        .with_llm("gpt4", LLMConfig::openai("sk-demo", "gpt-4"))
        .with_llm("llama3", LLMConfig::ollama("llama3"))
        .with_llm("doubao", LLMConfig::custom("demo-key", "doubao-pro-32k", "https://ark.cn-beijing.volces.com/api/v3"))
        .with_default_llm("gpt4")
        .build()
        .await
        .unwrap();

    let registry = &orchestrator.llm_registry();
    println!("已注册模型: {:?}", registry.names());
    println!("默认模型: {:?}", registry.default_name());
    println!("获取 gpt4: {}", registry.get("gpt4").is_some());
    println!("获取 doubao: {}", registry.get("doubao").is_some());

    // 4. ExecutionContext LLM 访问
    println!("\n=== 4. ExecutionContext LLM 访问（推荐方式）===\n");
    println!("DAG 结构（任务通过 ctx.get_llm() 获取模型）:");
    println!("  [数据获取] ──> [数据清洗] ──┬──> [GPT-4 深度分析]  ──┐");
    println!("                             ├──> [Llama3 本地校验]  ──┼──> [豆包报告润色] ──> [Token 统计]");
    println!("                             └──> [数据统计]         ──┘");
    println!();

    let result = orchestrator
        .execute_dag(|dag| {
            dag.add_node(SimpleTask::new("fetch_data", "获取销售数据", |_input, _ctx| {
                Box::pin(async move {
                    let records = generate_sample_data();
                    println!("  [fetch_data] 获取 {} 条销售记录", records.len());
                    Ok(serde_json::to_value(records).unwrap_or_default())
                })
            }));
            dag.add_node(SimpleTask::new("clean_data", "清洗数据", |input, _ctx| {
                Box::pin(async move {
                    let records: Vec<SalesRecord> = serde_json::from_value(input).unwrap_or_default();
                    let cleaned = clean_data(&records);
                    println!("  [clean_data] 清洗完成: {} 条记录, 总额 {:.2}", cleaned.total_records, cleaned.total_amount);
                    Ok(serde_json::to_value(&cleaned).unwrap_or_default())
                })
            }));
            dag.add_node(SimpleTask::new("gpt4_analysis", "GPT-4 深度分析", |input, ctx| {
                let _client = ctx.get_llm("gpt4");
                Box::pin(async move {
                    let _cleaned: CleanedData = serde_json::from_value(input).unwrap_or_default();
                    let _ = _client;
                    println!("  [gpt4_analysis] 使用 GPT-4 分析...");
                    Ok(serde_json::json!({ "model": "gpt-4", "insight": "深度分析: 销售趋势上升，华东区领先" }))
                })
            }));
            dag.add_node(SimpleTask::new("llama3_validate", "Llama3 本地校验", |input, ctx| {
                let _client = ctx.get_llm("llama3");
                Box::pin(async move {
                    let _cleaned: CleanedData = serde_json::from_value(input).unwrap_or_default();
                    let _ = _client;
                    println!("  [llama3_validate] 使用本地 Llama3 校验...");
                    Ok(serde_json::json!({ "model": "llama3", "validation": "数据质量良好" }))
                })
            }));
            dag.add_node(SimpleTask::new("statistics", "统计计算", |input, _ctx| {
                Box::pin(async move {
                    let cleaned: CleanedData = serde_json::from_value(input).unwrap_or_default();
                    println!("  [statistics] 计算统计指标...");
                    Ok(serde_json::json!({ "avg": cleaned.total_amount / cleaned.total_records as f64 }))
                })
            }));
            dag.add_node(SimpleTask::new("doubao_report", "豆包报告润色", |_input, ctx| {
                let _client = ctx.get_llm("doubao");
                Box::pin(async move {
                    let _ = _client;
                    println!("  [doubao_report] 使用豆包润色报告...");
                    Ok(serde_json::json!({ "model": "doubao-pro-32k", "report": "润色完成" }))
                })
            }));
            dag.add_node(SimpleTask::new("token_summary", "Token 使用统计", |_input, ctx| {
                let snapshot = ctx.token_snapshot();
                let llm_names = ctx.llm_names();
                Box::pin(async move {
                    let tokens = if let Some(s) = snapshot {
                        format!("调用次数: {}\n    总 Token: {}\n    估算成本: ${:.4}", s.call_count, s.total_tokens, s.total_cost)
                    } else {
                        "无数据".into()
                    };
                    println!("  [token_summary] Token 使用统计:\n    {}", tokens);
                    println!("  [token_summary] 可用模型: {:?}", llm_names);
                    Ok(serde_json::json!({ "token_tracking": "done" }))
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
        })
        .await
        .unwrap();

    // 5. TokenTracker 聚合统计
    println!("\n=== 5. TokenTracker 聚合统计 ===\n");
    let snapshot = registry.token_snapshot();
    println!("所有模型聚合 Token 使用:");
    println!("{}", snapshot);

    // 6. LLMClientRegistry 独立使用
    println!("\n=== 6. LLMClientRegistry 独立使用 ===\n");
    let standalone_registry = LLMClientRegistry::new();
    standalone_registry.register_with_config("gpt4", LLMConfig::openai("sk-xxx", "gpt-4"));
    standalone_registry.register_with_config("llama3", LLMConfig::ollama("llama3"));
    standalone_registry.register_with_config("doubao", LLMConfig::custom("key", "doubao-pro-32k", "https://ark.cn-beijing.volces.com/api/v3"));
    standalone_registry.set_default("gpt4");
    println!("注册表中的模型: {:?}", standalone_registry.names());
    println!("默认模型: {:?}", standalone_registry.default_name());
    println!("包含 gpt4: {}", standalone_registry.get("gpt4").is_some());
    println!("模型数量: {}", standalone_registry.names().len());

    // 7. 总结
    println!("\n=== 7. CortexFlow 多模型特性总结 ===\n");
    println!("P0: LLMConfig::custom()      - 豆包/DeepSeek 等 OpenAI 兼容 API 快捷构造");
    println!("P1: ChatRequest               - 请求级别覆盖 model/temperature/max_tokens");
    println!("P2: LLMClientRegistry         - HashMap 注册表，按名称管理多模型");
    println!("P3: Orchestrator 多模型集成    - Builder.with_llm()/with_default_llm() 链式注册");
    println!("P4: TokenTracker 自动集成     - 调用后自动记录，注册表级聚合统计");
    println!("P5: ExecutionContext LLM 访问 - ctx.get_llm()/ctx.default_llm() 零 Arc 克隆");

    println!("\n--- 推荐用法 ---");
    println!("1. Orchestrator::builder().with_llm(\"name\", config).build()");
    println!("2. DAG 任务中: ctx.get_llm(\"name\") 获取客户端");
    println!("3. 临时切换: ChatRequest::new(msg).with_model(\"other-model\")");
    println!("4. 查看成本: orchestrator.token_snapshot()");

    print_execution_result(&result);
}

// ============================================================================
// 辅助函数
// ============================================================================

fn print_execution_result(result: &cortex_flow::ExecutionResult) {
    println!("\n--- 执行结果 ---");
    println!("整体状态: {}", if result.success { "成功" } else { "失败" });
    println!("总耗时: {}ms", result.total_duration_ms);
    println!("节点结果:");
    for node in &result.node_results {
        let status = if node.success { "OK" } else { "FAIL" };
        println!("  [{}] {} ({}) - {}ms{}",
            status,
            node.task_id,
            node.name,
            node.duration_ms,
            node.error.as_deref().map(|e| format!(" - {}", e)).unwrap_or_default()
        );
    }
}

// ============================================================================
// 数据模型（与 pipeline.rs 共享逻辑）
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
