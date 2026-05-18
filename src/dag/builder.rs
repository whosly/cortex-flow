//! # DAG Builder

use std::sync::Arc;
use crate::error::Result;
use crate::task::TaskExecutor;
use crate::dag::{DAG, DAGNode, DAGEdge};

/// DAG构建器
pub struct DAGBuilder {
    /// DAG实例
    dag: DAG,
}

impl DAGBuilder {
    /// 创建新构建器
    pub fn new() -> Self {
        Self {
            dag: DAG::new(),
        }
    }

    /// 添加节点
    pub fn add_node<S>(&mut self, task: S) -> &mut Self
    where
        S: Into<Arc<dyn TaskExecutor>>,
    {
        let task = task.into();
        let node = DAGNode::new(task.task_id().to_string(), task);
        self.dag.add_node(node);
        self
    }

    /// 添加依赖边
    pub fn add_dependency(&mut self, from: impl Into<String>, to: impl Into<String>) -> &mut Self {
        self.dag.add_dependency(from, to);
        self
    }

    /// 添加边
    pub fn add_edge(&mut self, edge: DAGEdge) -> &mut Self {
        self.dag.add_edge(edge);
        self
    }

    /// 构建DAG
    pub fn build(self) -> Result<DAG> {
        self.dag.validate()?;
        Ok(self.dag)
    }

    /// 获取DAG引用
    pub fn dag(&self) -> &DAG {
        &self.dag
    }
}

impl Default for DAGBuilder {
    fn default() -> Self {
        Self::new()
    }
}
