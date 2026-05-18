//! # DAG模块
//!
//! 定义DAG编排相关的核心类型和构建器。

use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use crate::error::{Error, Result};
use crate::task::TaskExecutor;

pub use node::DAGNode;
pub use edge::DAGEdge;
pub use dag_impl::DAG;
pub use builder::DAGBuilder;
pub use executor::DAGExecutor;

mod node;
mod edge;
mod dag_impl;
mod builder;
mod executor;

/// DAG节点执行结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeExecutionResult {
    /// 节点ID
    pub task_id: String,
    /// 节点名称
    pub name: String,
    /// 是否成功
    pub success: bool,
    /// 输出数据
    pub output: Option<serde_json::Value>,
    /// 错误信息
    pub error: Option<String>,
    /// 执行时间(毫秒)
    pub duration_ms: u64,
    /// 开始时间戳
    pub start_time: Option<i64>,
    /// 结束时间戳
    pub end_time: Option<i64>,
}

impl NodeExecutionResult {
    /// 创建成功结果
    pub fn success(task_id: String, name: String, output: serde_json::Value, duration_ms: u64) -> Self {
        let now = chrono::Utc::now().timestamp_millis();
        Self {
            task_id,
            name,
            success: true,
            output: Some(output),
            error: None,
            duration_ms,
            start_time: Some(now - duration_ms as i64),
            end_time: Some(now),
        }
    }

    /// 创建失败结果
    pub fn failure(task_id: String, name: String, error: String, duration_ms: u64) -> Self {
        let now = chrono::Utc::now().timestamp_millis();
        Self {
            task_id,
            name,
            success: false,
            output: None,
            error: Some(error),
            duration_ms,
            start_time: Some(now - duration_ms as i64),
            end_time: Some(now),
        }
    }

    /// 检查是否成功
    pub fn is_success(&self) -> bool {
        self.success
    }

    /// 获取错误信息
    pub fn error_message(&self) -> Option<&str> {
        self.error.as_deref()
    }
}
