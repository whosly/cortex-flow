//! 共享应用状态

use crate::models::PipelineResult;
use cortex_flow::llm::{LLMClientRegistry, LLMConfig};
use std::sync::Arc;
use tokio::sync::{watch, Mutex};

pub struct AppState {
    pub llm_registry: Arc<LLMClientRegistry>,
    pub last_result: Arc<Mutex<Option<PipelineResult>>>,
    /// SSE 进度广播
    pub progress_tx: Arc<watch::Sender<Option<ProgressEvent>>>,
    pub progress_rx: watch::Receiver<Option<ProgressEvent>>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ProgressEvent {
    pub pipeline: String,
    pub node_id: String,
    pub node_name: String,
    pub status: String, // "started" | "completed" | "failed" | "done"
    pub percent: usize,
    pub completed: usize,
    pub total: usize,
    pub message: String,
}

impl AppState {
    pub fn new() -> Self {
        let registry = LLMClientRegistry::new();
        // 注册演示用的 Mock 模型
        registry.register_with_config("gpt4", LLMConfig::openai("sk-demo", "gpt-4"));
        registry.register_with_config("llama3", LLMConfig::ollama("llama3"));
        registry.register_with_config(
            "doubao",
            LLMConfig::custom("demo-key", "doubao-pro-32k", "https://ark.cn-beijing.volces.com/api/v3"),
        );
        registry.set_default("gpt4");

        let (progress_tx, progress_rx) = watch::channel(None);

        Self {
            llm_registry: Arc::new(registry),
            last_result: Arc::new(Mutex::new(None)),
            progress_tx: Arc::new(progress_tx),
            progress_rx,
        }
    }
}
