//! # 可观测性模块
//!
//! 提供追踪和指标收集能力。
//!
//! ## 模块概述
//!
//! - [`Tracer`] / [`TracerImpl`] - 分布式追踪
//! - [`Span`] / [`SpanStatus`] - 追踪 Span
//! - [`TraceReport`] - 追踪报告
//! - [`MetricsCollector`] / [`Metric`] - 指标收集

pub use metrics::{Metric, MetricType, MetricValue, MetricsCollector};
pub use tracer::{Span, SpanEvent, SpanStatus, TraceReport, Tracer, TracerImpl};

mod metrics;
mod tracer;
