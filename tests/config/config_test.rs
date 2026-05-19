//! ConfigManager 和 FrameworkConfig 集成测试
//!
//! 使用 `tests/fixtures/config/` 下的 JSON/TOML fixture 文件作为测试数据。

use crate::common::fixture_loader;
use cortex_flow::config::{ConfigManager, FrameworkConfig};

// ---- ConfigManager tests ----

/// 默认配置应通过验证
#[test]
fn test_default_config_is_valid() {
    let config = ConfigManager::new();
    assert!(config.validate().is_ok());
}

/// 从环境变量创建的配置应通过验证
#[test]
fn test_from_env() {
    let config = ConfigManager::from_env();
    assert!(config.validate().is_ok());
}

/// FrameworkConfig 默认值应通过验证
#[test]
fn test_framework_validate() {
    let config = FrameworkConfig::default();
    assert!(config.validate().is_ok());
}

/// 从 JSON fixture 解析有效配置，验证字段值与通过验证
#[test]
fn test_framework_parse_json() {
    let json = fixture_loader::load_fixture("config/valid_config.json");
    let config = FrameworkConfig::parse(&json);
    assert!(config.is_ok(), "Parse failed: {:?}", config.err());
    let config = config.unwrap();
    assert!(config.validate().is_ok());
    assert_eq!(config.execution.max_workers, 4);
    assert_eq!(config.logging.level, "info");
}

/// 从 TOML fixture 解析有效配置，验证字段值
#[test]
fn test_framework_parse_toml() {
    let toml = fixture_loader::load_fixture("config/valid_config.toml");
    let config = FrameworkConfig::parse(&toml);
    assert!(config.is_ok(), "Parse failed: {:?}", config.err());
    let config = config.unwrap();
    assert_eq!(config.execution.max_workers, 4);
}

/// 无效配置：max_workers=0 应验证失败
#[test]
fn test_framework_parse_invalid_zero_workers() {
    let json = fixture_loader::load_fixture("config/invalid_zero_workers.json");
    let config = FrameworkConfig::parse(&json).unwrap();
    assert!(config.validate().is_err(), "Should fail: max_workers=0");
}

/// 无效配置：default_timeout=0 应验证失败
#[test]
fn test_framework_parse_invalid_zero_timeout() {
    let json = fixture_loader::load_fixture("config/invalid_zero_timeout.json");
    let config = FrameworkConfig::parse(&json).unwrap();
    assert!(config.validate().is_err(), "Should fail: default_timeout=0");
}

/// 边界配置：高并行(16)+debug日志+text格式，验证通过且字段值正确
#[test]
fn test_framework_parse_high_parallelism() {
    let json = fixture_loader::load_fixture("config/high_parallelism.json");
    let config = FrameworkConfig::parse(&json).unwrap();
    assert!(config.validate().is_ok());
    assert_eq!(config.execution.max_workers, 16);
    assert_eq!(config.logging.level, "debug");
    assert_eq!(config.logging.format, "text");
}

// ---- ConfigManager reload tests ----

/// 未设置配置路径时 reload 应返回错误
#[test]
fn test_reload_without_path_fails() {
    let mut config = ConfigManager::new();
    let result = config.reload();
    assert!(result.is_err());
}
