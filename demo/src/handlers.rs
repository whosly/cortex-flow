//! HTTP 请求处理器

use crate::models::{LLMModelInfo, PipelineResult, TokenStats};
use crate::pipeline;
use crate::state::AppState;
use axum::{
    extract::{Path, Query, State},
    response::sse::{Event, KeepAlive, Sse},
    Json,
};
use futures::stream::Stream;
use serde::Deserialize;
use std::convert::Infallible;
use std::sync::Arc;
use tokio_stream::StreamExt;

#[derive(Deserialize)]
pub struct RunQuery {
    pub pipeline: String,
}

/// GET /api/pipelines — 列出所有可用管道
pub async fn list_pipelines(
    State(_state): State<Arc<AppState>>,
) -> Json<Vec<crate::models::PipelineSpec>> {
    Json(pipeline::get_pipelines())
}

/// GET /api/pipelines/run?pipeline=basic — SSE 执行管道
pub async fn run_pipeline_sse(
    State(state): State<Arc<AppState>>,
    Query(query): Query<RunQuery>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let pipeline_id = query.pipeline.clone();
    let state_clone = state.clone();

    // 在后台启动管道执行
    let pipeline_id_bg = pipeline_id.clone();
    tokio::spawn(async move {
        let result = pipeline::execute_pipeline(&pipeline_id_bg, state_clone.progress_tx.clone()).await;
        *state_clone.last_result.lock().await = Some(result);
    });

    // SSE 流：监听 progress_rx
    let rx = state.progress_rx.clone();
    let stream = tokio_stream::wrappers::WatchStream::new(rx).filter_map(|opt_event| {
        match opt_event {
            Some(event) => {
                let data = serde_json::to_string(&event).unwrap_or_default();
                Some(Ok(Event::default().data(data).event("progress")))
            }
            None => None,
        }
    });

    Sse::new(stream).keep_alive(KeepAlive::default())
}

/// GET /api/pipelines/result — 获取最近一次执行结果
pub async fn get_last_result(
    State(state): State<Arc<AppState>>,
) -> Json<Option<PipelineResult>> {
    let result = state.last_result.lock().await.clone();
    Json(result)
}

/// GET /api/dag/:name — 获取 DAG 的 Mermaid 定义
pub async fn get_dag_mermaid(
    State(_state): State<Arc<AppState>>,
    Path(name): Path<String>,
) -> Json<serde_json::Value> {
    let pipelines = pipeline::get_pipelines();
    let spec = pipelines.iter().find(|p| p.id == name);
    match spec {
        Some(s) => Json(serde_json::json!({ "mermaid": s.dag_mermaid, "name": s.name })),
        None => Json(serde_json::json!({ "error": format!("Pipeline '{}' not found", name) })),
    }
}

/// GET /api/llm/models — 列出已注册的 LLM 模型
pub async fn list_llm_models(
    State(state): State<Arc<AppState>>,
) -> Json<Vec<LLMModelInfo>> {
    let registry = &state.llm_registry;
    let default_name = registry.default_name();
    let models: Vec<LLMModelInfo> = registry
        .names()
        .into_iter()
        .map(|name| {
            let is_default = default_name.as_ref() == Some(&name);
            // 获取模型配置信息的简化方式
            let (provider, model) = if name.contains("gpt") {
                ("OpenAI".into(), "gpt-4".into())
            } else if name.contains("llama") {
                ("Ollama".into(), "llama3".into())
            } else if name.contains("doubao") {
                ("Custom".into(), "doubao-pro-32k".into())
            } else {
                ("Unknown".into(), name.clone())
            };
            LLMModelInfo { name, provider, model, is_default }
        })
        .collect();
    Json(models)
}

/// GET /api/tokens — 获取 Token 使用统计
pub async fn get_token_stats(
    State(state): State<Arc<AppState>>,
) -> Json<TokenStats> {
    let snapshot = state.llm_registry.token_snapshot();
    Json(TokenStats {
        call_count: snapshot.call_count,
        total_tokens: snapshot.total_tokens,
        prompt_tokens: snapshot.total_prompt_tokens,
        completion_tokens: snapshot.total_completion_tokens,
        total_cost: snapshot.total_cost,
    })
}
