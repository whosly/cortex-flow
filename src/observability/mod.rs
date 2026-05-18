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

pub use tracer::{Tracer, TracerImpl, Span, SpanStatus, SpanEvent, TraceReport};
pub use metrics::{MetricsCollector, Metric, MetricType, MetricValue};

mod tracer;
mod metrics;
