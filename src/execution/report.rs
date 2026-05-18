//! # 执行报告
//!
//! 定义执行完成后的报告生成功能。
//!
//! ## 模块概述
//!
//! - [`ExecutionReport`] - 完整的执行报告
//! - [`TaskReport`] - 单个任务的执行报告
//! - [`ReportGenerator`] - 报告生成器

use serde::{Deserialize, Serialize};
use crate::dag::NodeExecutionResult;
use crate::strategy::ExecutionResult;

/// 完整的执行报告
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionReport {
    /// 报告ID
    pub report_id: String,
    /// 执行是否成功
    pub success: bool,
    /// 总执行时间（毫秒）
    pub total_duration_ms: u64,
    /// 开始时间戳
    pub start_time: Option<i64>,
    /// 结束时间戳
    pub end_time: Option<i64>,
    /// 任务总数
    pub total_tasks: usize,
    /// 成功任务数
    pub succeeded_tasks: usize,
    /// 失败任务数
    pub failed_tasks: usize,
    /// 跳过任务数
    pub skipped_tasks: usize,
    /// 各任务报告
    pub tasks: Vec<TaskReport>,
    /// 错误摘要
    pub error_summary: Option<String>,
}

/// 单个任务的执行报告
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskReport {
    /// 任务ID
    pub task_id: String,
    /// 任务名称
    pub name: String,
    /// 是否成功
    pub success: bool,
    /// 执行时间（毫秒）
    pub duration_ms: u64,
    /// 错误信息
    pub error: Option<String>,
    /// 输出摘要（截断）
    pub output_summary: Option<String>,
}

/// 报告生成器
pub struct ReportGenerator;

impl ReportGenerator {
    /// 从执行结果生成报告
    pub fn from_execution_result(result: &ExecutionResult) -> ExecutionReport {
        let succeeded = result.node_results.iter().filter(|r| r.success).count();
        let failed = result.node_results.len() - succeeded;

        let tasks: Vec<TaskReport> = result.node_results.iter().map(|r| TaskReport {
            task_id: r.task_id.clone(),
            name: r.name.clone(),
            success: r.success,
            duration_ms: r.duration_ms,
            error: r.error.clone(),
            output_summary: r.output.as_ref().map(|o| {
                let s = o.to_string();
                if s.len() > 200 {
                    format!("{}...", &s[..200])
                } else {
                    s
                }
            }),
        }).collect();

        let error_summary = if failed > 0 {
            let errors: Vec<String> = result.node_results.iter()
                .filter(|r| !r.success)
                .filter_map(|r| r.error.clone())
                .collect();
            Some(format!("{} tasks failed: {}", failed, errors.join("; ")))
        } else {
            None
        };

        ExecutionReport {
            report_id: uuid::Uuid::new_v4().to_string(),
            success: result.success,
            total_duration_ms: result.total_duration_ms,
            start_time: Some(result.start_time),
            end_time: Some(result.end_time),
            total_tasks: result.node_results.len(),
            succeeded_tasks: succeeded,
            failed_tasks: failed,
            skipped_tasks: 0,
            tasks,
            error_summary,
        }
    }

    /// 从节点执行结果列表生成报告
    pub fn from_node_results(results: &[NodeExecutionResult]) -> ExecutionReport {
        let succeeded = results.iter().filter(|r| r.success).count();
        let failed = results.len() - succeeded;
        let total_duration: u64 = results.iter().map(|r| r.duration_ms).sum();

        let now = chrono::Utc::now().timestamp_millis();

        let tasks: Vec<TaskReport> = results.iter().map(|r| TaskReport {
            task_id: r.task_id.clone(),
            name: r.name.clone(),
            success: r.success,
            duration_ms: r.duration_ms,
            error: r.error.clone(),
            output_summary: r.output.as_ref().map(|o| {
                let s = o.to_string();
                if s.len() > 200 { format!("{}...", &s[..200]) } else { s }
            }),
        }).collect();

        let error_summary = if failed > 0 {
            let errors: Vec<String> = results.iter()
                .filter(|r| !r.success)
                .filter_map(|r| r.error.clone())
                .collect();
            Some(format!("{} tasks failed: {}", failed, errors.join("; ")))
        } else {
            None
        };

        ExecutionReport {
            report_id: uuid::Uuid::new_v4().to_string(),
            success: failed == 0,
            total_duration_ms: total_duration,
            start_time: results.first().and_then(|r| r.start_time),
            end_time: results.last().and_then(|r| r.end_time).or(Some(now)),
            total_tasks: results.len(),
            succeeded_tasks: succeeded,
            failed_tasks: failed,
            skipped_tasks: 0,
            tasks,
            error_summary,
        }
    }
}

impl std::fmt::Display for ExecutionReport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "=== Execution Report ===")?;
        writeln!(f, "Report ID: {}", self.report_id)?;
        writeln!(f, "Success: {}", if self.success { "YES" } else { "NO" })?;
        writeln!(f, "Total Duration: {}ms", self.total_duration_ms)?;
        writeln!(f, "Tasks: {} total, {} succeeded, {} failed, {} skipped",
            self.total_tasks, self.succeeded_tasks, self.failed_tasks, self.skipped_tasks)?;
        writeln!(f, "--- Task Details ---")?;
        for task in &self.tasks {
            let status = if task.success { "OK" } else { "FAIL" };
            writeln!(f, "  [{}] {} ({}) - {}ms", status, task.task_id, task.name, task.duration_ms)?;
            if let Some(ref error) = task.error {
                writeln!(f, "    Error: {}", error)?;
            }
        }
        if let Some(ref summary) = self.error_summary {
            writeln!(f, "--- Error Summary ---")?;
            writeln!(f, "{}", summary)?;
        }
        Ok(())
    }
}
