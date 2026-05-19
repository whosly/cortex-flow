//! DAG 模块测试公共辅助工具
//!
//! 提供各子模块共享的 DAG 构建工具函数和 fixture 加载。

use cortex_flow::dag::DAGBuilder;
use cortex_flow::task::SimpleTask;

#[path = "../../_common/fixture_loader.rs"]
pub mod fixture_loader;

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

/// 从 DAG fixture 构建真实的 DAG 实例
///
/// 读取 JSON fixture 中定义的节点和边，构建对应的 DAG 对象。
pub fn build_dag_from_fixture(fixture: &fixture_loader::DAGFixture) -> cortex_flow::dag::DAG {
    use cortex_flow::dag::DAGEdge;
    let mut builder = DAGBuilder::new();
    for node in &fixture.nodes {
        builder.add_node(simple_task(&node.id, &node.name));
    }
    for edge in &fixture.edges {
        if let Some(ref label) = edge.label {
            builder.add_edge(DAGEdge::new(&edge.from, &edge.to).with_label(label));
        } else {
            builder.add_dependency(&edge.from, &edge.to);
        }
    }
    builder.build().unwrap()
}
