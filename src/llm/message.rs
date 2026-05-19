//! # LLM 消息定义
//!
//! 定义 LLM 对话中的消息类型。

use serde::{Deserialize, Serialize};

/// 消息角色
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Role {
    /// 系统消息
    System,
    /// 用户消息
    User,
    /// 助手消息
    Assistant,
    /// 工具消息
    Tool,
}

impl Role {
    /// 转换为字符串
    pub fn as_str(&self) -> &'static str {
        match self {
            Role::System => "system",
            Role::User => "user",
            Role::Assistant => "assistant",
            Role::Tool => "tool",
        }
    }
}

impl std::fmt::Display for Role {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// LLM 消息
///
/// 表示一次对话中的单条消息。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    /// 消息角色
    pub role: Role,
    /// 消息内容
    pub content: String,
    /// 消息发送者名称（可选）
    pub name: Option<String>,
    /// 工具调用ID（用于工具消息）
    pub tool_call_id: Option<String>,
}

impl Message {
    /// 创建系统消息
    pub fn system(content: impl Into<String>) -> Self {
        Self {
            role: Role::System,
            content: content.into(),
            name: None,
            tool_call_id: None,
        }
    }

    /// 创建用户消息
    pub fn user(content: impl Into<String>) -> Self {
        Self {
            role: Role::User,
            content: content.into(),
            name: None,
            tool_call_id: None,
        }
    }

    /// 创建助手消息
    pub fn assistant(content: impl Into<String>) -> Self {
        Self {
            role: Role::Assistant,
            content: content.into(),
            name: None,
            tool_call_id: None,
        }
    }

    /// 创建工具消息
    pub fn tool(content: impl Into<String>, tool_call_id: impl Into<String>) -> Self {
        Self {
            role: Role::Tool,
            content: content.into(),
            name: None,
            tool_call_id: Some(tool_call_id.into()),
        }
    }

    /// 设置发送者名称
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }
}

/// 聊天请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatRequest {
    /// 消息列表
    pub messages: Vec<Message>,
    /// 模型名称（可选）
    pub model: Option<String>,
    /// 最大 token 数（可选）
    pub max_tokens: Option<u32>,
    /// 温度参数（可选）
    pub temperature: Option<f32>,
}

impl ChatRequest {
    /// 创建新的聊天请求
    pub fn new(messages: Vec<Message>) -> Self {
        Self {
            messages,
            model: None,
            max_tokens: None,
            temperature: None,
        }
    }

    /// 添加消息
    pub fn add_message(mut self, message: Message) -> Self {
        self.messages.push(message);
        self
    }

    /// 设置模型
    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.model = Some(model.into());
        self
    }

    /// 设置最大 token 数
    pub fn with_max_tokens(mut self, max_tokens: u32) -> Self {
        self.max_tokens = Some(max_tokens);
        self
    }

    /// 设置温度参数
    pub fn with_temperature(mut self, temperature: f32) -> Self {
        self.temperature = Some(temperature);
        self
    }
}
