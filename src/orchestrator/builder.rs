//! # Orchestrator 构建器
//!
//! 提供 Orchestrator 的构建器模式实现。

use super::Orchestrator;
use crate::config::{ConfigManager, FrameworkConfig};
use crate::error::Result;
use crate::llm::config::LLMConfig;
use crate::llm::{LLMClient, LLMClientRegistry, LLMClientTrait};
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
/// // 单模型配置（向后兼容）
/// let orchestrator = Orchestrator::builder()
///     .with_max_parallelism(4)
///     .with_llm_config(llm_config)
///     .build()
///     .await?;
///
/// // 多模型配置（推荐）
/// let orchestrator = Orchestrator::builder()
///     .with_max_parallelism(4)
///     .with_llm("gpt4", LLMConfig::openai("sk-xxx", "gpt-4"))
///     .with_llm("llama3", LLMConfig::ollama("llama3"))
///     .with_default_llm("gpt4")
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
    /// LLM配置（单模型，向后兼容）
    llm_config: Option<LLMConfig>,
    /// LLM 注册表配置（多模型）
    llm_configs: Vec<(String, LLMConfig)>,
    /// 默认 LLM 名称
    default_llm_name: Option<String>,
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
            llm_configs: Vec::new(),
            default_llm_name: None,
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

    /// 设置 LLM 配置（单模型，向后兼容）
    ///
    /// 如果同时使用了 `with_llm()`，此方法设置的客户端会以 "default" 名称
    /// 注册到注册表中。
    pub fn with_llm_config(mut self, config: LLMConfig) -> Self {
        self.llm_config = Some(config);
        self
    }

    /// 注册命名的 LLM 配置（多模型支持）
    ///
    /// 可以多次调用以注册多个模型。
    ///
    /// # 参数
    ///
    /// - `name`: 客户端名称（如 "gpt4"、"llama3"、"doubao"）
    /// - `config`: LLM 配置
    pub fn with_llm(mut self, name: impl Into<String>, config: LLMConfig) -> Self {
        self.llm_configs.push((name.into(), config));
        self
    }

    /// 设置默认 LLM 名称
    ///
    /// 必须是通过 `with_llm()` 注册的名称。
    /// 如果不设置，第一个注册的客户端会成为默认。
    pub fn with_default_llm(mut self, name: impl Into<String>) -> Self {
        self.default_llm_name = Some(name.into());
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

        // 创建 LLM 客户端注册表
        let llm_registry = LLMClientRegistry::new();

        // 注册多模型配置
        for (name, cfg) in &self.llm_configs {
            llm_registry.register_with_config(name, cfg.clone());
        }

        // 设置默认 LLM（如果指定）
        if let Some(ref default_name) = self.default_llm_name {
            llm_registry.set_default(default_name);
        }

        // 创建单模型客户端（向后兼容）
        let llm_client = self
            .llm_config
            .map(|cfg| Arc::new(LLMClient::new(cfg)) as Arc<dyn LLMClientTrait>);

        // 如果有单模型配置但注册表为空，将其注册为 "default"
        if llm_client.is_some() && llm_registry.is_empty() {
            if let Some(ref client) = llm_client {
                llm_registry.register("default", client.clone());
            }
        }

        // 创建策略注册表
        let strategy_registry = StrategyRegistry::new();

        Ok(Orchestrator {
            config,
            tracer,
            metrics,
            llm_client,
            llm_registry,
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
