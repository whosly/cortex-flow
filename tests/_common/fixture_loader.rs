//! # 测试 Fixture 加载器
//!
//! 提供统一的 fixture 文件加载工具，所有集成测试模块共享。
//! Fixture 文件位于 `tests/fixtures/` 目录下，以 JSON 格式存储。

#![allow(dead_code)]

use std::path::PathBuf;

/// 获取 fixtures 目录的根路径
///
/// 从测试二进制的工作目录向上查找 `tests/fixtures` 目录。
pub fn fixtures_dir() -> PathBuf {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
    PathBuf::from(manifest_dir).join("tests").join("fixtures")
}

/// 加载 fixture 文件内容为字符串
///
/// # 参数
/// - `relative_path`: 相对于 `tests/fixtures/` 的路径，如 `"config/valid_config.json"`
///
/// # Panics
/// 如果文件不存在或读取失败则 panic。
pub fn load_fixture(relative_path: &str) -> String {
    let path = fixtures_dir().join(relative_path);
    std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("Failed to load fixture '{}': {}", path.display(), e))
}

/// 加载并解析 JSON fixture 为指定类型
///
/// # 类型参数
/// - `T`: 反序列化目标类型，需实现 `serde::de::DeserializeOwned`
///
/// # 参数
/// - `relative_path`: 相对于 `tests/fixtures/` 的路径
///
/// # Panics
/// 如果文件不存在或 JSON 解析失败则 panic。
pub fn load_json_fixture<T: serde::de::DeserializeOwned>(relative_path: &str) -> T {
    let content = load_fixture(relative_path);
    serde_json::from_str::<T>(&content)
        .unwrap_or_else(|e| panic!("Failed to parse JSON fixture '{}': {}", relative_path, e))
}

/// 加载并解析 TOML fixture 为指定类型
///
/// # 类型参数
/// - `T`: 反序列化目标类型，需实现 `serde::de::DeserializeOwned`
///
/// # 参数
/// - `relative_path`: 相对于 `tests/fixtures/` 的路径
pub fn load_toml_fixture<T: serde::de::DeserializeOwned>(relative_path: &str) -> T {
    let content = load_fixture(relative_path);
    toml::from_str::<T>(&content)
        .unwrap_or_else(|e| panic!("Failed to parse TOML fixture '{}': {}", relative_path, e))
}

// ============================================================
// DAG Fixture 类型定义
// ============================================================

/// DAG fixture 定义（对应 `tests/fixtures/dag/*.json`）
#[derive(Debug, serde::Deserialize)]
pub struct DAGFixture {
    pub nodes: Vec<DAGNodeDef>,
    pub edges: Vec<DAGEdgeDef>,
    pub expected: DAGExpected,
}

/// DAG 节点定义
#[derive(Debug, serde::Deserialize)]
pub struct DAGNodeDef {
    pub id: String,
    pub name: String,
}

/// DAG 边定义
#[derive(Debug, serde::Deserialize)]
pub struct DAGEdgeDef {
    pub from: String,
    pub to: String,
    #[serde(default)]
    pub label: Option<String>,
}

/// DAG 预期结果
#[derive(Debug, serde::Deserialize)]
pub struct DAGExpected {
    pub node_count: usize,
    pub edge_count: usize,
}

// ============================================================
// LLM Fixture 类型定义
// ============================================================

/// LLM 配置 fixture（对应 `tests/fixtures/llm/llm_configs.json`）
#[derive(Debug, serde::Deserialize)]
pub struct LLMConfigsFixture {
    pub scenarios: Vec<LLMConfigScenario>,
}

/// LLM 配置场景
#[derive(Debug, serde::Deserialize)]
pub struct LLMConfigScenario {
    pub name: String,
    pub provider: String,
    pub api_key: String,
    pub model: String,
    #[serde(default)]
    pub base_url: Option<String>,
    pub max_tokens: Option<u32>,
    pub temperature: Option<f32>,
    pub timeout_secs: Option<u64>,
    pub max_retries: Option<u32>,
}

/// Mock LLM 响应 fixture（对应 `tests/fixtures/llm/mock_responses.json`）
#[derive(Debug, serde::Deserialize)]
pub struct MockResponsesFixture {
    pub responses: Vec<MockResponseDef>,
}

/// Mock 响应定义
#[derive(Debug, serde::Deserialize)]
pub struct MockResponseDef {
    pub name: String,
    pub content: String,
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
}

/// Token 场景 fixture（对应 `tests/fixtures/llm/token_scenarios.json`）
#[derive(Debug, serde::Deserialize)]
pub struct TokenScenariosFixture {
    pub scenarios: Vec<TokenScenario>,
}

/// Token 追踪场景
#[derive(Debug, serde::Deserialize)]
pub struct TokenScenario {
    pub name: String,
    #[serde(default)]
    pub input_price_per_m: Option<f64>,
    #[serde(default)]
    pub output_price_per_m: Option<f64>,
    pub records: Vec<TokenRecordDef>,
    pub expected: TokenExpected,
}

/// Token 记录定义
#[derive(Debug, serde::Deserialize)]
pub struct TokenRecordDef {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
}

/// Token 预期结果
#[derive(Debug, serde::Deserialize)]
pub struct TokenExpected {
    #[serde(default)]
    pub total_prompt_tokens: Option<u64>,
    #[serde(default)]
    pub total_completion_tokens: Option<u64>,
    #[serde(default)]
    pub total_tokens: Option<u64>,
    #[serde(default)]
    pub call_count: Option<u64>,
    #[serde(default)]
    pub total_cost: Option<f64>,
}

// ============================================================
// Task Fixture 类型定义
// ============================================================

/// 任务定义 fixture（对应 `tests/fixtures/task/task_definitions.json`）
#[derive(Debug, serde::Deserialize)]
pub struct TaskDefinitionsFixture {
    pub definitions: Vec<TaskDefinition>,
}

/// 任务定义
#[derive(Debug, serde::Deserialize)]
pub struct TaskDefinition {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
}
