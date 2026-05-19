//! Task 模块测试公共辅助工具

use cortex_flow::task::SimpleTask;

#[path = "../../_common/fixture_loader.rs"]
pub mod fixture_loader;

/// 创建简单的测试任务
pub fn simple_task(id: &str, name: &str) -> SimpleTask {
    SimpleTask::new(id, name, |_input, _ctx| {
        Box::pin(async { Ok(serde_json::json!({"result": "ok"})) })
    })
}

/// 创建带描述的测试任务
pub fn described_task(id: &str, name: &str, desc: &str) -> SimpleTask {
    SimpleTask::with_description(id, name, desc, |_input, _ctx| {
        Box::pin(async { Ok(serde_json::json!({"result": "ok"})) })
    })
}
