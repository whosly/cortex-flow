//! # 追踪器
//!
//! 提供分布式追踪能力，记录任务执行的调用链。
//!
//! ## 模块概述
//!
//! - [`Span`] - 表示追踪中的一个操作单元
//! - [`Tracer`] - 追踪器 Trait，定义追踪接口
//! - [`TracerImpl`] - 追踪器的内存实现
//!
//! ## 与 tracing crate 集成
//!
//! 当启用 `tracing_enabled` feature 时，追踪信息会同时输出到 tracing 生态。

use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

/// 追踪器 Trait
pub trait Tracer: Send + Sync {
    /// 开始一个 span
    fn start_span(&self, name: &str) -> Span;

    /// 结束一个 span
    fn end_span(&self, span: &mut Span);

    /// 记录事件
    fn event(&self, name: &str, attributes: Option<HashMap<String, String>>);
}

/// 追踪 Span
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Span {
    /// Span ID
    pub span_id: String,
    /// Trace ID
    pub trace_id: String,
    /// Span 名称
    pub name: String,
    /// 开始时间（毫秒时间戳）
    pub start_time: Option<i64>,
    /// 结束时间（毫秒时间戳）
    pub end_time: Option<i64>,
    /// 持续时间（毫秒）
    pub duration_ms: Option<u64>,
    /// Span 属性
    pub attributes: HashMap<String, String>,
    /// Span 状态
    pub status: SpanStatus,
    /// 事件列表
    pub events: Vec<SpanEvent>,
}

/// Span 状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SpanStatus {
    /// 未设置
    Unset,
    /// 正常
    Ok,
    /// 错误
    Error,
}

#[allow(clippy::derivable_impls)]
impl Default for SpanStatus {
    fn default() -> Self {
        Self::Unset
    }
}

/// Span 事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpanEvent {
    /// 事件名称
    pub name: String,
    /// 事件时间戳
    pub timestamp: i64,
    /// 事件属性
    pub attributes: HashMap<String, String>,
}

impl Span {
    /// 创建新 span
    pub fn new(name: impl Into<String>, trace_id: impl Into<String>) -> Self {
        Self {
            span_id: Uuid::new_v4().to_string(),
            trace_id: trace_id.into(),
            name: name.into(),
            start_time: Some(chrono::Utc::now().timestamp_millis()),
            end_time: None,
            duration_ms: None,
            attributes: HashMap::new(),
            status: SpanStatus::Unset,
            events: Vec::new(),
        }
    }

    /// 设置属性
    pub fn set_attribute(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.attributes.insert(key.into(), value.into());
    }

    /// 添加事件
    pub fn add_event(&mut self, name: impl Into<String>, attributes: HashMap<String, String>) {
        self.events.push(SpanEvent {
            name: name.into(),
            timestamp: chrono::Utc::now().timestamp_millis(),
            attributes,
        });
    }

    /// 标记为成功
    pub fn ok(&mut self) {
        self.status = SpanStatus::Ok;
    }

    /// 标记为错误
    pub fn error(&mut self, message: impl Into<String>) {
        self.status = SpanStatus::Error;
        self.set_attribute("error", message);
    }

    /// 结束 span
    pub fn end(&mut self) {
        let now = chrono::Utc::now().timestamp_millis();
        self.end_time = Some(now);
        if let Some(start) = self.start_time {
            self.duration_ms = Some((now - start) as u64);
        }
    }

    /// 是否已完成
    pub fn is_finished(&self) -> bool {
        self.end_time.is_some()
    }
}

/// 追踪器内存实现
#[derive(Default, Clone)]
pub struct TracerImpl {
    spans: Arc<RwLock<Vec<Span>>>,
    current_trace_id: Arc<RwLock<Option<String>>>,
}

impl TracerImpl {
    /// 创建新的追踪器
    pub fn new() -> Self {
        Self::default()
    }

    /// 开始一个 span（返回新 span）
    pub fn create_span(&self, name: impl Into<String>) -> Span {
        let trace_id = self
            .current_trace_id
            .read()
            .clone()
            .unwrap_or_else(|| Uuid::new_v4().to_string());
        Span::new(name, trace_id)
    }

    /// 开始一个 span 并记录到追踪器
    pub fn start_span_and_record(&self, name: &str) -> Span {
        let span = self.create_span(name);
        // 记录到 tracing（如果启用）
        #[cfg(feature = "tracing_enabled")]
        {
            tracing::info!(span_name = name, span_id = %span.span_id, "Span started");
        }
        span
    }

    /// 结束 span 并记录
    pub fn finish_span(&self, span: &mut Span) {
        span.end();
        let mut spans = self.spans.write();
        spans.push(span.clone());

        #[cfg(feature = "tracing_enabled")]
        {
            tracing::info!(
                span_name = %span.name,
                span_id = %span.span_id,
                duration_ms = span.duration_ms.unwrap_or(0),
                status = ?span.status,
                "Span finished"
            );
        }
    }

    /// 记录事件
    pub fn record_event(
        &self,
        name: impl Into<String>,
        attributes: Option<HashMap<String, String>>,
    ) {
        #[cfg(feature = "tracing_enabled")]
        {
            match &attributes {
                Some(attrs) => {
                    tracing::info!(event_name = %name.into(), ?attrs, "Tracing event");
                }
                None => {
                    tracing::info!(event_name = %name.into(), "Tracing event");
                }
            }
        }
        let _ = (name, attributes);
    }

    /// 设置当前 trace ID
    pub fn set_trace_id(&self, trace_id: impl Into<String>) {
        *self.current_trace_id.write() = Some(trace_id.into());
    }

    /// 获取所有已记录的 spans
    pub fn spans(&self) -> Vec<Span> {
        self.spans.read().clone()
    }

    /// 获取指定名称的 spans
    pub fn spans_by_name(&self, name: &str) -> Vec<Span> {
        self.spans
            .read()
            .iter()
            .filter(|s| s.name == name)
            .cloned()
            .collect()
    }

    /// 清除所有 spans
    pub fn clear(&self) {
        self.spans.write().clear();
    }

    /// 获取 span 总数
    pub fn span_count(&self) -> usize {
        self.spans.read().len()
    }

    /// 生成追踪报告
    pub fn report(&self) -> TraceReport {
        let spans = self.spans.read();
        let total_spans = spans.len();
        let error_spans = spans
            .iter()
            .filter(|s| s.status == SpanStatus::Error)
            .count();
        let total_duration_ms: u64 = spans.iter().filter_map(|s| s.duration_ms).sum();

        TraceReport {
            total_spans,
            error_spans,
            total_duration_ms,
            spans: spans.clone(),
        }
    }
}

/// 追踪报告
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceReport {
    /// 总 span 数
    pub total_spans: usize,
    /// 错误 span 数
    pub error_spans: usize,
    /// 总持续时间（毫秒）
    pub total_duration_ms: u64,
    /// 所有 spans
    pub spans: Vec<Span>,
}

impl std::fmt::Display for TraceReport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "=== Trace Report ===")?;
        writeln!(f, "Total Spans: {}", self.total_spans)?;
        writeln!(f, "Error Spans: {}", self.error_spans)?;
        writeln!(f, "Total Duration: {}ms", self.total_duration_ms)?;
        writeln!(f, "--- Spans ---")?;
        for span in &self.spans {
            let status = match span.status {
                SpanStatus::Ok => "OK",
                SpanStatus::Error => "ERROR",
                SpanStatus::Unset => "UNSET",
            };
            writeln!(
                f,
                "  [{}] {} ({}ms) {}",
                status,
                span.name,
                span.duration_ms.unwrap_or(0),
                span.span_id.chars().take(8).collect::<String>()
            )?;
        }
        Ok(())
    }
}

impl Tracer for TracerImpl {
    fn start_span(&self, name: &str) -> Span {
        self.start_span_and_record(name)
    }

    fn end_span(&self, span: &mut Span) {
        self.finish_span(span)
    }

    fn event(&self, name: &str, attributes: Option<HashMap<String, String>>) {
        self.record_event(name, attributes);
    }
}
