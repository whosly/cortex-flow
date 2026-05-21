//! 数据模型

use serde::{Deserialize, Serialize};

/// 管道定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineSpec {
    pub id: String,
    pub name: String,
    pub description: String,
    pub nodes: Vec<NodeSpec>,
    pub edges: Vec<(String, String)>,
    pub dag_mermaid: String,
}

/// 节点定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeSpec {
    pub id: String,
    pub name: String,
    pub node_type: String, // "data" | "llm" | "transform"
}

/// 管道执行结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineResult {
    pub pipeline_id: String,
    pub success: bool,
    pub total_duration_ms: u64,
    pub node_results: Vec<NodeResult>,
}

/// 节点执行结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeResult {
    pub task_id: String,
    pub name: String,
    pub success: bool,
    pub duration_ms: u64,
    pub error: Option<String>,
}

/// LLM 模型信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMModelInfo {
    pub name: String,
    pub provider: String,
    pub model: String,
    pub is_default: bool,
}

/// Token 统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenStats {
    pub call_count: u64,
    pub total_tokens: u64,
    pub prompt_tokens: u64,
    pub completion_tokens: u64,
    pub total_cost: f64,
}
