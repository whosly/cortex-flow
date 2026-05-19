//! # LLM 模块
//!
//! 提供大语言模型（LLM）客户端封装，支持多种 LLM 服务提供商。
//!
//! ## 支持的提供商
//!
//! - **OpenAI**: GPT-4, GPT-3.5 等
//! - **Anthropic**: Claude 系列
//! - **Azure OpenAI**: Azure 托管的 OpenAI 模型
//! - **Ollama**: 本地 LLM 模型
//! - **Custom**: 任何兼容 OpenAI Chat Completions API 的服务（如火山引擎豆包、DeepSeek 等）
//!
//! ## 多模型管理
//!
//! 通过 [`LLMClientRegistry`] 可以注册和管理多个 LLM 客户端，
//! 在 DAG 任务中按名称获取不同的模型：
//!
//! ```rust,ignore
//! use cortex_flow::llm::{LLMClientRegistry, LLMConfig};
//!
//! let registry = LLMClientRegistry::new();
//! registry.register_with_config("gpt4", LLMConfig::openai("sk-xxx", "gpt-4"));
//! registry.register_with_config("llama3", LLMConfig::ollama("llama3"));
//! registry.register_with_config("doubao", LLMConfig::custom("key", "doubao-pro-32k", "https://ark.cn-beijing.volces.com/api/v3"));
//!
//! let gpt4 = registry.get("gpt4").unwrap();
//! let doubao = registry.get("doubao").unwrap();
//! ```
//!
//! ## 模块结构
//!
//! - [`client`] - LLM 客户端接口和实现
//! - [`config`] - LLM 配置定义
//! - [`message`] - 消息类型定义（含 [`ChatRequest`]）
//! - [`registry`] - LLM 客户端注册表
//! - [`response`] - 响应类型定义
//! - [`token_tracker`] - Token 消耗追踪

pub mod client;
pub mod config;
pub mod message;
pub mod registry;
pub mod response;
pub mod token_tracker;

// 重新导出常用类型
pub use self::client::{chat_structured, LLMClient, LLMClientTrait, MockLLMClient, StreamChunk};
pub use self::config::{LLMConfig, LLMProvider};
pub use self::message::{ChatRequest, Message};
pub use self::registry::LLMClientRegistry;
pub use self::response::{LLMResponse, TokenUsage};
pub use self::token_tracker::{TokenTracker, TokenTrackerSnapshot};
