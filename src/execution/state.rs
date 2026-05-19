//! # 执行状态
//!
//! 定义执行状态跟踪和管理。

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 执行状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExecutionStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
}

#[allow(clippy::derivable_impls)]
impl Default for ExecutionStatus {
    fn default() -> Self {
        Self::Pending
    }
}

impl std::fmt::Display for ExecutionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExecutionStatus::Pending => write!(f, "pending"),
            ExecutionStatus::Running => write!(f, "running"),
            ExecutionStatus::Completed => write!(f, "completed"),
            ExecutionStatus::Failed => write!(f, "failed"),
            ExecutionStatus::Cancelled => write!(f, "cancelled"),
        }
    }
}

/// 执行阶段
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum ExecutionPhase {
    #[default]
    Initializing,
    Validating,
    Planning,
    Executing,
    Completed,
    RollingBack,
    Terminated,
}

impl std::fmt::Display for ExecutionPhase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExecutionPhase::Initializing => write!(f, "initializing"),
            ExecutionPhase::Validating => write!(f, "validating"),
            ExecutionPhase::Planning => write!(f, "planning"),
            ExecutionPhase::Executing => write!(f, "executing"),
            ExecutionPhase::Completed => write!(f, "completed"),
            ExecutionPhase::RollingBack => write!(f, "rolling_back"),
            ExecutionPhase::Terminated => write!(f, "terminated"),
        }
    }
}

/// 任务执行状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskState {
    /// 任务ID
    pub task_id: String,
    /// 任务名称
    pub name: String,
    /// 执行状态
    pub status: ExecutionStatus,
    /// 重试次数
    pub attempts: u32,
    /// 错误信息
    pub error: Option<String>,
    /// 开始时间
    pub start_time: Option<i64>,
    /// 结束时间
    pub end_time: Option<i64>,
}

impl TaskState {
    pub fn new(task_id: String, name: String) -> Self {
        Self {
            task_id,
            name,
            status: ExecutionStatus::Pending,
            attempts: 0,
            error: None,
            start_time: None,
            end_time: None,
        }
    }

    pub fn start(&mut self) {
        self.status = ExecutionStatus::Running;
        self.start_time = Some(chrono::Utc::now().timestamp_millis());
    }

    pub fn complete(&mut self) {
        self.status = ExecutionStatus::Completed;
        self.end_time = Some(chrono::Utc::now().timestamp_millis());
    }

    pub fn fail(&mut self, error: impl Into<String>) {
        self.status = ExecutionStatus::Failed;
        self.error = Some(error.into());
        self.end_time = Some(chrono::Utc::now().timestamp_millis());
    }

    pub fn cancel(&mut self) {
        self.status = ExecutionStatus::Cancelled;
        self.end_time = Some(chrono::Utc::now().timestamp_millis());
    }

    pub fn increment_attempts(&mut self) {
        self.attempts += 1;
    }

    pub fn duration_ms(&self) -> Option<u64> {
        match (self.start_time, self.end_time) {
            (Some(start), Some(end)) => Some((end - start) as u64),
            _ => None,
        }
    }
}

impl Default for TaskState {
    fn default() -> Self {
        Self {
            task_id: String::new(),
            name: String::new(),
            status: ExecutionStatus::Pending,
            attempts: 0,
            error: None,
            start_time: None,
            end_time: None,
        }
    }
}

/// 执行状态跟踪
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ExecutionState {
    /// 任务状态映射
    pub task_states: HashMap<String, TaskState>,
    /// 当前执行阶段
    pub phase: ExecutionPhase,
    /// 总体开始时间
    pub start_time: Option<i64>,
    /// 总体结束时间
    pub end_time: Option<i64>,
}

impl ExecutionState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn start(&mut self) {
        self.start_time = Some(chrono::Utc::now().timestamp_millis());
        self.phase = ExecutionPhase::Initializing;
    }

    pub fn set_phase(&mut self, phase: ExecutionPhase) {
        self.phase = phase;
    }

    pub fn add_task(&mut self, task_id: String, name: String) {
        self.task_states
            .insert(task_id.clone(), TaskState::new(task_id, name));
    }

    pub fn update_task(&mut self, task_id: String, status: ExecutionStatus) {
        let state = self
            .task_states
            .entry(task_id.clone())
            .or_insert_with(|| TaskState {
                task_id: task_id.clone(),
                name: String::new(),
                status: ExecutionStatus::Pending,
                attempts: 0,
                error: None,
                start_time: None,
                end_time: None,
            });
        state.status = status;
    }

    pub fn get_task_state(&self, task_id: &str) -> Option<&TaskState> {
        self.task_states.get(task_id)
    }

    pub fn completed_count(&self) -> usize {
        self.task_states
            .values()
            .filter(|s| s.status == ExecutionStatus::Completed)
            .count()
    }

    pub fn failed_count(&self) -> usize {
        self.task_states
            .values()
            .filter(|s| s.status == ExecutionStatus::Failed)
            .count()
    }

    pub fn running_count(&self) -> usize {
        self.task_states
            .values()
            .filter(|s| s.status == ExecutionStatus::Running)
            .count()
    }

    pub fn pending_count(&self) -> usize {
        self.task_states
            .values()
            .filter(|s| s.status == ExecutionStatus::Pending)
            .count()
    }

    pub fn total_count(&self) -> usize {
        self.task_states.len()
    }

    pub fn is_completed(&self) -> bool {
        self.phase == ExecutionPhase::Completed || self.phase == ExecutionPhase::Terminated
    }

    pub fn is_failed(&self) -> bool {
        self.task_states
            .values()
            .any(|s| s.status == ExecutionStatus::Failed)
    }

    pub fn all_completed(&self) -> bool {
        self.task_states.values().all(|s| {
            s.status == ExecutionStatus::Completed
                || s.status == ExecutionStatus::Failed
                || s.status == ExecutionStatus::Cancelled
        })
    }

    pub fn duration_ms(&self) -> Option<u64> {
        match (self.start_time, self.end_time) {
            (Some(start), Some(end)) => Some((end - start) as u64),
            _ => None,
        }
    }

    pub fn finish(&mut self) {
        self.end_time = Some(chrono::Utc::now().timestamp_millis());
        if self.is_failed() {
            self.phase = ExecutionPhase::Terminated;
        } else {
            self.phase = ExecutionPhase::Completed;
        }
    }
}
