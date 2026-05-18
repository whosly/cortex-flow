//! # 上下文作用域

use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct ContextScope {
    pub name: String,
    pub level: usize,
    data: HashMap<String, serde_json::Value>,
}

impl ContextScope {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            level: 0,
            data: HashMap::new(),
        }
    }

    pub fn set(&mut self, key: impl Into<String>, value: impl Into<serde_json::Value>) {
        self.data.insert(key.into(), value.into());
    }

    pub fn get(&self, key: &str) -> Option<&serde_json::Value> {
        self.data.get(key)
    }

    pub fn remove(&mut self, key: &str) -> Option<serde_json::Value> {
        self.data.remove(key)
    }

    pub fn clear(&mut self) {
        self.data.clear();
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
}
