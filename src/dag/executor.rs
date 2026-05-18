//! # DAG 执行器
//!
//! 负责执行 DAG 中的所有任务节点。
//!
//! ## 模块概述
//!
//! DAG 执行器是任务执行的核心组件，负责：
//!
//! - **并行执行**: 支持配置最大并发数，实现任务的并行执行
//! - **依赖管理**: 自动处理任务间的依赖关系，确保执行顺序
//! - **状态跟踪**: 跟踪每个任务的执行状态和结果
//! - **错误处理**: 收集并返回所有任务的执行结果
//!
//! ## 执行算法
//!
//! 执行器使用改进的 Kahn 算法：
//!
//! 1. 计算所有边的入度
//! 2. 将入度为 0 的节点加入就绪队列
//! 3. 从就绪队列中取出一批节点并发执行
//! 4. 节点完成后，更新其后继节点的入度
//! 5. 当后继节点入度变为 0 时，加入就绪队列
//! 6. 重复步骤 3-5 直到所有节点执行完成
//!
//! ## 并行控制
//!
//! - `max_parallelism`: 控制同一时刻最多执行的任务数
//! - 默认为 10，可通过 `with_parallelism()` 自定义
//!
//! ## 使用示例
//!
//! ```rust,ignore
//! use ai_collector::dag::{DAG, DAGExecutor};
//!
//! let dag = DAG::new();
//! // 添加节点和边...
//!
//! let mut executor = DAGExecutor::with_parallelism(dag, 4);
//! let results = executor.execute_all(&ctx).await?;
//! ```

use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;
use futures::stream::{self, StreamExt};
use crate::error::{Error, Result};
use crate::context::ExecutionContext;
use crate::task::TaskExecutor;
use super::{DAG, NodeExecutionResult};

/// DAG执行器状态
#[derive(Debug, Default)]
struct ExecutorState {
    /// 已完成的结果
    results: HashMap<String, NodeExecutionResult>,
    /// 已完成数量
    completed_count: usize,
}

/// DAG执行器
pub struct DAGExecutor {
    /// DAG引用
    dag: Arc<DAG>,
    /// 执行器状态
    state: Arc<RwLock<ExecutorState>>,
    /// 最大并发数
    max_parallelism: usize,
}

impl DAGExecutor {
    /// 创建新的执行器
    pub fn new(dag: DAG) -> Self {
        Self {
            dag: Arc::new(dag),
            state: Arc::new(RwLock::new(ExecutorState::default())),
            max_parallelism: 10,
        }
    }

    /// 创建新的执行器（带最大并发数）
    pub fn with_parallelism(dag: DAG, max_parallelism: usize) -> Self {
        Self {
            dag: Arc::new(dag),
            state: Arc::new(RwLock::new(ExecutorState::default())),
            max_parallelism,
        }
    }

    /// 执行所有节点
    pub async fn execute_all(&mut self, ctx: &ExecutionContext) -> Result<Vec<NodeExecutionResult>> {
        self.dag.validate()?;

        let dag = Arc::clone(&self.dag);
        let order = dag.topological_sort()?;
        
        // 计算初始入度
        let mut in_degree: HashMap<String, usize> = dag.get_edges().iter()
            .fold(HashMap::new(), |mut acc, edge| {
                *acc.entry(edge.to.clone()).or_insert(0) += 1;
                acc
            });

        // 找到所有根节点（入度为0的节点）
        let mut ready_queue: VecDeque<String> = order.iter()
            .filter(|id| !in_degree.contains_key(*id))
            .cloned()
            .collect();

        let mut all_results = Vec::new();

        while !ready_queue.is_empty() {
            // 获取当前批次大小
            let batch_size = ready_queue.len().min(self.max_parallelism);
            let mut handles = Vec::with_capacity(batch_size);

            for _ in 0..batch_size {
                if let Some(node_id) = ready_queue.pop_front() {
                    let dag_clone = Arc::clone(&dag);
                    let ctx_clone = ctx.clone();
                    let node_id_clone = node_id.clone();
                    
                    handles.push(async move {
                        // 直接获取 task 引用，避免克隆整个 DAGNode（会丢失 task）
                        if let Some(node) = dag_clone.get_node(&node_id_clone) {
                            let task = Arc::clone(&node.task);
                            let name = node.name.clone();
                            Self::execute_single_node(node_id_clone, name, task, &ctx_clone).await
                        } else {
                            Err(Error::TaskNotFound(node_id_clone))
                        }
                    });
                }
            }

            // 并发执行当前批次
            let results: Vec<NodeExecutionResult> = stream::iter(handles)
                .buffer_unordered(self.max_parallelism)
                .filter_map(|r| async { r.ok() })
                .collect()
                .await;

            // 处理执行结果
            for result in results {
                let node_id = result.task_id.clone();
                all_results.push(result.clone());

                // 更新状态
                {
                    let mut state = self.state.write().await;
                    state.results.insert(node_id.clone(), result);
                    state.completed_count += 1;
                }

                // 更新后继节点的入度
                for succ in dag.successors(&node_id) {
                    if let Some(deg) = in_degree.get_mut(&succ.id) {
                        *deg -= 1;
                        if *deg == 0 {
                            ready_queue.push_back(succ.id.clone());
                        }
                    }
                }
            }
        }

        Ok(all_results)
    }

    /// 执行单个节点
    async fn execute_single_node(
        task_id: String,
        name: String,
        task: Arc<dyn TaskExecutor>,
        ctx: &ExecutionContext,
    ) -> Result<NodeExecutionResult> {
        let start_time = Instant::now();

        match task.execute_task(serde_json::Value::Null, ctx).await {
            Ok(output) => {
                let duration = start_time.elapsed().as_millis() as u64;
                Ok(NodeExecutionResult::success(task_id, name, output, duration))
            }
            Err(e) => {
                let duration = start_time.elapsed().as_millis() as u64;
                Ok(NodeExecutionResult::failure(task_id, name, e.to_string(), duration))
            }
        }
    }

    /// 检查是否完成
    pub async fn is_completed(&self) -> bool {
        let state = self.state.read().await;
        state.completed_count >= self.dag.node_count()
    }

    /// 获取已完成的结果
    pub async fn get_results(&self) -> HashMap<String, NodeExecutionResult> {
        let state = self.state.read().await;
        state.results.clone()
    }

    /// 获取成功的结果
    pub async fn get_successful_results(&self) -> Vec<NodeExecutionResult> {
        let state = self.state.read().await;
        state.results.values()
            .filter(|r| r.success)
            .cloned()
            .collect()
    }

    /// 获取失败的结果
    pub async fn get_failed_results(&self) -> Vec<NodeExecutionResult> {
        let state = self.state.read().await;
        state.results.values()
            .filter(|r| !r.success)
            .cloned()
            .collect()
    }

    /// 获取已完成的节点数量
    pub async fn completed_count(&self) -> usize {
        let state = self.state.read().await;
        state.completed_count
    }
}

impl Clone for DAGExecutor {
    fn clone(&self) -> Self {
        Self {
            dag: Arc::clone(&self.dag),
            state: Arc::clone(&self.state),
            max_parallelism: self.max_parallelism,
        }
    }
}
