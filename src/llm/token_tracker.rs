//! # LLM Token 追踪器
//!
//! 累计追踪 LLM 调用的 Token 消耗和成本。

use crate::llm::TokenUsage;
use parking_lot::RwLock;
use std::sync::Arc;

/// LLM Token 追踪器
///
/// 跨多次 LLM 调用累计记录 Token 消耗和估算成本。
pub struct TokenTracker {
    inner: Arc<RwLock<TokenTrackerInner>>,
}

#[derive(Debug, Clone)]
struct TokenTrackerInner {
    /// 累计输入 token
    total_prompt_tokens: u64,
    /// 累计输出 token
    total_completion_tokens: u64,
    /// 调用次数
    call_count: u64,
    /// 累计成本（美元）
    total_cost: f64,
    /// 输入每 1M token 价格
    input_price_per_m: f64,
    /// 输出每 1M token 价格
    output_price_per_m: f64,
}

impl TokenTracker {
    /// 创建新的追踪器
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(TokenTrackerInner {
                total_prompt_tokens: 0,
                total_completion_tokens: 0,
                call_count: 0,
                total_cost: 0.0,
                input_price_per_m: 30.0,  // GPT-4 默认输入价格
                output_price_per_m: 60.0, // GPT-4 默认输出价格
            })),
        }
    }

    /// 创建带自定义定价的追踪器
    pub fn with_pricing(input_price_per_m: f64, output_price_per_m: f64) -> Self {
        let tracker = Self::new();
        {
            let mut inner = tracker.inner.write();
            inner.input_price_per_m = input_price_per_m;
            inner.output_price_per_m = output_price_per_m;
        }
        tracker
    }

    /// 记录一次调用的 Token 使用
    pub fn record(&self, usage: &TokenUsage) {
        let mut inner = self.inner.write();
        inner.total_prompt_tokens += usage.prompt_tokens as u64;
        inner.total_completion_tokens += usage.completion_tokens as u64;
        inner.call_count += 1;

        let cost = usage.calculate_cost(inner.input_price_per_m, inner.output_price_per_m);
        inner.total_cost += cost;
    }

    /// 获取累计输入 token 数
    pub fn total_prompt_tokens(&self) -> u64 {
        self.inner.read().total_prompt_tokens
    }

    /// 获取累计输出 token 数
    pub fn total_completion_tokens(&self) -> u64 {
        self.inner.read().total_completion_tokens
    }

    /// 获取累计总 token 数
    pub fn total_tokens(&self) -> u64 {
        let inner = self.inner.read();
        inner.total_prompt_tokens + inner.total_completion_tokens
    }

    /// 获取调用次数
    pub fn call_count(&self) -> u64 {
        self.inner.read().call_count
    }

    /// 获取累计成本（美元）
    pub fn total_cost(&self) -> f64 {
        self.inner.read().total_cost
    }

    /// 获取快照
    pub fn snapshot(&self) -> TokenTrackerSnapshot {
        let inner = self.inner.read();
        TokenTrackerSnapshot {
            total_prompt_tokens: inner.total_prompt_tokens,
            total_completion_tokens: inner.total_completion_tokens,
            total_tokens: inner.total_prompt_tokens + inner.total_completion_tokens,
            call_count: inner.call_count,
            total_cost: inner.total_cost,
        }
    }

    /// 重置追踪器
    pub fn reset(&self) {
        let mut inner = self.inner.write();
        inner.total_prompt_tokens = 0;
        inner.total_completion_tokens = 0;
        inner.call_count = 0;
        inner.total_cost = 0.0;
    }
}

impl Default for TokenTracker {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for TokenTracker {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
        }
    }
}

/// Token 追踪快照
#[derive(Debug, Clone)]
pub struct TokenTrackerSnapshot {
    pub total_prompt_tokens: u64,
    pub total_completion_tokens: u64,
    pub total_tokens: u64,
    pub call_count: u64,
    pub total_cost: f64,
}

impl std::fmt::Display for TokenTrackerSnapshot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "=== Token Usage Summary ===")?;
        writeln!(f, "Calls: {}", self.call_count)?;
        writeln!(f, "Prompt Tokens: {}", self.total_prompt_tokens)?;
        writeln!(f, "Completion Tokens: {}", self.total_completion_tokens)?;
        writeln!(f, "Total Tokens: {}", self.total_tokens)?;
        writeln!(f, "Estimated Cost: ${:.4}", self.total_cost)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_usage() {
        let tracker = TokenTracker::with_pricing(30.0, 60.0);
        tracker.record(&TokenUsage::new(1000, 500));
        tracker.record(&TokenUsage::new(2000, 1000));

        assert_eq!(tracker.total_prompt_tokens(), 3000);
        assert_eq!(tracker.total_completion_tokens(), 1500);
        assert_eq!(tracker.total_tokens(), 4500);
        assert_eq!(tracker.call_count(), 2);
    }

    #[test]
    fn test_snapshot() {
        let tracker = TokenTracker::new();
        tracker.record(&TokenUsage::new(100, 50));
        let snap = tracker.snapshot();
        assert_eq!(snap.total_prompt_tokens, 100);
        assert_eq!(snap.call_count, 1);
    }

    #[test]
    fn test_reset() {
        let tracker = TokenTracker::new();
        tracker.record(&TokenUsage::new(100, 50));
        tracker.reset();
        assert_eq!(tracker.call_count(), 0);
        assert_eq!(tracker.total_tokens(), 0);
    }
}
