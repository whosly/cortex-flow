//! # 执行上下文

use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::RwLock;
use chrono::Utc;
use uuid::Uuid;

#[derive(Clone)]
pub struct ExecutionContext {
    inner: Arc<RwLock<ExecutionContextInner>>,
}

struct ExecutionContextInner {
    session_id: String,
    data: HashMap<String, serde_json::Value>,
    logs: Vec<ContextLog>,
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
            })),
        }
    }

    pub fn session_id(&self) -> String {
        self.inner.read().session_id.clone()
    }

    pub fn get<T: for<'de> serde::Deserialize<'de>>(&self, key: impl Into<String>) -> Option<T> {
        let inner = self.inner.read();
        inner.data.get(&key.into()).and_then(|v| serde_json::from_value(v.clone()).ok())
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

    pub fn log(&mut self, level: impl Into<String>, message: impl Into<String>) {
        let mut inner = self.inner.write();
        inner.logs.push(ContextLog {
            timestamp: Utc::now(),
            level: level.into(),
            message: message.into(),
            task_id: None,
        });
    }

    pub fn log_with_task(&self, level: impl Into<String>, message: impl Into<String>, task_id: impl Into<String>) {
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
}

impl Default for ExecutionContext {
    fn default() -> Self {
        Self::new()
    }
}
