//! # LLM客户端
//!
//! 提供大语言模型调用能力，支持 OpenAI 兼容 API。
//!
//! ## 模块概述
//!
//! - [`LLMClientTrait`] - LLM客户端抽象接口
//! - [`LLMClient`] - 默认客户端实现（基于 async-openai）
//! - [`MockLLMClient`] - Mock 客户端（用于测试）
//!
//! ## 使用示例
//!
//! ```rust,ignore
//! use cortex_flow::llm::{LLMClient, LLMConfig, Message};
//!
//! let config = LLMConfig::openai("sk-xxx", "gpt-4");
//! let client = LLMClient::new(config);
//!
//! let messages = vec![Message::user("Hello!")];
//! let response = client.chat(messages).await?;
//! println!("{}", response.content);
//! ```

use super::{LLMConfig, LLMResponse, Message, TokenUsage};
use crate::error::{Error, Result};
use async_trait::async_trait;
use std::sync::Arc;

/// LLM客户端抽象接口
#[async_trait]
pub trait LLMClientTrait: Send + Sync {
    /// 发送聊天请求
    async fn chat(&self, messages: Vec<Message>) -> Result<LLMResponse>;

    /// 发送带配置的聊天请求
    async fn chat_with_config(
        &self,
        messages: Vec<Message>,
        config: &LLMConfig,
    ) -> Result<LLMResponse>;

    /// 流式聊天请求
    ///
    /// 返回流式响应的 channel，调用者可以逐块接收响应内容。
    /// 如果实现不支持流式，回退到普通 chat 并一次性返回完整结果。
    async fn stream_chat(
        &self,
        messages: Vec<Message>,
    ) -> Result<tokio::sync::mpsc::Receiver<Result<StreamChunk>>> {
        // 默认实现：回退到普通 chat
        let response = self.chat(messages).await?;
        let (tx, rx) = tokio::sync::mpsc::channel(16);
        let _ = tx
            .send(Ok(StreamChunk {
                content: response.content,
                finish_reason: response.finish_reason,
                usage: Some(response.usage),
            }))
            .await;
        Ok(rx)
    }

    /// 结构化输出聊天请求
    ///
    /// 调用 LLM 并将响应解析为指定类型。
    /// 默认实现通过 JSON 反序列化完成。
    async fn chat_structured<T: serde::de::DeserializeOwned>(
        &self,
        messages: Vec<Message>,
    ) -> Result<T> {
        let response = self.chat(messages).await?;
        serde_json::from_str::<T>(&response.content)
            .map_err(|e| Error::Serialization(format!("Failed to parse structured output: {}", e)))
    }
}

/// 流式响应块
#[derive(Debug, Clone)]
pub struct StreamChunk {
    /// 本块内容
    pub content: String,
    /// 完成原因（最后一个块有值）
    pub finish_reason: Option<String>,
    /// Token 使用统计（最后一个块有值）
    pub usage: Option<TokenUsage>,
}

/// LLM 客户端实现
///
/// 当启用 `llm` feature 时使用 `async-openai` 库进行真实调用，
/// 否则回退到 Mock 实现。
#[derive(Clone)]
pub struct LLMClient {
    config: LLMConfig,
}

impl LLMClient {
    /// 创建新的 LLM 客户端
    pub fn new(config: LLMConfig) -> Self {
        Self { config }
    }

    /// 从配置创建 trait 对象
    pub fn from_config(config: LLMConfig) -> Arc<dyn LLMClientTrait> {
        Arc::new(Self { config })
    }

    /// 创建默认客户端
    pub fn default_client() -> Self {
        Self {
            config: LLMConfig::default(),
        }
    }

    /// 发送聊天请求
    pub async fn chat(&self, messages: Vec<Message>) -> Result<LLMResponse> {
        self.chat_with_config(messages, &self.config).await
    }

    /// 发送带配置的聊天请求
    pub async fn chat_with_config(
        &self,
        messages: Vec<Message>,
        config: &LLMConfig,
    ) -> Result<LLMResponse> {
        let max_retries = config.retries();
        let timeout = config.timeout();
        let mut last_error = None;

        for attempt in 0..=max_retries {
            if attempt > 0 {
                let delay = std::time::Duration::from_millis(500 * attempt as u64);
                tokio::time::sleep(delay).await;
            }

            let result = tokio::time::timeout(
                std::time::Duration::from_secs(timeout),
                self.chat_inner(messages.clone(), config),
            )
            .await;

            match result {
                Ok(Ok(response)) => return Ok(response),
                Ok(Err(e)) => {
                    if e.is_recoverable() && attempt < max_retries {
                        last_error = Some(e);
                        continue;
                    }
                    return Err(e);
                }
                Err(_) => {
                    let err = Error::Timeout(format!(
                        "LLM request timed out after {}s (attempt {}/{})",
                        timeout,
                        attempt + 1,
                        max_retries + 1
                    ));
                    if attempt < max_retries {
                        last_error = Some(err);
                        continue;
                    }
                    return Err(err);
                }
            }
        }

        Err(last_error.unwrap_or_else(|| Error::LLMCall("All retries exhausted".to_string())))
    }

    /// 内部聊天实现
    async fn chat_inner(&self, messages: Vec<Message>, config: &LLMConfig) -> Result<LLMResponse> {
        #[cfg(feature = "llm")]
        {
            self.chat_with_async_openai(messages, config).await
        }
        #[cfg(not(feature = "llm"))]
        {
            self.chat_mock(messages, config).await
        }
    }

    /// 使用 async-openai 库进行真实调用
    #[cfg(feature = "llm")]
    async fn chat_with_async_openai(
        &self,
        messages: Vec<Message>,
        config: &LLMConfig,
    ) -> Result<LLMResponse> {
        use async_openai::{
            config::OpenAIConfig,
            types::{ChatCompletionRequestMessage, CreateChatCompletionRequest},
            Client,
        };

        let openai_config = if let Some(ref base_url) = config.base_url {
            OpenAIConfig::new()
                .with_api_key(&config.api_key)
                .with_api_base(base_url)
        } else {
            OpenAIConfig::new().with_api_key(&config.api_key)
        };

        let client = Client::with_config(openai_config);

        // 转换消息格式
        let chat_messages: Vec<ChatCompletionRequestMessage> =
            messages.into_iter().map(|m| m.into()).collect();

        let mut request = CreateChatCompletionRequest {
            model: config.model.clone(),
            messages: chat_messages,
            ..Default::default()
        };

        if let Some(max_tokens) = config.max_tokens {
            request.max_tokens = Some(max_tokens);
        }
        if let Some(temperature) = config.temperature {
            request.temperature = Some(temperature);
        }

        let response = client
            .chat()
            .create(request)
            .await
            .map_err(|e| Error::LLMCall(format!("OpenAI API call failed: {}", e)))?;

        // 提取响应
        let content = response
            .choices
            .first()
            .and_then(|c| c.message.content.clone())
            .unwrap_or_default();

        let usage = response
            .usage
            .map(|u| TokenUsage::new(u.prompt_tokens, u.completion_tokens))
            .unwrap_or_else(|| TokenUsage::new(0, 0));

        Ok(LLMResponse {
            content,
            model: response.model,
            finish_reason: response
                .choices
                .first()
                .and_then(|c| c.finish_reason.clone().map(|r| r.to_string())),
            usage,
            metadata: None,
        })
    }

    /// Mock 实现（用于测试或无 async-openai 的场景）
    async fn chat_mock(&self, messages: Vec<Message>, config: &LLMConfig) -> Result<LLMResponse> {
        let usage = TokenUsage::new(
            messages.iter().map(|m| m.content.len() as u32 / 4).sum(),
            10,
        );
        Ok(LLMResponse {
            content: format!("[Mock] Response to {} messages", messages.len()),
            model: config.model.clone(),
            finish_reason: Some("stop".to_string()),
            usage,
            metadata: None,
        })
    }

    /// 获取配置
    pub fn config(&self) -> &LLMConfig {
        &self.config
    }
}

#[async_trait]
impl LLMClientTrait for LLMClient {
    async fn chat(&self, messages: Vec<Message>) -> Result<LLMResponse> {
        // 调用结构体方法，避免递归
        self.chat(messages).await
    }

    async fn chat_with_config(
        &self,
        messages: Vec<Message>,
        config: &LLMConfig,
    ) -> Result<LLMResponse> {
        self.chat_with_config(messages, config).await
    }
}

/// Mock LLM 客户端（用于测试）
pub struct MockLLMClient {
    /// 固定响应内容
    response_content: String,
    /// 是否模拟失败
    should_fail: bool,
}

impl MockLLMClient {
    /// 创建成功的 Mock 客户端
    pub fn new(response_content: impl Into<String>) -> Self {
        Self {
            response_content: response_content.into(),
            should_fail: false,
        }
    }

    /// 创建会失败的 Mock 客户端
    pub fn failing() -> Self {
        Self {
            response_content: String::new(),
            should_fail: true,
        }
    }

    /// 转换为 trait 对象
    pub fn into_trait(self) -> Arc<dyn LLMClientTrait> {
        Arc::new(self)
    }
}

#[async_trait]
impl LLMClientTrait for MockLLMClient {
    async fn chat(&self, _messages: Vec<Message>) -> Result<LLMResponse> {
        if self.should_fail {
            return Err(Error::LLMCall("Mock LLM client error".to_string()));
        }
        Ok(LLMResponse {
            content: self.response_content.clone(),
            model: "mock-model".to_string(),
            finish_reason: Some("stop".to_string()),
            usage: TokenUsage::new(10, 5),
            metadata: None,
        })
    }

    async fn chat_with_config(
        &self,
        messages: Vec<Message>,
        _config: &LLMConfig,
    ) -> Result<LLMResponse> {
        self.chat(messages).await
    }
}

/// Message 到 async-openai 格式的转换（仅 llm feature 启用时可用）
#[cfg(feature = "llm")]
impl From<Message> for async_openai::types::ChatCompletionRequestMessage {
    fn from(msg: Message) -> Self {
        use async_openai::types::ChatCompletionRequestMessage;

        match msg.role {
            crate::llm::message::Role::System => ChatCompletionRequestMessage::System(
                async_openai::types::ChatCompletionRequestSystemMessage {
                    content: msg.content,
                    ..Default::default()
                },
            ),
            crate::llm::message::Role::User => ChatCompletionRequestMessage::User(
                async_openai::types::ChatCompletionRequestUserMessage {
                    content: async_openai::types::ChatCompletionRequestUserMessageContent::Text(
                        msg.content,
                    ),
                    ..Default::default()
                },
            ),
            crate::llm::message::Role::Assistant => ChatCompletionRequestMessage::Assistant(
                async_openai::types::ChatCompletionRequestAssistantMessage {
                    content: Some(msg.content),
                    ..Default::default()
                },
            ),
            crate::llm::message::Role::Tool => ChatCompletionRequestMessage::Tool(
                async_openai::types::ChatCompletionRequestToolMessage {
                    content: async_openai::types::ChatCompletionRequestToolMessageContent::Text(
                        msg.content,
                    ),
                    tool_call_id: msg.tool_call_id.unwrap_or_default(),
                },
            ),
        }
    }
}
