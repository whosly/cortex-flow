//! # 错误处理器

use crate::error::Error;
use serde::{Deserialize, Serialize};

/// 错误处理动作
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ErrorAction {
    /// 重试执行
    Retry,
    /// 跳过任务，标记为成功
    Skip,
    /// 终止整个工作流
    Abort,
    /// 执行补偿操作
    Compensate,
    /// 忽略错误，继续执行
    Ignore,
}

/// 错误处理结果
#[derive(Debug, Clone)]
pub struct ErrorHandleResult {
    /// 要执行的动作
    pub action: ErrorAction,
    /// 错误分类
    pub category: ErrorCategory,
    /// 错误消息
    pub message: String,
}

/// 错误分类
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ErrorCategory {
    /// 可恢复错误（网络超时、临时故障等）
    Recoverable,
    /// 不可恢复错误（配置错误、数据格式错误等）
    Unrecoverable,
    /// 部分可恢复（某些条件下可重试）
    PartiallyRecoverable,
}

/// 错误处理器
///
/// 根据错误类型决定如何处理执行错误。
pub struct ErrorHandler {
    /// 默认错误动作
    default_action: ErrorAction,
    /// 错误码到动作的映射
    action_map: std::collections::HashMap<String, ErrorAction>,
    /// 最大连续错误数
    max_consecutive_errors: u32,
    /// 当前连续错误数
    consecutive_errors: u32,
}

impl ErrorHandler {
    /// 创建新的错误处理器
    pub fn new(default_action: ErrorAction) -> Self {
        Self {
            default_action,
            action_map: std::collections::HashMap::new(),
            max_consecutive_errors: 10,
            consecutive_errors: 0,
        }
    }

    /// 默认处理器：可恢复错误重试，不可恢复错误终止
    pub fn default_handler() -> Self {
        let mut handler = Self::new(ErrorAction::Retry);
        handler.register_action("DAG_001", ErrorAction::Abort);
        handler.register_action("DAG_002", ErrorAction::Abort);
        handler.register_action("CFG_001", ErrorAction::Abort);
        handler.register_action("VAL_001", ErrorAction::Abort);
        handler.register_action("TASK_001", ErrorAction::Retry);
        handler.register_action("LLM_001", ErrorAction::Retry);
        handler.register_action("TMO_001", ErrorAction::Retry);
        handler
    }

    /// 注册错误码对应的动作
    pub fn register_action(&mut self, error_code: impl Into<String>, action: ErrorAction) {
        self.action_map.insert(error_code.into(), action);
    }

    /// 设置最大连续错误数
    pub fn with_max_consecutive_errors(mut self, max: u32) -> Self {
        self.max_consecutive_errors = max;
        self
    }

    /// 处理错误
    pub fn handle(&mut self, error: &Error) -> ErrorHandleResult {
        self.consecutive_errors += 1;

        let category = Self::categorize(error);
        let error_code = error.error_code();

        // 检查连续错误是否超过阈值
        if self.consecutive_errors > self.max_consecutive_errors {
            return ErrorHandleResult {
                action: ErrorAction::Abort,
                category: ErrorCategory::Unrecoverable,
                message: format!(
                    "Too many consecutive errors ({}), aborting",
                    self.consecutive_errors
                ),
            };
        }

        // 查找注册的动作
        let action = self
            .action_map
            .get(error_code)
            .cloned()
            .unwrap_or_else(|| self.default_action.clone());

        ErrorHandleResult {
            action,
            category,
            message: error.to_string(),
        }
    }

    /// 对错误进行分类
    pub fn categorize(error: &Error) -> ErrorCategory {
        if error.is_recoverable() {
            ErrorCategory::Recoverable
        } else {
            match error {
                Error::TaskExecution(_) | Error::LLMCall(_) => ErrorCategory::PartiallyRecoverable,
                _ => ErrorCategory::Unrecoverable,
            }
        }
    }

    /// 重置连续错误计数
    pub fn reset_consecutive_errors(&mut self) {
        self.consecutive_errors = 0;
    }

    /// 获取当前连续错误数
    pub fn consecutive_error_count(&self) -> u32 {
        self.consecutive_errors
    }
}

impl Default for ErrorHandler {
    fn default() -> Self {
        Self::default_handler()
    }
}
