//! DAG 可视化导出测试
//!
//! 对应 `src/dag/dag_impl.rs` 中 `DAG::to_dot / to_mermaid / to_ascii` 方法。
//! 使用 `tests/fixtures/dag/` 下的 JSON fixture 定义 DAG 拓扑。

use crate::common;
use crate::common::fixture_loader;
use cortex_flow::dag::DAGEdge;

// ---- to_dot tests ----

/// 线性 DAG 的 DOT 输出格式，验证节点、边、rankdir 及首尾格式
#[test]
fn test_to_dot_linear_dag() {
    let dag = common::build_linear_dag();
    let dot = dag.to_dot();

    assert!(dot.starts_with("digraph DAG {"));
    assert!(dot.contains("rankdir=TB"));
    assert!(dot.contains("\"a\""));
    assert!(dot.contains("\"b\""));
    assert!(dot.contains("\"c\""));
    assert!(dot.contains("\"a\" -> \"b\""));
    assert!(dot.contains("\"b\" -> \"c\""));
    assert!(dot.ends_with("}\n"));
}

/// 菱形 DAG 的 DOT 输出，验证根节点椭圆形状、名称标签、多分支边
#[test]
fn test_to_dot_diamond_dag() {
    let dag = common::build_diamond_dag();
    let dot = dag.to_dot();

    assert!(dot.contains("\"root\" -> \"left\""));
    assert!(dot.contains("\"root\" -> \"right\""));
    assert!(dot.contains("\"left\" -> \"sink\""));
    assert!(dot.contains("\"right\" -> \"sink\""));
    assert!(dot.contains("shape=ellipse"));
    assert!(dot.contains("Root Task"));
    assert!(dot.contains("Left Branch"));
}

/// 单节点 DAG 无箭头，验证仅含节点无边
#[test]
fn test_to_dot_single_node() {
    let dag = common::build_single_node_dag();
    let dot = dag.to_dot();

    assert!(dot.contains("\"only\""));
    assert!(dot.contains("Only Task"));
    assert!(!dot.contains("->"));
}

/// 特殊字符（引号、换行）在 DOT 输出中正确转义
#[test]
fn test_to_dot_escapes_special_chars() {
    let mut builder = cortex_flow::dag::DAGBuilder::new();
    builder.add_node(common::simple_task("n1", "Task \"with quotes\""));
    builder.add_node(common::simple_task("n2", "Task\nnewline"));
    builder.add_dependency("n1", "n2");
    let dag = builder.build().unwrap();
    let dot = dag.to_dot();

    assert!(dot.contains("\\\"with quotes\\\""));
    assert!(dot.contains("\\nnewline"));
}

/// 带标签边的 DOT 输出，验证 label 属性
#[test]
fn test_to_dot_edge_labels() {
    let mut builder = cortex_flow::dag::DAGBuilder::new();
    builder.add_node(common::simple_task("a", "A"));
    builder.add_node(common::simple_task("b", "B"));
    builder.add_edge(DAGEdge::new("a", "b").with_label("condition_x"));
    let dag = builder.build().unwrap();
    let dot = dag.to_dot();

    assert!(dot.contains("label=\"condition_x\""));
}

// ---- to_mermaid tests ----

/// 线性 DAG 的 Mermaid 格式，验证代码块标记、flowchart LR、箭头
#[test]
fn test_to_mermaid_linear_dag() {
    let dag = common::build_linear_dag();
    let mermaid = dag.to_mermaid();

    assert!(mermaid.starts_with("```mermaid\n"));
    assert!(mermaid.contains("flowchart LR"));
    assert!(mermaid.contains("a --> b"));
    assert!(mermaid.contains("b --> c"));
    assert!(mermaid.ends_with("```\n"));
}

/// 菱形 DAG 的 Mermaid 格式，验证多分支并行边
#[test]
fn test_to_mermaid_diamond_dag() {
    let dag = common::build_diamond_dag();
    let mermaid = dag.to_mermaid();

    assert!(mermaid.contains("root --> left"));
    assert!(mermaid.contains("root --> right"));
    assert!(mermaid.contains("left --> sink"));
    assert!(mermaid.contains("right --> sink"));
}

/// 单节点 Mermaid 无箭头，验证仅含节点名圆括号语法
#[test]
fn test_to_mermaid_single_node() {
    let dag = common::build_single_node_dag();
    let mermaid = dag.to_mermaid();

    assert!(mermaid.contains("only(Only Task)"));
    assert!(!mermaid.contains("-->"));
}

// ---- to_ascii tests ----

/// 线性 DAG 的 ASCII 输出，验证节点/边计数、名称显示
#[test]
fn test_to_ascii_linear_dag() {
    let dag = common::build_linear_dag();
    let ascii = dag.to_ascii();

    assert!(ascii.starts_with("DAG Structure:"));
    assert!(ascii.contains("Nodes: 3"));
    assert!(ascii.contains("Edges: 2"));
    assert!(ascii.contains("Task A"));
    assert!(ascii.contains("Task C"));
}

/// 菱形 DAG 的 ASCII 输出，验证 4 节点 4 边的统计
#[test]
fn test_to_ascii_diamond_dag() {
    let dag = common::build_diamond_dag();
    let ascii = dag.to_ascii();

    assert!(ascii.contains("Nodes: 4"));
    assert!(ascii.contains("Edges: 4"));
    assert!(ascii.contains("Root Task"));
    assert!(ascii.contains("Sink Task"));
}

/// 单节点 ASCII 输出，验证 1 节点 0 边
#[test]
fn test_to_ascii_single_node() {
    let dag = common::build_single_node_dag();
    let ascii = dag.to_ascii();

    assert!(ascii.contains("Nodes: 1"));
    assert!(ascii.contains("Edges: 0"));
}

// ---- structural consistency ----

/// DOT 结构一致性，验证菱形 DAG 所有节点和 4 条箭头均出现
#[test]
fn test_dot_contains_all_nodes_and_edges() {
    let dag = common::build_diamond_dag();
    let dot = dag.to_dot();

    for id in &["root", "left", "right", "sink"] {
        assert!(dot.contains(&format!("\"{}\"", id)), "missing node {}", id);
    }
    let arrow_count = dot.matches("->").count();
    assert_eq!(arrow_count, 4);
}

/// Mermaid 结构一致性，验证线性 DAG 的 2 条箭头数
#[test]
fn test_mermaid_contains_all_edges() {
    let dag = common::build_linear_dag();
    let mermaid = dag.to_mermaid();

    let arrow_count = mermaid.matches("-->").count();
    assert_eq!(arrow_count, 2);
}

// ---- Fixture-driven DAG tests ----

/// Fixture 驱动：从 linear_dag.json 构建 DAG，验证节点/边数及 DOT 输出
#[test]
fn test_dag_from_linear_fixture() {
    let fixture =
        fixture_loader::load_json_fixture::<fixture_loader::DAGFixture>("dag/linear_dag.json");
    let dag = common::build_dag_from_fixture(&fixture);

    assert_eq!(dag.node_count(), fixture.expected.node_count);
    assert_eq!(dag.edge_count(), fixture.expected.edge_count);

    let dot = dag.to_dot();
    assert!(dot.contains("\"a\" -> \"b\""));
    assert!(dot.contains("\"b\" -> \"c\""));
}

/// Fixture 驱动：从 diamond_dag.json 构建 DAG，验证节点/边数及菱形边
#[test]
fn test_dag_from_diamond_fixture() {
    let fixture =
        fixture_loader::load_json_fixture::<fixture_loader::DAGFixture>("dag/diamond_dag.json");
    let dag = common::build_dag_from_fixture(&fixture);

    assert_eq!(dag.node_count(), fixture.expected.node_count);
    assert_eq!(dag.edge_count(), fixture.expected.edge_count);

    let dot = dag.to_dot();
    assert!(dot.contains("\"root\" -> \"left\""));
    assert!(dot.contains("\"right\" -> \"sink\""));
}

/// Fixture 驱动：从 single_node_dag.json 构建单节点 DAG，验证节点/边数
#[test]
fn test_dag_from_single_node_fixture() {
    let fixture =
        fixture_loader::load_json_fixture::<fixture_loader::DAGFixture>("dag/single_node_dag.json");
    let dag = common::build_dag_from_fixture(&fixture);

    assert_eq!(dag.node_count(), fixture.expected.node_count);
    assert_eq!(dag.edge_count(), fixture.expected.edge_count);
}

/// Fixture 驱动：从 labeled_edge_dag.json 构建带标签边 DAG，验证 DOT label 属性
#[test]
fn test_dag_from_labeled_edge_fixture() {
    let fixture = fixture_loader::load_json_fixture::<fixture_loader::DAGFixture>(
        "dag/labeled_edge_dag.json",
    );
    let dag = common::build_dag_from_fixture(&fixture);

    assert_eq!(dag.edge_count(), 1);
    let dot = dag.to_dot();
    assert!(dot.contains("label=\"condition_x\""));
}
