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

use super::{NodeExecutionResult, DAG};
use crate::context::ExecutionContext;
use crate::error::{Error, Result};
use crate::task::TaskExecutor;
use futures::stream::{self, StreamExt};
use parking_lot::RwLock as SyncRwLock;
use std::collections::{HashMap, VecDeque};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;

/// DAG执行器状态
#[derive(Debug, Default)]
struct ExecutorState {
    /// 已完成的结果
    results: HashMap<String, NodeExecutionResult>,
    /// 已完成数量
    completed_count: usize,
}

/// 任务执行进度
#[derive(Debug, Clone)]
pub struct ExecutionProgress {
    /// 已完成节点数
    pub completed: usize,
    /// 总节点数
    pub total: usize,
    /// 当前正在执行的任务数
    pub running: usize,
    /// 进度百分比 (0-100)
    pub percent: u8,
    /// 成功完成的任务数
    pub success_count: usize,
    /// 失败的任务数
    pub failure_count: usize,
}

impl ExecutionProgress {
    fn new(total: usize) -> Self {
        Self {
            completed: 0,
            total,
            running: 0,
            percent: 0,
            success_count: 0,
            failure_count: 0,
        }
    }

    fn update(&mut self, completed: usize, running: usize, success: usize, failure: usize) {
        self.completed = completed;
        self.running = running;
        self.success_count = success;
        self.failure_count = failure;
        self.percent = if self.total > 0 {
            ((completed as f64 / self.total as f64) * 100.0) as u8
        } else {
            0
        };
    }
}

/// 进度回调函数类型
pub type ProgressCallback = Arc<dyn Fn(ExecutionProgress) + Send + Sync>;

/// DAG执行器
pub struct DAGExecutor {
    /// DAG引用
    dag: Arc<DAG>,
    /// 执行器状态
    state: Arc<RwLock<ExecutorState>>,
    /// 节点输出数据（用于任务间数据传递，使用同步锁以支持同步读取）
    node_outputs: Arc<SyncRwLock<HashMap<String, serde_json::Value>>>,
    /// 最大并发数
    max_parallelism: usize,
    /// 取消标志
    cancelled: Arc<AtomicBool>,
    /// 进度回调
    progress_callback: Option<ProgressCallback>,
}

impl DAGExecutor {
    /// 创建新的执行器
    pub fn new(dag: DAG) -> Self {
        Self {
            dag: Arc::new(dag),
            state: Arc::new(RwLock::new(ExecutorState::default())),
            node_outputs: Arc::new(SyncRwLock::new(HashMap::new())),
            max_parallelism: 10,
            cancelled: Arc::new(AtomicBool::new(false)),
            progress_callback: None,
        }
    }

    /// 创建新的执行器（带最大并发数）
    pub fn with_parallelism(dag: DAG, max_parallelism: usize) -> Self {
        Self {
            dag: Arc::new(dag),
            state: Arc::new(RwLock::new(ExecutorState::default())),
            node_outputs: Arc::new(SyncRwLock::new(HashMap::new())),
            max_parallelism,
            cancelled: Arc::new(AtomicBool::new(false)),
            progress_callback: None,
        }
    }

    /// 设置进度回调
    pub fn with_progress_callback<C: Fn(ExecutionProgress) + Send + Sync + 'static>(
        mut self,
        callback: C,
    ) -> Self {
        self.progress_callback = Some(Arc::new(callback));
        self
    }

    /// 获取取消标志的引用
    pub fn cancelled_flag(&self) -> Arc<AtomicBool> {
        Arc::clone(&self.cancelled)
    }

    /// 取消执行
    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::SeqCst);
    }

    /// 检查是否已取消
    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::SeqCst)
    }

    /// 获取当前进度
    pub async fn get_progress(&self) -> ExecutionProgress {
        let state = self.state.read().await;
        let total = self.dag.node_count();
        let success = state.results.values().filter(|r| r.success).count();
        let failure = state.results.values().filter(|r| !r.success).count();
        let completed = state.completed_count;
        let running = total.saturating_sub(completed);

        let mut progress = ExecutionProgress::new(total);
        progress.update(completed, running, success, failure);
        progress
    }

    /// 执行所有节点
    pub async fn execute_all(
        &mut self,
        ctx: &ExecutionContext,
    ) -> Result<Vec<NodeExecutionResult>> {
        self.dag.validate()?;

        // 重置取消标志
        self.cancelled.store(false, Ordering::SeqCst);

        let dag = Arc::clone(&self.dag);
        let total_nodes = dag.node_count();
        let order = dag.topological_sort()?;
        let progress_callback = self.progress_callback.clone();

        // 计算初始入度
        let mut in_degree: HashMap<String, usize> =
            dag.get_edges()
                .iter()
                .fold(HashMap::new(), |mut acc, edge| {
                    *acc.entry(edge.to.clone()).or_insert(0) += 1;
                    acc
                });

        // 找到所有根节点（入度为0的节点）
        let mut ready_queue: VecDeque<String> = order
            .iter()
            .filter(|id| !in_degree.contains_key(*id))
            .cloned()
            .collect();

        let mut all_results = Vec::new();
        let cancelled = Arc::clone(&self.cancelled);

        // 通知进度回调开始执行
        if let Some(ref cb) = progress_callback {
            let mut progress = ExecutionProgress::new(total_nodes);
            progress.update(0, ready_queue.len(), 0, 0);
            cb(progress);
        }

        while !ready_queue.is_empty() {
            // 检查取消标志
            if cancelled.load(Ordering::SeqCst) {
                // 收集已取消时已完成的结果
                let state = self.state.read().await;
                for (_, result) in state.results.iter() {
                    if !all_results.contains(result) {
                        all_results.push(result.clone());
                    }
                }
                return Err(Error::Cancelled("Execution was cancelled".to_string()));
            }

            // 获取当前批次大小
            let batch_size = ready_queue.len().min(self.max_parallelism);
            let mut handles = Vec::with_capacity(batch_size);

            for _ in 0..batch_size {
                if let Some(node_id) = ready_queue.pop_front() {
                    let dag_clone = Arc::clone(&dag);
                    let ctx_clone = ctx.clone();
                    let node_id_clone = node_id.clone();
                    let cancelled_clone = Arc::clone(&cancelled);

                    // 收集上游节点的输出作为输入
                    let upstream_output = self.collect_upstream_output_sync(&node_id);

                    handles.push(async move {
                        // 检查取消状态后再执行
                        if cancelled_clone.load(Ordering::SeqCst) {
                            return Err(Error::Cancelled(
                                "Execution was cancelled before task started".to_string(),
                            ));
                        }

                        // 直接获取 task 引用，避免克隆整个 DAGNode（会丢失 task）
                        if let Some(node) = dag_clone.get_node(&node_id_clone) {
                            let task = Arc::clone(&node.task);
                            let name = node.name.clone();
                            Self::execute_single_node(
                                node_id_clone,
                                name,
                                task,
                                &ctx_clone,
                                upstream_output,
                            )
                            .await
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

                // 保存节点输出用于下游任务
                if result.success {
                    if let Some(ref output) = result.output {
                        self.node_outputs
                            .write()
                            .insert(node_id.clone(), output.clone());
                    }
                }

                all_results.push(result.clone());

                // 更新状态
                {
                    let mut state = self.state.write().await;
                    state.results.insert(node_id.clone(), result);
                    state.completed_count += 1;
                }

                // 触发进度回调
                if let Some(ref cb) = progress_callback {
                    let prog = self.get_progress().await;
                    cb(prog);
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

    /// 同步方式收集上游节点的输出数据
    ///
    /// 如果只有一个上游，直接传递其输出。
    /// 如果有多个上游，合并为一个 JSON 对象，key 为上游节点 ID。
    fn collect_upstream_output_sync(&self, node_id: &str) -> serde_json::Value {
        let dag = &self.dag;
        let predecessors: Vec<String> = dag
            .predecessors(node_id)
            .iter()
            .map(|n| n.id.clone())
            .collect();

        if predecessors.is_empty() {
            return serde_json::Value::Null;
        }

        let outputs = self.node_outputs.read();

        if predecessors.len() == 1 {
            // 单个上游：直接传递输出
            outputs
                .get(&predecessors[0])
                .cloned()
                .unwrap_or(serde_json::Value::Null)
        } else {
            // 多个上游：合并为对象
            let mut merged = serde_json::Map::new();
            for pred_id in predecessors {
                if let Some(output) = outputs.get(&pred_id) {
                    merged.insert(pred_id, output.clone());
                }
            }
            serde_json::Value::Object(merged)
        }
    }

    /// 执行单个节点
    async fn execute_single_node(
        task_id: String,
        name: String,
        task: Arc<dyn TaskExecutor>,
        ctx: &ExecutionContext,
        input: serde_json::Value,
    ) -> Result<NodeExecutionResult> {
        let start_time = Instant::now();

        match task.execute_task(input, ctx).await {
            Ok(output) => {
                let duration = start_time.elapsed().as_millis() as u64;
                Ok(NodeExecutionResult::success(
                    task_id, name, output, duration,
                ))
            }
            Err(e) => {
                let duration = start_time.elapsed().as_millis() as u64;
                Ok(NodeExecutionResult::failure(
                    task_id,
                    name,
                    e.to_string(),
                    duration,
                ))
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
        state
            .results
            .values()
            .filter(|r| r.success)
            .cloned()
            .collect()
    }

    /// 获取失败的结果
    pub async fn get_failed_results(&self) -> Vec<NodeExecutionResult> {
        let state = self.state.read().await;
        state
            .results
            .values()
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
            node_outputs: Arc::clone(&self.node_outputs),
            max_parallelism: self.max_parallelism,
            cancelled: Arc::clone(&self.cancelled),
            progress_callback: self.progress_callback.clone(),
        }
    }
}
