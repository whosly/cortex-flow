//! # 执行节点

use std::sync::Arc;
use serde::{Deserialize, Serialize};
use crate::task::TaskExecutor;

/// 执行节点
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionNode {
    /// 节点ID
    pub id: String,
    /// 节点名称
    pub name: String,
    /// 依赖节点ID列表
    pub dependencies: Vec<String>,
    /// 任务执行器
    #[serde(skip)]
    pub task: Arc<dyn TaskExecutor>,
}

impl ExecutionNode {
    /// 创建新节点
    pub fn new(id: String, name: String, task: Arc<dyn TaskExecutor>) -> Self {
        Self {
            id,
            name,
            dependencies: Vec::new(),
            task,
        }
    }

    /// 添加依赖
    pub fn with_dependency(mut self, dep: impl Into<String>) -> Self {
        self.dependencies.push(dep.into());
        self
    }

    /// 添加多个依赖
    pub fn with_dependencies(mut self, deps: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.dependencies.extend(deps.into_iter().map(|d| d.into()));
        self
    }
}
