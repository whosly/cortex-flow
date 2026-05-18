//! # LLM 响应模块
//!
//! 定义 LLM 调用返回的响应类型和相关数据结构。

use serde::{Deserialize, Serialize};

/// LLM 响应
///
/// 封装 LLM 调用的返回结果。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMResponse {
    /// 生成的文本内容
    pub content: String,
    /// Token 使用统计
    pub usage: TokenUsage,
    /// 模型名称
    pub model: String,
    /// 完成原因（如 "stop", "length" 等）
    pub finish_reason: Option<String>,
    /// 响应元数据
    #[serde(default)]
    pub metadata: Option<serde_json::Value>,
}

impl LLMResponse {
    /// 创建新的响应
    pub fn new(content: String, usage: TokenUsage, model: impl Into<String>) -> Self {
        Self {
            content,
            usage,
            model: model.into(),
            finish_reason: None,
            metadata: None,
        }
    }

    /// 设置完成原因
    pub fn with_finish_reason(mut self, reason: impl Into<String>) -> Self {
        self.finish_reason = Some(reason.into());
        self
    }

    /// 设置元数据
    pub fn with_metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = Some(metadata);
        self
    }

    /// 检查响应是否为空
    pub fn is_empty(&self) -> bool {
        self.content.trim().is_empty()
    }

    /// 获取内容长度
    pub fn len(&self) -> usize {
        self.content.len()
    }
}

/// Token 使用统计
///
/// 记录 LLM 调用消耗的 Token 数量。
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TokenUsage {
    /// 输入 token 数量
    #[serde(default)]
    pub prompt_tokens: u32,
    /// 输出 token 数量
    #[serde(default)]
    pub completion_tokens: u32,
    /// 总 token 数量
    #[serde(default)]
    pub total_tokens: u32,
}

impl TokenUsage {
    /// 创建新的 Token 使用统计
    pub fn new(prompt_tokens: u32, completion_tokens: u32) -> Self {
        let total_tokens = prompt_tokens + completion_tokens;
        Self {
            prompt_tokens,
            completion_tokens,
            total_tokens,
        }
    }

    /// 计算成本（基于每 token 价格）
    ///
    /// # 参数
    ///
    /// - `input_price`: 输入每 1M token 的价格（美元）
    /// - `output_price`: 输出每 1M token 的价格（美元）
    pub fn calculate_cost(&self, input_price: f64, output_price: f64) -> f64 {
        let input_cost = (self.prompt_tokens as f64 / 1_000_000.0) * input_price;
        let output_cost = (self.completion_tokens as f64 / 1_000_000.0) * output_price;
        input_cost + output_cost
    }

    /// 更新统计（累加）
    pub fn add(&mut self, other: &TokenUsage) {
        self.prompt_tokens += other.prompt_tokens;
        self.completion_tokens += other.completion_tokens;
        self.total_tokens += other.total_tokens;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_usage() {
        let usage = TokenUsage::new(100, 50);
        assert_eq!(usage.prompt_tokens, 100);
        assert_eq!(usage.completion_tokens, 50);
        assert_eq!(usage.total_tokens, 150);
    }

    #[test]
    fn test_llm_response() {
        let usage = TokenUsage::new(100, 50);
        let response = LLMResponse::new("Hello".to_string(), usage, "gpt-4");
        assert_eq!(response.content, "Hello");
        assert_eq!(response.model, "gpt-4");
        assert!(!response.is_empty());
    }
}
