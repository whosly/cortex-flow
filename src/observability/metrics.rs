//! # 指标收集器

use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::RwLock;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MetricType {
    Counter,
    Gauge,
    Histogram,
}

#[derive(Debug, Clone)]
pub struct MetricValue {
    pub value: f64,
    pub labels: HashMap<String, String>,
}

impl MetricValue {
    pub fn new(value: f64) -> Self {
        Self { value, labels: HashMap::new() }
    }
}

#[derive(Debug, Clone)]
pub struct Metric {
    pub name: String,
    pub description: String,
    pub metric_type: MetricType,
    pub value: MetricValue,
}

impl Metric {
    pub fn counter(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            metric_type: MetricType::Counter,
            value: MetricValue::new(0.0),
        }
    }

    pub fn gauge(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            metric_type: MetricType::Gauge,
            value: MetricValue::new(0.0),
        }
    }

    pub fn record(&mut self, value: f64) {
        match self.metric_type {
            MetricType::Counter => self.value.value += value,
            MetricType::Gauge | MetricType::Histogram => self.value.value = value,
        }
    }
}

#[derive(Default, Clone)]
pub struct MetricsCollector {
    metrics: Arc<RwLock<HashMap<String, Metric>>>,
}

impl MetricsCollector {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(&self, metric: Metric) {
        let mut metrics = self.metrics.write();
        metrics.insert(metric.name.clone(), metric);
    }

    pub fn get(&self, name: &str) -> Option<Metric> {
        let metrics = self.metrics.read();
        metrics.get(name).cloned()
    }

    pub fn record(&self, name: &str, value: f64) {
        let mut metrics = self.metrics.write();
        if let Some(metric) = metrics.get_mut(name) {
            metric.record(value);
        }
    }

    pub fn increment(&self, name: &str) {
        self.record(name, 1.0);
    }

    pub fn all(&self) -> Vec<Metric> {
        let metrics = self.metrics.read();
        metrics.values().cloned().collect()
    }

    pub fn clear(&self) {
        let mut metrics = self.metrics.write();
        metrics.clear();
    }
}
