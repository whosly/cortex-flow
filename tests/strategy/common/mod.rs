//! 策略模块测试公共辅助工具

use cortex_flow::dag::DAGBuilder;
use cortex_flow::task::SimpleTask;
use serde_json::json;

#[path = "../../_common/fixture_loader.rs"]
pub mod fixture_loader;

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
        .add_node(simple_task("c", "Task C"))
        .add_dependency("a", "b")
        .add_dependency("b", "c");
    builder.build().unwrap()
}

pub fn build_diamond_dag() -> cortex_flow::dag::DAG {
    let mut builder = DAGBuilder::new();
    builder
        .add_node(simple_task("a", "Task A"))
        .add_node(simple_task("b", "Task B"))
        .add_node(simple_task("c", "Task C"))
        .add_node(simple_task("d", "Task D"))
        .add_dependency("a", "b")
        .add_dependency("a", "c")
        .add_dependency("b", "d")
        .add_dependency("c", "d");
    builder.build().unwrap()
}
