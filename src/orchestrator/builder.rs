//! # Orchestrator 构建器
//!
//! 提供 Orchestrator 的构建器模式实现。

use super::Orchestrator;
use crate::config::{ConfigManager, FrameworkConfig};
use crate::error::Result;
use crate::llm::config::LLMConfig;
use crate::llm::{LLMClient, LLMClientTrait};
use crate::observability::{MetricsCollector, TracerImpl};
use crate::strategy::{StrategyConfig, StrategyRegistry};
use std::sync::Arc;

/// Orchestrator 构建器
///
/// 使用构建器模式创建 Orchestrator 实例。
///
/// # 示例
///
/// ```rust,ignore
/// let orchestrator = Orchestrator::builder()
///     .with_max_parallelism(4)
///     .with_llm_config(llm_config)
///     .build()
///     .await?;
/// ```
pub struct OrchestratorBuilder {
    /// 最大并发数
    max_parallelism: usize,
    /// 策略配置
    strategy_config: StrategyConfig,
    /// 框架配置
    framework_config: Option<FrameworkConfig>,
    /// LLM配置
    llm_config: Option<LLMConfig>,
    /// 是否启用追踪
    enable_tracing: bool,
    /// 是否启用指标
    enable_metrics: bool,
}

impl OrchestratorBuilder {
    /// 创建新的构建器
    pub fn new() -> Self {
        Self {
            max_parallelism: 10,
            strategy_config: StrategyConfig::default(),
            framework_config: None,
            llm_config: None,
            enable_tracing: true,
            enable_metrics: true,
        }
    }

    /// 设置最大并发数
    pub fn with_max_parallelism(mut self, parallelism: usize) -> Self {
        self.max_parallelism = parallelism;
        self.strategy_config = self.strategy_config.with_parallelism(parallelism);
        self
    }

    /// 设置策略配置
    pub fn with_strategy_config(mut self, config: StrategyConfig) -> Self {
        self.strategy_config = config;
        self
    }

    /// 设置框架配置
    pub fn with_framework_config(mut self, config: FrameworkConfig) -> Self {
        self.framework_config = Some(config);
        self
    }

    /// 设置 LLM 配置
    pub fn with_llm_config(mut self, config: LLMConfig) -> Self {
        self.llm_config = Some(config);
        self
    }

    /// 设置超时时间（毫秒）
    pub fn with_timeout(mut self, timeout_ms: u64) -> Self {
        self.strategy_config = self.strategy_config.with_timeout(timeout_ms);
        self
    }

    /// 设置是否启用追踪
    pub fn with_tracing(mut self, enable: bool) -> Self {
        self.enable_tracing = enable;
        self
    }

    /// 设置是否启用指标收集
    pub fn with_metrics(mut self, enable: bool) -> Self {
        self.enable_metrics = enable;
        self
    }

    /// 构建 Orchestrator 实例
    pub async fn build(self) -> Result<Orchestrator> {
        // 创建配置管理器
        let mut config = ConfigManager::new();
        if let Some(fw_config) = self.framework_config {
            *config.framework_mut() = fw_config;
        }

        // 创建追踪器
        let tracer = TracerImpl::new();

        // 创建指标收集器
        let metrics = MetricsCollector::new();

        // 创建 LLM 客户端（如果有配置）
        let llm_client = self
            .llm_config
            .map(|cfg| Arc::new(LLMClient::new(cfg)) as Arc<dyn LLMClientTrait>);

        // 创建策略注册表
        let strategy_registry = StrategyRegistry::new();

        Ok(Orchestrator {
            config,
            tracer,
            metrics,
            llm_client,
            strategy_registry,
            max_parallelism: self.max_parallelism,
        })
    }
}

impl Default for OrchestratorBuilder {
    fn default() -> Self {
        Self::new()
    }
}
