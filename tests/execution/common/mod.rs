//! 执行引擎模块测试公共辅助工具

use cortex_flow::dag::DAGBuilder;
use cortex_flow::task::SimpleTask;
use serde_json::json;

pub fn simple_task(id: &str, name: &str) -> SimpleTask {
    SimpleTask::new(id, name, |_input, _ctx| {
        Box::pin(async move { Ok(json!({"result": "ok"})) })
    })
}

pub fn build_linear_dag() -> cortex_flow::dag::DAG {
    let mut builder = DAGBuilder::new();
    builder
        .add_node(simple_task("a", "Task A"))
        .add_node(simple_task("b", "Task B"))
        .add_dependency("a", "b");
    builder.build().unwrap()
}
