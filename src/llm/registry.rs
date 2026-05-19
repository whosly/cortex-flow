//! # LLM 客户端注册表
//!
//! 提供基于名称的 LLM 客户端注册和查找机制，支持多模型管理。
//!
//! ## 使用示例
//!
//! ```rust,ignore
//! use cortex_flow::llm::{LLMClientRegistry, LLMClient, LLMConfig};
//!
//! let mut registry = LLMClientRegistry::new();
//!
//! // 注册客户端
//! registry.register("gpt4", LLMClient::new(LLMConfig::openai("sk-xxx", "gpt-4")));
//! registry.register("llama3", LLMClient::new(LLMConfig::ollama("llama3")));
//!
//! // 设置默认客户端
//! registry.set_default("gpt4");
//!
//! // 获取客户端
//! let client = registry.get("gpt4").unwrap();
//! let default_client = registry.default_client().unwrap();
//! ```

use super::{LLMClientTrait, LLMConfig, TokenTracker, TokenUsage};
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;

/// LLM 客户端注册项
///
/// 保存客户端实例及其配置的快照，方便后续查询。
struct RegistryEntry {
    client: Arc<dyn LLMClientTrait>,
    config: LLMConfig,
}

/// LLM 客户端注册表
///
/// 支持按名称注册、查找和管理多个 LLM 客户端。
/// 内置 `TokenTracker` 自动聚合所有客户端的 Token 使用量。
///
/// # 线程安全
///
/// 内部使用 `RwLock` 保护，可以安全地在多个线程间共享。
#[derive(Clone)]
pub struct LLMClientRegistry {
    inner: Arc<RwLock<LLMClientRegistryInner>>,
}

struct LLMClientRegistryInner {
    /// 注册的客户端映射
    clients: HashMap<String, RegistryEntry>,
    /// 默认客户端名称
    default_name: Option<String>,
    /// 聚合 Token 追踪器
    token_tracker: TokenTracker,
}

impl LLMClientRegistry {
    /// 创建空的注册表
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(LLMClientRegistryInner {
                clients: HashMap::new(),
                default_name: None,
                token_tracker: TokenTracker::new(),
            })),
        }
    }

    /// 注册 LLM 客户端
    ///
    /// 如果名称已存在，覆盖旧的客户端。
    /// 如果这是第一个注册的客户端，自动设为默认。
    ///
    /// # 参数
    ///
    /// - `name`: 客户端名称（如 "gpt4"、"llama3"、"doubao"）
    /// - `client`: 客户端实例
    pub fn register(&self, name: impl Into<String>, client: impl Into<Arc<dyn LLMClientTrait>>) {
        let name = name.into();
        let client = client.into();
        let config = client.default_config();

        let mut inner = self.inner.write();
        let is_first = inner.clients.is_empty();
        inner
            .clients
            .insert(name.clone(), RegistryEntry { client, config });
        if is_first || inner.default_name.is_none() {
            inner.default_name = Some(name);
        }
    }

    /// 注册 LLM 客户端（使用配置自动创建）
    ///
    /// 便捷方法：直接传入 `LLMConfig`，自动创建 `LLMClient` 并注册。
    /// 创建的客户端会自动关联注册表的共享 TokenTracker。
    ///
    /// # 参数
    ///
    /// - `name`: 客户端名称
    /// - `config`: LLM 配置
    pub fn register_with_config(&self, name: impl Into<String>, config: LLMConfig) {
        use super::client::LLMClient;
        let tracker = self.inner.read().token_tracker.clone();
        let client =
            Arc::new(LLMClient::with_token_tracker(config, tracker)) as Arc<dyn LLMClientTrait>;
        self.register(name, client);
    }

    /// 注销 LLM 客户端
    ///
    /// 返回被移除的客户端，如果不存在则返回 `None`。
    /// 如果移除的是默认客户端，默认设置将被清除。
    pub fn unregister(&self, name: &str) -> Option<Arc<dyn LLMClientTrait>> {
        let mut inner = self.inner.write();
        let removed = inner.clients.remove(name);
        if inner.default_name.as_deref() == Some(name) {
            inner.default_name = inner.clients.keys().next().cloned();
        }
        removed.map(|e| e.client)
    }

    /// 获取指定名称的客户端
    pub fn get(&self, name: &str) -> Option<Arc<dyn LLMClientTrait>> {
        self.inner
            .read()
            .clients
            .get(name)
            .map(|e| e.client.clone())
    }

    /// 获取默认客户端
    ///
    /// 默认客户端是第一个注册的客户端，或通过 `set_default` 指定的客户端。
    pub fn default_client(&self) -> Option<Arc<dyn LLMClientTrait>> {
        let inner = self.inner.read();
        inner
            .default_name
            .as_ref()
            .and_then(|name| inner.clients.get(name).map(|e| e.client.clone()))
    }

    /// 获取默认客户端名称
    pub fn default_name(&self) -> Option<String> {
        self.inner.read().default_name.clone()
    }

    /// 设置默认客户端
    ///
    /// 名称必须已注册，否则返回 `false`。
    pub fn set_default(&self, name: &str) -> bool {
        let mut inner = self.inner.write();
        if inner.clients.contains_key(name) {
            inner.default_name = Some(name.to_string());
            true
        } else {
            false
        }
    }

    /// 获取指定名称客户端的配置
    pub fn get_config(&self, name: &str) -> Option<LLMConfig> {
        self.inner
            .read()
            .clients
            .get(name)
            .map(|e| e.config.clone())
    }

    /// 获取所有已注册的客户端名称
    pub fn names(&self) -> Vec<String> {
        self.inner.read().clients.keys().cloned().collect()
    }

    /// 获取已注册客户端数量
    pub fn len(&self) -> usize {
        self.inner.read().clients.len()
    }

    /// 检查注册表是否为空
    pub fn is_empty(&self) -> bool {
        self.inner.read().clients.is_empty()
    }

    /// 检查是否包含指定名称的客户端
    pub fn contains(&self, name: &str) -> bool {
        self.inner.read().clients.contains_key(name)
    }

    /// 记录 Token 使用量
    ///
    /// 通常由 LLMClient 自动调用，也可手动调用以记录外部 Token 消耗。
    pub fn record_token_usage(&self, usage: &TokenUsage) {
        self.inner.read().token_tracker.record(usage);
    }

    /// 获取聚合 Token 追踪器
    pub fn token_tracker(&self) -> TokenTracker {
        self.inner.read().token_tracker.clone()
    }

    /// 获取 Token 使用快照
    pub fn token_snapshot(&self) -> super::token_tracker::TokenTrackerSnapshot {
        self.inner.read().token_tracker.snapshot()
    }
}

impl Default for LLMClientRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for LLMClientRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let inner = self.inner.read();
        f.debug_struct("LLMClientRegistry")
            .field("clients", &inner.clients.keys().collect::<Vec<_>>())
            .field("default_name", &inner.default_name)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::llm::MockLLMClient;

    #[test]
    fn test_register_and_get() {
        let registry = LLMClientRegistry::new();

        let mock1 = MockLLMClient::new("response1");
        let mock2 = MockLLMClient::new("response2");

        registry.register("gpt4", Arc::new(mock1) as Arc<dyn LLMClientTrait>);
        registry.register("llama3", Arc::new(mock2) as Arc<dyn LLMClientTrait>);

        assert!(registry.get("gpt4").is_some());
        assert!(registry.get("llama3").is_some());
        assert!(registry.get("unknown").is_none());
        assert_eq!(registry.len(), 2);
    }

    #[test]
    fn test_default_client() {
        let registry = LLMClientRegistry::new();

        // 第一个注册的自动成为默认
        registry.register("gpt4", MockLLMClient::new("gpt4").into_trait());
        assert_eq!(registry.default_name(), Some("gpt4".to_string()));
        assert!(registry.default_client().is_some());

        // 设置新默认
        registry.register("llama3", MockLLMClient::new("llama3").into_trait());
        assert_eq!(registry.default_name(), Some("gpt4".to_string())); // 仍然是 gpt4

        assert!(registry.set_default("llama3"));
        assert_eq!(registry.default_name(), Some("llama3".to_string()));

        // 不存在的名称
        assert!(!registry.set_default("unknown"));
    }

    #[test]
    fn test_unregister() {
        let registry = LLMClientRegistry::new();

        registry.register("gpt4", MockLLMClient::new("gpt4").into_trait());
        registry.register("llama3", MockLLMClient::new("llama3").into_trait());

        // 注销默认客户端，自动切换
        let removed = registry.unregister("gpt4");
        assert!(removed.is_some());
        assert_eq!(registry.default_name(), Some("llama3".to_string()));

        // 不存在的
        let removed = registry.unregister("unknown");
        assert!(removed.is_none());
    }

    #[test]
    fn test_register_with_config() {
        let registry = LLMClientRegistry::new();
        let config = LLMConfig::ollama("test-model");

        registry.register_with_config("local", config);
        assert!(registry.get("local").is_some());
        assert!(registry.get_config("local").is_some());
    }

    #[test]
    fn test_names_and_contains() {
        let registry = LLMClientRegistry::new();
        assert!(registry.is_empty());

        registry.register("a", MockLLMClient::new("a").into_trait());
        registry.register("b", MockLLMClient::new("b").into_trait());

        assert!(!registry.is_empty());
        assert!(registry.contains("a"));
        assert!(registry.contains("b"));
        assert!(!registry.contains("c"));

        let mut names = registry.names();
        names.sort();
        assert_eq!(names, vec!["a", "b"]);
    }
}
