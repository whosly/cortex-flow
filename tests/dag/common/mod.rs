//! DAG 模块测试公共辅助工具
//!
//! 提供各子模块共享的 DAG 构建工具函数。

use cortex_flow::dag::DAGBuilder;
use cortex_flow::task::SimpleTask;

/// 创建一个简单的成功任务
pub fn simple_task(id: &str, name: &str) -> SimpleTask {
    SimpleTask::new(id, name, |_ctx, _input| {
        Box::pin(async move { Ok(serde_json::json!({"result": "ok"})) })
    })
}

/// 构建线性 DAG：a → b → c
pub fn build_linear_dag() -> cortex_flow::dag::DAG {
    let mut builder = DAGBuilder::new();
    builder.add_node(simple_task("a", "Task A"));
    builder.add_node(simple_task("b", "Task B"));
    builder.add_node(simple_task("c", "Task C"));
    builder.add_dependency("a", "b");
    builder.add_dependency("b", "c");
    builder.build().unwrap()
}

/// 构建菱形 DAG：
/// ```text
///     root
///     / \
///   left right
///     \ /
///    sink
/// ```
pub fn build_diamond_dag() -> cortex_flow::dag::DAG {
    let mut builder = DAGBuilder::new();
    builder.add_node(simple_task("root", "Root Task"));
    builder.add_node(simple_task("left", "Left Branch"));
    builder.add_node(simple_task("right", "Right Branch"));
    builder.add_node(simple_task("sink", "Sink Task"));
    builder.add_dependency("root", "left");
    builder.add_dependency("root", "right");
    builder.add_dependency("left", "sink");
    builder.add_dependency("right", "sink");
    builder.build().unwrap()
}

/// 构建单节点 DAG：only
pub fn build_single_node_dag() -> cortex_flow::dag::DAG {
    let mut builder = DAGBuilder::new();
    builder.add_node(simple_task("only", "Only Task"));
    builder.build().unwrap()
}
