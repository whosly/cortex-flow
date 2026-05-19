//! # LLM配置

use serde::{Deserialize, Serialize};

/// LLM服务提供商
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LLMProvider {
    OpenAI,
    Anthropic,
    Azure,
    Ollama,
    Custom,
}

#[allow(clippy::derivable_impls)]
impl Default for LLMProvider {
    fn default() -> Self {
        Self::OpenAI
    }
}

impl std::fmt::Display for LLMProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LLMProvider::OpenAI => write!(f, "openai"),
            LLMProvider::Anthropic => write!(f, "anthropic"),
            LLMProvider::Azure => write!(f, "azure"),
            LLMProvider::Ollama => write!(f, "ollama"),
            LLMProvider::Custom => write!(f, "custom"),
        }
    }
}

/// LLM调用配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMConfig {
    /// 提供商
    pub provider: LLMProvider,
    /// API密钥
    pub api_key: String,
    /// 模型名称
    pub model: String,
    /// 自定义API地址
    pub base_url: Option<String>,
    /// 最大token数
    pub max_tokens: Option<u32>,
    /// 温度参数
    pub temperature: Option<f32>,
    /// 请求超时秒数
    pub timeout_secs: Option<u64>,
    /// 最大重试次数
    pub max_retries: Option<u32>,
}

impl LLMConfig {
    /// 创建OpenAI配置
    pub fn openai(api_key: impl Into<String>, model: impl Into<String>) -> Self {
        Self {
            provider: LLMProvider::OpenAI,
            api_key: api_key.into(),
            model: model.into(),
            base_url: None,
            max_tokens: None,
            temperature: None,
            timeout_secs: Some(60),
            max_retries: Some(3),
        }
    }

    /// 创建Anthropic配置
    pub fn anthropic(api_key: impl Into<String>, model: impl Into<String>) -> Self {
        Self {
            provider: LLMProvider::Anthropic,
            api_key: api_key.into(),
            model: model.into(),
            base_url: None,
            max_tokens: None,
            temperature: None,
            timeout_secs: Some(60),
            max_retries: Some(3),
        }
    }

    /// 创建Ollama配置（本地模型）
    pub fn ollama(model: impl Into<String>) -> Self {
        Self {
            provider: LLMProvider::Ollama,
            api_key: String::new(),
            model: model.into(),
            base_url: Some("http://localhost:11434".to_string()),
            max_tokens: None,
            temperature: None,
            timeout_secs: Some(120),
            max_retries: Some(1),
        }
    }

    /// 创建Azure OpenAI配置
    pub fn azure(
        api_key: impl Into<String>,
        endpoint: impl Into<String>,
        deployment: impl Into<String>,
    ) -> Self {
        Self {
            provider: LLMProvider::Azure,
            api_key: api_key.into(),
            model: deployment.into(),
            base_url: Some(endpoint.into()),
            max_tokens: None,
            temperature: None,
            timeout_secs: Some(60),
            max_retries: Some(3),
        }
    }

    /// 创建自定义 OpenAI 兼容 API 配置
    ///
    /// 适用于任何兼容 OpenAI Chat Completions API 的服务，例如：
    /// - 火山引擎豆包 (Doubao)
    /// - DeepSeek
    /// - Moonshot (Kimi)
    /// - 智谱 AI (GLM)
    /// - 其他 OpenAI 兼容的推理服务
    ///
    /// # 参数
    ///
    /// - `api_key`: API 密钥
    /// - `model`: 模型名称（如 "doubao-pro-32k"、"deepseek-chat"）
    /// - `base_url`: API 基础地址（如 "https://ark.cn-beijing.volces.com/api/v3"）
    ///
    /// # 示例
    ///
    /// ```rust
    /// use cortex_flow::llm::LLMConfig;
    ///
    /// // 火山引擎豆包
    /// let config = LLMConfig::custom(
    ///     "your-api-key",
    ///     "doubao-pro-32k",
    ///     "https://ark.cn-beijing.volces.com/api/v3",
    /// );
    ///
    /// // DeepSeek
    /// let config = LLMConfig::custom(
    ///     "your-api-key",
    ///     "deepseek-chat",
    ///     "https://api.deepseek.com/v1",
    /// );
    /// ```
    pub fn custom(
        api_key: impl Into<String>,
        model: impl Into<String>,
        base_url: impl Into<String>,
    ) -> Self {
        Self {
            provider: LLMProvider::Custom,
            api_key: api_key.into(),
            model: model.into(),
            base_url: Some(base_url.into()),
            max_tokens: None,
            temperature: None,
            timeout_secs: Some(60),
            max_retries: Some(3),
        }
    }

    /// 设置最大token数
    pub fn with_max_tokens(mut self, max_tokens: u32) -> Self {
        self.max_tokens = Some(max_tokens);
        self
    }

    /// 设置温度参数
    pub fn with_temperature(mut self, temperature: f32) -> Self {
        self.temperature = Some(temperature);
        self
    }

    /// 设置超时时间
    pub fn with_timeout(mut self, secs: u64) -> Self {
        self.timeout_secs = Some(secs);
        self
    }

    /// 设置最大重试次数
    pub fn with_max_retries(mut self, retries: u32) -> Self {
        self.max_retries = Some(retries);
        self
    }

    /// 从环境变量设置API Key
    pub fn with_env_api_key(mut self, env_var: &str) -> Self {
        if let Ok(key) = std::env::var(env_var) {
            self.api_key = key;
        }
        self
    }

    /// 设置自定义API地址
    pub fn with_base_url(mut self, url: impl Into<String>) -> Self {
        self.base_url = Some(url.into());
        self
    }

    /// 获取有效超时时间（秒）
    pub fn timeout(&self) -> u64 {
        self.timeout_secs.unwrap_or(60)
    }

    /// 获取有效最大重试次数
    pub fn retries(&self) -> u32 {
        self.max_retries.unwrap_or(3)
    }
}

impl Default for LLMConfig {
    fn default() -> Self {
        Self::openai(std::env::var("OPENAI_API_KEY").unwrap_or_default(), "gpt-4")
    }
}
