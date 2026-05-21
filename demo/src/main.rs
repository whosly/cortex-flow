//! # CortexFlow Demo
//!
//! 支持两种运行模式：
//! - **CLI 模式**：`cargo run -- <command>`，终端输出详细演示
//! - **Web 模式**：`cargo run`（无参数），启动 Web UI

mod cli;
mod handlers;
mod models;
mod pipeline;
mod state;

use axum::{routing::get, Router};
use state::AppState;
use std::sync::Arc;
use tower_http::cors::CorsLayer;
use tower_http::services::ServeDir;

const BANNER: &str = r#"
========================================================
  CortexFlow 数据分析管道 Demo
========================================================"#;

const USAGE: &str = r#"
用法:
  cargo run                        启动 Web UI (http://localhost:3000)
  cargo run -- <command>           运行 CLI 演示

CLI 命令:
  basic            基础 DAG 编排（6节点，并行+依赖）
  llm              LLM 集成（MockLLMClient）
  error-recovery   重试策略、重试耗尽、DAG 节点失败
  context          上下文读写、日志、快照、DAG 数据传递
  progress         进度回调、进度条可视化
  multi-model      多模型 LLM 配置（GPT-4/Llama3/豆包）与切换
  all              运行所有演示（不含 multi-model）
"#;

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().collect();

    // 无参数 → Web 模式
    if args.len() == 1 {
        run_web_server().await;
        return;
    }

    let command = &args[1];

    match command.as_str() {
        "basic" | "llm" | "error-recovery" | "context" | "progress" | "multi-model" => {
            println!("{}", BANNER);
            cli::run_demo(command).await;
        }
        "all" => {
            println!("{}", BANNER);
            for cmd in &["basic", "llm", "error-recovery", "context", "progress"] {
                cli::run_demo(cmd).await;
                println!();
            }
        }
        "-h" | "--help" => {
            println!("{}", BANNER);
            println!("{}", USAGE);
        }
        _ => {
            eprintln!("未知命令: {}", command);
            println!("{}", USAGE);
            std::process::exit(1);
        }
    }
}

async fn run_web_server() {
    tracing_subscriber::fmt()
        .with_env_filter("cortex_flow_demo=info,tower_http=info")
        .init();

    let state = Arc::new(AppState::new());
    let app = Router::new()
        .route("/api/pipelines", get(handlers::list_pipelines))
        .route("/api/pipelines/run", get(handlers::run_pipeline_sse))
        .route("/api/pipelines/result", get(handlers::get_last_result))
        .route("/api/dag/:name", get(handlers::get_dag_mermaid))
        .route("/api/llm/models", get(handlers::list_llm_models))
        .route("/api/tokens", get(handlers::get_token_stats))
        // 自适应路径
        .nest_service("/", ServeDir::new(find_static_dir()).append_index_html_on_directories(true))
        .layer(CorsLayer::permissive())
        .with_state(state);

    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], 3000));
    println!("{}", BANNER);
    println!("\n🚀 Web UI 启动: http://{}", addr);
    println!("   按 Ctrl+C 停止服务\n");

/// 自动查找 static 目录，兼容从项目根目录或 demo 目录启动
fn find_static_dir() -> &'static str {
    if std::path::Path::new("static/index.html").exists() {
        "static"
    } else if std::path::Path::new("demo/static/index.html").exists() {
        "demo/static"
    } else {
        // fallback: 尝试相对于 CARGO_MANIFEST_DIR
        eprintln!("⚠️  未找到 static 目录，请确保从项目根目录或 demo 目录运行");
        "static"
    }
}

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
