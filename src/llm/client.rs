//! # LLM客户端

use std::sync::Arc;
use async_trait::async_trait;
use crate::error::Result;
use super::{LLMConfig, Message, LLMResponse, TokenUsage};

#[async_trait]
pub trait LLMClientTrait: Send + Sync {
    async fn chat(&self, messages: Vec<Message>) -> Result<LLMResponse>;
    async fn chat_with_config(&self, messages: Vec<Message>, config: &LLMConfig) -> Result<LLMResponse>;
}

#[derive(Clone)]
pub struct LLMClient {
    config: LLMConfig,
}

impl LLMClient {
    pub fn new(config: LLMConfig) -> Self {
        Self { config }
    }

    pub fn from_config(config: LLMConfig) -> Arc<dyn LLMClientTrait> {
        Arc::new(Self { config })
    }

    pub fn default_client() -> Self {
        Self { config: LLMConfig::default() }
    }

    pub async fn chat(&self, messages: Vec<Message>) -> Result<LLMResponse> {
        self.chat_with_config(messages, &self.config).await
    }

    pub async fn chat_with_config(&self, messages: Vec<Message>, config: &LLMConfig) -> Result<LLMResponse> {
        // 简化实现，实际应该使用async-openai库
        let usage = TokenUsage::new(
            messages.iter().map(|m| m.content.len() as u32 / 4).sum(),
            10,
        );
        Ok(LLMResponse {
            content: "Mock response".to_string(),
            model: config.model.clone(),
            finish_reason: Some("stop".to_string()),
            usage,
            metadata: None,
        })
    }

    pub fn config(&self) -> &LLMConfig {
        &self.config
    }
}

#[async_trait]
impl LLMClientTrait for LLMClient {
    async fn chat(&self, messages: Vec<Message>) -> Result<LLMResponse> {
        self.chat(messages).await
    }

    async fn chat_with_config(&self, messages: Vec<Message>, config: &LLMConfig) -> Result<LLMResponse> {
        self.chat_with_config(messages, config).await
    }
}
