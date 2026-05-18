//! # DAG Edge 定义

use serde::{Deserialize, Serialize};

/// DAG边
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DAGEdge {
    /// 源节点ID
    pub from: String,
    /// 目标节点ID
    pub to: String,
    /// 边标签
    pub label: Option<String>,
}

impl DAGEdge {
    /// 创建新边
    pub fn new(from: impl Into<String>, to: impl Into<String>) -> Self {
        Self {
            from: from.into(),
            to: to.into(),
            label: None,
        }
    }

    /// 创建带标签的边
    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }
}

impl PartialEq for DAGEdge {
    fn eq(&self, other: &Self) -> bool {
        self.from == other.from && self.to == other.to
    }
}

impl Eq for DAGEdge {}
