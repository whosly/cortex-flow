//! LLMClient 和 TokenTracker 集成测试
//!
//! 使用 `tests/fixtures/llm/` 下的 JSON fixture 文件作为测试数据。

use crate::common::fixture_loader;
use cortex_flow::llm::{LLMClientTrait, LLMConfig, MockLLMClient, TokenTracker, TokenUsage};

// ---- MockLLMClient tests ----

/// MockLLMClient 成功调用，从 fixture 加载预设响应并验证返回内容
#[tokio::test]
async fn test_mock_llm_client_success() {
    let responses = fixture_loader::load_json_fixture::<fixture_loader::MockResponsesFixture>(
        "llm/mock_responses.json",
    );
    let greeting = responses
        .responses
        .iter()
        .find(|r| r.name == "simple_greeting")
        .expect("simple_greeting fixture not found");

    let client = MockLLMClient::new(&greeting.content);
    let messages = vec![cortex_flow::llm::Message::user("Hi")];
    let response = client.chat(messages).await.unwrap();
    assert_eq!(response.content, greeting.content);
}

/// MockLLMClient::failing() 应始终返回错误
#[tokio::test]
async fn test_mock_llm_client_failure() {
    let client = MockLLMClient::failing();
    let messages = vec![cortex_flow::llm::Message::user("Hi")];
    let result = client.chat(messages).await;
    assert!(result.is_err());
}

// ---- TokenTracker tests (using fixture scenarios) ----

/// Fixture 驱动：从 token_scenarios.json 验证多场景 Token 追踪（累计/计数/成本）
#[test]
fn test_token_tracker_from_fixture() {
    let scenarios = fixture_loader::load_json_fixture::<fixture_loader::TokenScenariosFixture>(
        "llm/token_scenarios.json",
    );

    for scenario in &scenarios.scenarios {
        let tracker = match (scenario.input_price_per_m, scenario.output_price_per_m) {
            (Some(inp), Some(out)) => TokenTracker::with_pricing(inp, out),
            _ => TokenTracker::new(),
        };

        for record in &scenario.records {
            tracker.record(&TokenUsage::new(
                record.prompt_tokens,
                record.completion_tokens,
            ));
        }

        if let Some(expected) = scenario.expected.total_prompt_tokens {
            assert_eq!(
                tracker.total_prompt_tokens(),
                expected,
                "{}: prompt tokens",
                scenario.name
            );
        }
        if let Some(expected) = scenario.expected.total_completion_tokens {
            assert_eq!(
                tracker.total_completion_tokens(),
                expected,
                "{}: completion tokens",
                scenario.name
            );
        }
        if let Some(expected) = scenario.expected.total_tokens {
            assert_eq!(
                tracker.total_tokens(),
                expected,
                "{}: total tokens",
                scenario.name
            );
        }
        if let Some(expected) = scenario.expected.call_count {
            assert_eq!(
                tracker.call_count(),
                expected,
                "{}: call count",
                scenario.name
            );
        }
        if let Some(_expected) = scenario.expected.total_cost {
            assert!(tracker.total_cost() > 0.0, "{}: cost > 0", scenario.name);
        }
    }
}

/// TokenTracker 快照功能，验证 snapshot 捕获当前状态
#[test]
fn test_token_tracker_snapshot() {
    let tracker = TokenTracker::new();
    tracker.record(&TokenUsage::new(100, 50));
    let snap = tracker.snapshot();
    assert_eq!(snap.total_prompt_tokens, 100);
    assert_eq!(snap.call_count, 1);
}

/// TokenTracker 重置功能，验证 reset 后计数归零
#[test]
fn test_token_tracker_reset() {
    let tracker = TokenTracker::new();
    tracker.record(&TokenUsage::new(100, 50));
    tracker.reset();
    assert_eq!(tracker.call_count(), 0);
    assert_eq!(tracker.total_tokens(), 0);
}

// ---- LLMConfig tests (using fixture scenarios) ----

/// Fixture 驱动：从 llm_configs.json 验证 OpenAI 配置创建与模型名
#[test]
fn test_llm_configs_from_fixture() {
    let configs = fixture_loader::load_json_fixture::<fixture_loader::LLMConfigsFixture>(
        "llm/llm_configs.json",
    );

    let openai = configs
        .scenarios
        .iter()
        .find(|s| s.name == "openai_gpt4")
        .expect("openai_gpt4 fixture not found");

    let config = LLMConfig::openai(&openai.api_key, &openai.model);
    assert_eq!(config.model, "gpt-4");
}

/// Fixture 驱动：从 llm_configs.json 验证 Ollama 配置创建与 base_url
#[test]
fn test_llm_config_ollama_from_fixture() {
    let configs = fixture_loader::load_json_fixture::<fixture_loader::LLMConfigsFixture>(
        "llm/llm_configs.json",
    );

    let ollama = configs
        .scenarios
        .iter()
        .find(|s| s.name == "ollama_llama2")
        .expect("ollama_llama2 fixture not found");

    let config = LLMConfig::ollama(&ollama.model);
    assert_eq!(config.model, "llama2");
    assert!(config.base_url.is_some());
}

/// Fixture 驱动：从 llm_configs.json 验证 Builder 模式配置（max_tokens/temperature/timeout/retries）
#[test]
fn test_llm_config_builder_from_fixture() {
    let configs = fixture_loader::load_json_fixture::<fixture_loader::LLMConfigsFixture>(
        "llm/llm_configs.json",
    );

    let openai = configs
        .scenarios
        .iter()
        .find(|s| s.name == "openai_gpt4")
        .expect("openai_gpt4 fixture not found");

    let config = LLMConfig::openai(&openai.api_key, &openai.model)
        .with_max_tokens(openai.max_tokens.unwrap_or(0))
        .with_temperature(openai.temperature.unwrap_or(0.0))
        .with_timeout(openai.timeout_secs.unwrap_or(60))
        .with_max_retries(openai.max_retries.unwrap_or(3));
    assert_eq!(config.max_tokens, Some(1000));
    assert_eq!(config.temperature, Some(0.7));
    assert_eq!(config.timeout(), 120);
    assert_eq!(config.retries(), 5);
}
