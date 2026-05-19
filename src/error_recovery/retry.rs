//! # 重试策略

use serde::{Deserialize, Serialize};
use std::time::Duration;

/// 重试策略
///
/// 定义任务执行失败时的重试行为。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryPolicy {
    /// 最大重试次数
    pub max_retries: u32,
    /// 初始重试间隔（毫秒）
    pub initial_interval_ms: u64,
    /// 退避倍数（用于指数退避）
    pub backoff_multiplier: f64,
    /// 最大重试间隔（毫秒）
    pub max_interval_ms: u64,
    /// 是否添加抖动
    pub jitter: bool,
    /// 可重试的错误码列表（空表示所有错误都可重试）
    pub retryable_error_codes: Vec<String>,
}

impl RetryPolicy {
    /// 创建固定间隔重试策略
    pub fn fixed(max_retries: u32, interval_ms: u64) -> Self {
        Self {
            max_retries,
            initial_interval_ms: interval_ms,
            backoff_multiplier: 1.0,
            max_interval_ms: interval_ms,
            jitter: false,
            retryable_error_codes: Vec::new(),
        }
    }

    /// 创建指数退避重试策略
    pub fn exponential(max_retries: u32, initial_interval_ms: u64, multiplier: f64) -> Self {
        Self {
            max_retries,
            initial_interval_ms,
            backoff_multiplier: multiplier,
            max_interval_ms: 60_000, // 最大60秒
            jitter: true,
            retryable_error_codes: Vec::new(),
        }
    }

    /// 不重试
    pub fn none() -> Self {
        Self {
            max_retries: 0,
            initial_interval_ms: 0,
            backoff_multiplier: 1.0,
            max_interval_ms: 0,
            jitter: false,
            retryable_error_codes: Vec::new(),
        }
    }

    /// 默认策略：3次重试，指数退避
    pub fn default_policy() -> Self {
        Self::exponential(3, 1000, 2.0)
    }

    /// 设置最大重试间隔
    pub fn with_max_interval(mut self, max_interval_ms: u64) -> Self {
        self.max_interval_ms = max_interval_ms;
        self
    }

    /// 设置是否添加抖动
    pub fn with_jitter(mut self, jitter: bool) -> Self {
        self.jitter = jitter;
        self
    }

    /// 设置可重试的错误码
    pub fn with_retryable_codes(mut self, codes: Vec<String>) -> Self {
        self.retryable_error_codes = codes;
        self
    }

    /// 计算第 N 次重试的等待时间
    pub fn wait_duration(&self, retry_count: u32) -> Duration {
        if retry_count == 0 {
            return Duration::from_millis(0);
        }

        let base = self.initial_interval_ms as f64
            * self.backoff_multiplier.powi((retry_count - 1) as i32);
        let interval = base.min(self.max_interval_ms as f64) as u64;

        let jitter_ms = if self.jitter {
            // 简单抖动：随机 ±25%
            let range = (interval as f64 * 0.25) as u64;
            // 使用简单的伪随机（基于重试次数）
            let pseudo_random =
                ((retry_count as u64).wrapping_mul(6364136223846793005) + 1) % (range * 2 + 1);
            pseudo_random.saturating_sub(range)
        } else {
            0
        };

        Duration::from_millis(interval.saturating_add_signed(jitter_ms as i64))
    }

    /// 判断是否应该重试
    pub fn should_retry(&self, attempt: u32, error_code: Option<&str>) -> bool {
        if attempt >= self.max_retries {
            return false;
        }

        // 如果指定了可重试错误码列表，只重试列表中的错误
        if !self.retryable_error_codes.is_empty() {
            if let Some(code) = error_code {
                return self.retryable_error_codes.iter().any(|c| c == code);
            }
            return false;
        }

        true
    }

    /// 是否启用了重试
    pub fn is_enabled(&self) -> bool {
        self.max_retries > 0
    }
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self::default_policy()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fixed_retry() {
        let policy = RetryPolicy::fixed(3, 1000);
        assert_eq!(policy.wait_duration(1), Duration::from_millis(1000));
        assert_eq!(policy.wait_duration(2), Duration::from_millis(1000));
        assert_eq!(policy.wait_duration(3), Duration::from_millis(1000));
    }

    #[test]
    fn test_exponential_retry() {
        let policy = RetryPolicy::exponential(3, 1000, 2.0).with_jitter(false);
        assert_eq!(policy.wait_duration(1), Duration::from_millis(1000));
        assert_eq!(policy.wait_duration(2), Duration::from_millis(2000));
        assert_eq!(policy.wait_duration(3), Duration::from_millis(4000));
    }

    #[test]
    fn test_should_retry() {
        let policy = RetryPolicy::fixed(3, 1000);
        assert!(policy.should_retry(0, None));
        assert!(policy.should_retry(2, None));
        assert!(!policy.should_retry(3, None));
    }

    #[test]
    fn test_should_retry_with_error_code() {
        let policy = RetryPolicy::fixed(3, 1000).with_retryable_codes(vec!["TIMEOUT".to_string()]);
        assert!(policy.should_retry(0, Some("TIMEOUT")));
        assert!(!policy.should_retry(0, Some("VALIDATION")));
    }

    #[test]
    fn test_none_policy() {
        let policy = RetryPolicy::none();
        assert!(!policy.is_enabled());
        assert!(!policy.should_retry(0, None));
    }
}
