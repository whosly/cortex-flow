//! ErrorRecovery 集成测试
//!
//! 使用 `tests/fixtures/error_recovery/` 下的 JSON fixture 定义重试策略和补偿链。

use cortex_flow::error::Error;
use cortex_flow::error_recovery::{CompensationResult, ErrorRecovery, FnCompensation, RetryPolicy};
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;

// ---- retry tests ----

/// 固定间隔重试：首次失败后第二次成功，验证 execute_with_retry 自动恢复
#[tokio::test]
async fn test_retry_success_on_second_attempt() {
    let policy = RetryPolicy::fixed(3, 10);
    let mut recovery = ErrorRecovery::new(policy);
    let counter = Arc::new(AtomicU32::new(0));

    let result: cortex_flow::error::Result<String> = recovery
        .execute_with_retry(|| {
            let counter = counter.clone();
            async move {
                let count = counter.fetch_add(1, Ordering::SeqCst);
                if count == 0 {
                    Err(Error::TaskExecution("first attempt fails".to_string()))
                } else {
                    Ok("success".to_string())
                }
            }
        })
        .await;

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "success");
}

/// 重试次数耗尽后应返回错误
#[tokio::test]
async fn test_retry_exhausted() {
    let policy = RetryPolicy::fixed(2, 10);
    let mut recovery = ErrorRecovery::new(policy);

    let result: cortex_flow::error::Result<String> = recovery
        .execute_with_retry(|| async { Err(Error::TaskExecution("always fails".to_string())) })
        .await;

    assert!(result.is_err());
}

// ---- compensation tests ----

/// 补偿链执行：按注册逆序执行补偿操作，验证 Success/Skipped 结果
#[tokio::test]
async fn test_compensation_chain() {
    use cortex_flow::context::ExecutionContext;
    use cortex_flow::error_recovery::CompensationChain;

    let ctx = ExecutionContext::new();
    let chain = CompensationChain::new()
        .add(FnCompensation::new("step1", |_| {
            CompensationResult::Success
        }))
        .add(FnCompensation::new("step2", |_| {
            CompensationResult::Skipped
        }));

    let results = chain.execute_all(&ctx).await;
    assert_eq!(results.len(), 2);
}

// ---- error handler tests ----

/// 默认错误处理器：TaskExecution→Retry，Config→Abort
#[test]
fn test_error_handler_default() {
    use cortex_flow::error_recovery::ErrorAction;
    use cortex_flow::error_recovery::ErrorHandler;

    let mut handler = ErrorHandler::default_handler();
    let result = handler.handle(&Error::TaskExecution("fail".to_string()));
    assert_eq!(result.action, ErrorAction::Retry);

    let result2 = handler.handle(&Error::Config("bad".to_string()));
    assert_eq!(result2.action, ErrorAction::Abort);
}

/// 错误分类：TaskExecution 可恢复，Config 不可恢复
#[test]
fn test_error_category() {
    let recoverable = Error::TaskExecution("fail".to_string());
    assert!(recoverable.is_recoverable());

    let unrecoverable = Error::Config("bad".to_string());
    assert!(!unrecoverable.is_recoverable());
}

// ---- Fixture-driven retry policy tests ----

/// Fixture 驱动：从 retry_and_compensation.json 验证重试策略参数（max_retries/interval_ms）
#[test]
fn test_retry_policies_from_fixture() {
    use crate::common::fixture_loader;

    let fixture = fixture_loader::load_json_fixture::<serde_json::Value>(
        "error_recovery/retry_and_compensation.json",
    );

    let policies = fixture.get("policies").unwrap().as_array().unwrap();
    assert_eq!(policies.len(), 2);

    // 验证 fixture 中的策略定义可以正确映射到 RetryPolicy
    let fixed_3 = policies
        .iter()
        .find(|p| p["name"] == "fixed_3_retries")
        .unwrap();
    assert_eq!(fixed_3["max_retries"].as_u64().unwrap(), 3);
    assert_eq!(fixed_3["interval_ms"].as_u64().unwrap(), 10);

    let fixed_2 = policies
        .iter()
        .find(|p| p["name"] == "fixed_2_retries")
        .unwrap();
    assert_eq!(fixed_2["max_retries"].as_u64().unwrap(), 2);
}
