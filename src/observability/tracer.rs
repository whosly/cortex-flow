//! # 追踪器
//!
//! 提供分布式追踪能力，记录任务执行的调用链。
//!
//! ## 模块概述
//!
//! 追踪器实现了基本的分布式追踪功能：
//!
//! - **Span**: 表示追踪中的一个操作单元
//! - **Tracer**: 追踪器 Trait，定义追踪接口
//! - **TracerImpl**: 追踪器的内存实现

use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::RwLock;
use uuid::Uuid;

pub trait Tracer: Send + Sync {
    fn start_span(&self, name: &str) -> Span;
    fn event(&self, name: &str, attributes: Option<HashMap<String, String>>);
}

#[derive(Debug, Clone)]
pub struct Span {
    pub span_id: String,
    pub trace_id: String,
    pub name: String,
    pub attributes: HashMap<String, String>,
}

#[derive(Default, Clone)]
pub struct TracerImpl {
    spans: Arc<RwLock<Vec<Span>>>,
}

impl TracerImpl {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn start_span(&self, name: impl Into<String>) -> Span {
        Span {
            span_id: Uuid::new_v4().to_string(),
            trace_id: Uuid::new_v4().to_string(),
            name: name.into(),
            attributes: HashMap::new(),
        }
    }

    pub fn event(&self, name: impl Into<String>, attributes: Option<HashMap<String, String>>) {
        let _ = attributes;
    }

    pub fn spans(&self) -> Vec<Span> {
        self.spans.read().clone()
    }

    pub fn clear(&self) {
        self.spans.write().clear();
    }
}

impl Tracer for TracerImpl {
    fn start_span(&self, name: &str) -> Span {
        self.start_span(name)
    }

    fn event(&self, name: &str, attributes: Option<HashMap<String, String>>) {
        self.event(name, attributes);
    }
}
