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
//!
//! ## 模块结构
//!
//! - [`client`] - LLM 客户端接口和实现
//! - [`config`] - LLM 配置定义
//! - [`message`] - 消息类型定义
//! - [`response`] - 响应类型定义

pub mod client;
pub mod config;
pub mod message;
pub mod response;

// 重新导出常用类型
pub use self::client::{LLMClient, LLMClientTrait};
pub use self::config::{LLMConfig, LLMProvider};
pub use self::message::Message;
pub use self::response::{LLMResponse, TokenUsage};
