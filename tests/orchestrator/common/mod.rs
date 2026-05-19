//! 编排器模块测试公共辅助工具

use cortex_flow::task::SimpleTask;
use serde_json::json;

pub fn simple_task(id: &str, name: &str) -> SimpleTask {
    SimpleTask::new(id, name, |_input, _ctx| {
        Box::pin(async move { Ok(json!({"result": "ok"})) })
    })
}

#[allow(dead_code)]
pub fn counting_task(id: &str, name: &str) -> SimpleTask {
    SimpleTask::new(id, name, |input, _ctx| {
        Box::pin(async move {
            let count = input.get("count").and_then(|v| v.as_u64()).unwrap_or(0);
            Ok(json!({"count": count + 1}))
        })
    })
}
