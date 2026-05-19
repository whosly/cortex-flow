//! # 上下文作用域
//!
//! 提供作用域隔离机制，防止不同任务间的数据污染。

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 上下文作用域
///
/// 每个作用域拥有独立的数据空间，与父作用域隔离。
/// 子作用域可以读取父作用域的数据（只读），但写入只影响当前作用域。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextScope {
    /// 作用域名称
    pub name: String,
    /// 嵌套层级
    pub level: usize,
    /// 当前作用域数据
    data: HashMap<String, serde_json::Value>,
    /// 父作用域数据快照（只读）
    parent_data: Option<HashMap<String, serde_json::Value>>,
}

impl ContextScope {
    /// 创建新的根作用域
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            level: 0,
            data: HashMap::new(),
            parent_data: None,
        }
    }

    /// 创建子作用域
    ///
    /// 子作用域继承父作用域的数据快照（只读），写入只影响子作用域自身。
    pub fn child(&self, name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            level: self.level + 1,
            data: HashMap::new(),
            parent_data: Some(self.data.clone()),
        }
    }

    /// 设置数据（只影响当前作用域）
    pub fn set(&mut self, key: impl Into<String>, value: impl Into<serde_json::Value>) {
        self.data.insert(key.into(), value.into());
    }

    /// 获取数据（优先当前作用域，然后查找父作用域）
    pub fn get(&self, key: &str) -> Option<&serde_json::Value> {
        self.data
            .get(key)
            .or_else(|| self.parent_data.as_ref()?.get(key))
    }

    /// 检查当前作用域是否包含指定键（不含父作用域）
    pub fn contains_local(&self, key: &str) -> bool {
        self.data.contains_key(key)
    }

    /// 检查是否包含指定键（含父作用域）
    pub fn contains(&self, key: &str) -> bool {
        self.data.contains_key(key)
            || self
                .parent_data
                .as_ref()
                .is_some_and(|p| p.contains_key(key))
    }

    /// 删除数据（只删除当前作用域的数据）
    pub fn remove(&mut self, key: &str) -> Option<serde_json::Value> {
        self.data.remove(key)
    }

    /// 清除当前作用域的数据（不影响父作用域）
    pub fn clear(&mut self) {
        self.data.clear();
    }

    /// 当前作用域数据项数量（不含父作用域）
    pub fn local_len(&self) -> usize {
        self.data.len()
    }

    /// 当前作用域是否为空（不含父作用域）
    pub fn is_local_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// 获取当前作用域的所有键（不含父作用域）
    pub fn local_keys(&self) -> Vec<&String> {
        self.data.keys().collect()
    }

    /// 是否为根作用域
    pub fn is_root(&self) -> bool {
        self.parent_data.is_none()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_scope_isolation() {
        let mut parent = ContextScope::new("parent");
        parent.set("shared", json!("from_parent"));
        parent.set("parent_only", json!("only_in_parent"));

        let mut child = parent.child("child");
        child.set("shared", json!("from_child"));
        child.set("child_only", json!("only_in_child"));

        // 子作用域读取自己的覆盖值
        assert_eq!(child.get("shared"), Some(&json!("from_child")));
        // 子作用域读取父作用域的值
        assert_eq!(child.get("parent_only"), Some(&json!("only_in_parent")));
        // 父作用域不受子作用域影响
        assert_eq!(parent.get("shared"), Some(&json!("from_parent")));
        // 父作用域看不到子作用域的数据
        assert!(parent.get("child_only").is_none());
    }

    #[test]
    fn test_scope_child_reads_parent() {
        let mut parent = ContextScope::new("parent");
        parent.set("config", json!({"timeout": 30}));

        let child = parent.child("child");
        assert_eq!(child.get("config"), Some(&json!({"timeout": 30})));
    }

    #[test]
    fn test_scope_level() {
        let root = ContextScope::new("root");
        assert_eq!(root.level, 0);
        assert!(root.is_root());

        let child = root.child("child");
        assert_eq!(child.level, 1);
        assert!(!child.is_root());
    }
}
