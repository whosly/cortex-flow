//! DAG 可视化导出测试
//!
//! 对应 `src/dag/dag_impl.rs` 中 `DAG::to_dot / to_mermaid / to_ascii` 方法。

use crate::common;
use cortex_flow::dag::DAGEdge;

// ---- to_dot tests ----

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

#[test]
fn test_to_dot_single_node() {
    let dag = common::build_single_node_dag();
    let dot = dag.to_dot();

    assert!(dot.contains("\"only\""));
    assert!(dot.contains("Only Task"));
    assert!(!dot.contains("->"));
}

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

#[test]
fn test_to_mermaid_diamond_dag() {
    let dag = common::build_diamond_dag();
    let mermaid = dag.to_mermaid();

    assert!(mermaid.contains("root --> left"));
    assert!(mermaid.contains("root --> right"));
    assert!(mermaid.contains("left --> sink"));
    assert!(mermaid.contains("right --> sink"));
}

#[test]
fn test_to_mermaid_single_node() {
    let dag = common::build_single_node_dag();
    let mermaid = dag.to_mermaid();

    assert!(mermaid.contains("only(Only Task)"));
    assert!(!mermaid.contains("-->"));
}

// ---- to_ascii tests ----

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

#[test]
fn test_to_ascii_diamond_dag() {
    let dag = common::build_diamond_dag();
    let ascii = dag.to_ascii();

    assert!(ascii.contains("Nodes: 4"));
    assert!(ascii.contains("Edges: 4"));
    assert!(ascii.contains("Root Task"));
    assert!(ascii.contains("Sink Task"));
}

#[test]
fn test_to_ascii_single_node() {
    let dag = common::build_single_node_dag();
    let ascii = dag.to_ascii();

    assert!(ascii.contains("Nodes: 1"));
    assert!(ascii.contains("Edges: 0"));
}

// ---- structural consistency ----

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

#[test]
fn test_mermaid_contains_all_edges() {
    let dag = common::build_linear_dag();
    let mermaid = dag.to_mermaid();

    let arrow_count = mermaid.matches("-->").count();
    assert_eq!(arrow_count, 2);
}
