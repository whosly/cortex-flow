//! # 执行上下文

use crate::llm::{LLMClientRegistry, LLMClientTrait, TokenTrackerSnapshot};
use chrono::Utc;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Clone)]
pub struct ExecutionContext {
    inner: Arc<RwLock<ExecutionContextInner>>,
}

struct ExecutionContextInner {
    session_id: String,
    data: HashMap<String, serde_json::Value>,
    logs: Vec<ContextLog>,
    /// LLM 客户端注册表引用（可选）
    llm_registry: Option<Arc<LLMClientRegistry>>,
}

#[derive(Debug, Clone)]
pub struct ContextLog {
    pub timestamp: chrono::DateTime<Utc>,
    pub level: String,
    pub message: String,
    pub task_id: Option<String>,
}

impl ExecutionContext {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(ExecutionContextInner {
                session_id: Uuid::new_v4().to_string(),
                data: HashMap::new(),
                logs: Vec::new(),
                llm_registry: None,
            })),
        }
    }

    pub fn session_id(&self) -> String {
        self.inner.read().session_id.clone()
    }

    pub fn get<T: for<'de> serde::Deserialize<'de>>(&self, key: impl Into<String>) -> Option<T> {
        let inner = self.inner.read();
        inner
            .data
            .get(&key.into())
            .and_then(|v| serde_json::from_value(v.clone()).ok())
    }

    pub fn set<K, V>(&mut self, key: K, value: V) -> std::result::Result<(), crate::error::Error>
    where
        K: Into<String>,
        V: serde::Serialize,
    {
        let mut inner = self.inner.write();
        let json_value = serde_json::to_value(value)?;
        inner.data.insert(key.into(), json_value);
        Ok(())
    }

    /// 检查是否包含指定键
    pub fn contains(&self, key: &str) -> bool {
        self.inner.read().data.contains_key(key)
    }

    /// 删除指定键
    pub fn remove(&self, key: &str) -> Option<serde_json::Value> {
        self.inner.write().data.remove(key)
    }

    /// 获取所有键
    pub fn keys(&self) -> Vec<String> {
        self.inner.read().data.keys().cloned().collect()
    }

    /// 获取数据项数量
    pub fn len(&self) -> usize {
        self.inner.read().data.len()
    }

    /// 检查是否为空
    pub fn is_empty(&self) -> bool {
        self.inner.read().data.is_empty()
    }

    pub fn log(&mut self, level: impl Into<String>, message: impl Into<String>) {
        let mut inner = self.inner.write();
        inner.logs.push(ContextLog {
            timestamp: Utc::now(),
            level: level.into(),
            message: message.into(),
            task_id: None,
        });
    }

    pub fn log_with_task(
        &self,
        level: impl Into<String>,
        message: impl Into<String>,
        task_id: impl Into<String>,
    ) {
        let mut inner = self.inner.write();
        inner.logs.push(ContextLog {
            timestamp: Utc::now(),
            level: level.into(),
            message: message.into(),
            task_id: Some(task_id.into()),
        });
    }

    pub fn logs(&self) -> Vec<ContextLog> {
        self.inner.read().logs.clone()
    }

    /// 创建上下文快照
    ///
    /// 返回当前上下文数据的快照，可用于后续恢复。
    pub fn snapshot(&self) -> ContextSnapshot {
        let inner = self.inner.read();
        ContextSnapshot {
            session_id: inner.session_id.clone(),
            data: inner.data.clone(),
            log_count: inner.logs.len(),
        }
    }

    /// 从快照恢复上下文
    ///
    /// 用快照中的数据替换当前上下文数据。
    /// 注意：日志不会被恢复。
    pub fn restore(&mut self, snapshot: &ContextSnapshot) {
        let mut inner = self.inner.write();
        inner.session_id = snapshot.session_id.clone();
        inner.data = snapshot.data.clone();
    }

    /// 清除所有数据
    pub fn clear(&mut self) {
        let mut inner = self.inner.write();
        inner.data.clear();
        inner.logs.clear();
    }

    // ========================================================================
    // LLM 客户端访问
    // ========================================================================

    /// 关联 LLM 客户端注册表
    ///
    /// 通常由 Orchestrator 在 DAG 执行前自动调用。
    /// 关联后，DAG 任务闭包中可通过 `ctx.get_llm()` 获取 LLM 客户端。
    pub fn attach_llm_registry(&mut self, registry: Arc<LLMClientRegistry>) {
        let mut inner = self.inner.write();
        inner.llm_registry = Some(registry);
    }

    /// 通过名称获取 LLM 客户端
    ///
    /// 需要先通过 `attach_llm_registry` 关联注册表。
    ///
    /// # 示例
    ///
    /// ```rust,ignore
    /// // 在 DAG 任务闭包中：
    /// if let Some(client) = ctx.get_llm("gpt4") {
    ///     let response = client.chat(messages).await?;
    /// }
    /// ```
    pub fn get_llm(&self, name: &str) -> Option<Arc<dyn LLMClientTrait>> {
        self.inner
            .read()
            .llm_registry
            .as_ref()
            .and_then(|r| r.get(name))
    }

    /// 获取默认 LLM 客户端
    ///
    /// 返回注册表中标记为默认的客户端，或第一个注册的客户端。
    pub fn default_llm(&self) -> Option<Arc<dyn LLMClientTrait>> {
        self.inner
            .read()
            .llm_registry
            .as_ref()
            .and_then(|r| r.default_client())
    }

    /// 获取所有已注册的 LLM 客户端名称
    pub fn llm_names(&self) -> Vec<String> {
        self.inner
            .read()
            .llm_registry
            .as_ref()
            .map(|r| r.names())
            .unwrap_or_default()
    }

    /// 获取 Token 使用快照
    pub fn token_snapshot(&self) -> Option<TokenTrackerSnapshot> {
        self.inner
            .read()
            .llm_registry
            .as_ref()
            .map(|r| r.token_snapshot())
    }
}

/// 上下文快照
///
/// 保存上下文在某个时刻的数据状态，可用于恢复。
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ContextSnapshot {
    /// 会话ID
    pub session_id: String,
    /// 数据快照
    pub data: HashMap<String, serde_json::Value>,
    /// 快照时的日志条数
    pub log_count: usize,
}

impl ContextSnapshot {
    /// 获取快照中的数据项数量
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// 检查快照是否为空
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
}

impl Default for ExecutionContext {
    fn default() -> Self {
        Self::new()
    }
}
