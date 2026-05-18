//! # DAG 实现

use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use crate::error::{Error, Result};
use crate::execution::{ExecutionPlan, ExecutionNode};
use super::{DAGNode, DAGEdge};
use super::executor::DAGExecutor;

/// DAG（有向无环图）实现
///
/// 表示一个有向无环图，用于任务编排。
///
/// # 类型特点
///
/// - **节点**: 存储任务节点及其执行器
/// - **边**: 存储节点间的依赖关系
/// - **邻接表**: 优化后继节点查询
/// - **入度表**: 用于拓扑排序
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DAG {
    /// 节点映射
    #[serde(skip)]
    nodes: HashMap<String, DAGNode>,
    /// 边列表
    #[serde(default)]
    edges: Vec<DAGEdge>,
    /// 邻接表（出边）
    #[serde(skip)]
    adjacency: HashMap<String, Vec<String>>,
    /// 入度表
    #[serde(skip)]
    in_degree: HashMap<String, usize>,
    /// 反向邻接表（入边）
    #[serde(skip)]
    reverse_adjacency: HashMap<String, Vec<String>>,
}

impl Default for DAG {
    fn default() -> Self {
        Self::new()
    }
}

impl DAG {
    /// 创建新的空DAG
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            edges: Vec::new(),
            adjacency: HashMap::new(),
            in_degree: HashMap::new(),
            reverse_adjacency: HashMap::new(),
        }
    }

    /// 添加节点
    pub fn add_node(&mut self, node: DAGNode) {
        let node_id = node.id.clone();
        self.nodes.insert(node_id.clone(), node);
        self.adjacency.entry(node_id.clone()).or_insert_with(Vec::new);
        self.reverse_adjacency.entry(node_id.clone()).or_insert_with(Vec::new);
        self.in_degree.entry(node_id).or_insert(0);
    }

    /// 添加边
    pub fn add_edge(&mut self, edge: DAGEdge) {
        if !self.nodes.contains_key(&edge.from) {
            panic!("Source node not found: {}", edge.from);
        }
        if !self.nodes.contains_key(&edge.to) {
            panic!("Target node not found: {}", edge.to);
        }

        self.edges.push(edge.clone());
        
        // 更新出边表
        self.adjacency
            .entry(edge.from.clone())
            .or_insert_with(Vec::new)
            .push(edge.to.clone());
        
        // 更新入边表
        self.reverse_adjacency
            .entry(edge.to.clone())
            .or_insert_with(Vec::new)
            .push(edge.from.clone());
        
        // 更新入度
        *self.in_degree.entry(edge.to.clone()).or_insert(0) += 1;
    }

    /// 添加边（通过节点ID）
    pub fn add_dependency(&mut self, from: impl Into<String>, to: impl Into<String>) {
        self.add_edge(DAGEdge::new(from, to))
    }

    /// 获取节点
    pub fn get_node(&self, id: &str) -> Option<&DAGNode> {
        self.nodes.get(id)
    }

    /// 获取可变的节点
    pub fn get_node_mut(&mut self, id: &str) -> Option<&mut DAGNode> {
        self.nodes.get_mut(id)
    }

    /// 获取所有节点ID
    pub fn node_ids(&self) -> Vec<&String> {
        self.nodes.keys().collect()
    }

    /// 获取所有节点
    pub fn nodes(&self) -> &HashMap<String, DAGNode> {
        &self.nodes
    }

    /// 获取所有边
    pub fn edges(&self) -> &[DAGEdge] {
        &self.edges
    }

    /// 获取所有边（克隆）
    pub fn get_edges(&self) -> Vec<DAGEdge> {
        self.edges.clone()
    }

    /// 获取节点的入边
    pub fn incoming_edges(&self, node_id: &str) -> Vec<&DAGEdge> {
        self.edges.iter().filter(|e| e.to == node_id).collect()
    }

    /// 获取节点的出边
    pub fn outgoing_edges(&self, node_id: &str) -> Vec<&DAGEdge> {
        self.edges.iter().filter(|e| e.from == node_id).collect()
    }

    /// 获取根节点（无入边的节点）
    pub fn roots(&self) -> Vec<&DAGNode> {
        self.nodes
            .values()
            .filter(|node| self.in_degree.get(&node.id).copied().unwrap_or(0) == 0)
            .collect()
    }

    /// 获取叶子节点（无出边的节点）
    pub fn leaves(&self) -> Vec<&DAGNode> {
        self.nodes
            .values()
            .filter(|node| !self.adjacency.contains_key(&node.id) || self.adjacency.get(&node.id).map_or(true, |v| v.is_empty()))
            .collect()
    }

    /// 获取节点的后继
    pub fn successors(&self, node_id: &str) -> Vec<&DAGNode> {
        self.adjacency
            .get(node_id)
            .map(|ids| {
                ids.iter()
                    .filter_map(|id| self.nodes.get(id))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// 获取节点的前驱
    pub fn predecessors(&self, node_id: &str) -> Vec<&DAGNode> {
        self.reverse_adjacency
            .get(node_id)
            .map(|ids| {
                ids.iter()
                    .filter_map(|id| self.nodes.get(id))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// 检查节点是否有依赖
    pub fn has_dependencies(&self, node_id: &str) -> bool {
        self.in_degree.get(node_id).copied().unwrap_or(0) > 0
    }

    /// 获取节点的入度
    pub fn in_degree_of(&self, node_id: &str) -> usize {
        self.in_degree.get(node_id).copied().unwrap_or(0)
    }

    /// 获取节点的出度
    pub fn out_degree_of(&self, node_id: &str) -> usize {
        self.adjacency.get(node_id).map_or(0, |v| v.len())
    }

    /// 检查DAG是否为空
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    /// 获取节点数量
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    /// 获取边数量
    pub fn edge_count(&self) -> usize {
        self.edges.len()
    }

    /// 获取根节点数量
    pub fn root_count(&self) -> usize {
        self.roots().len()
    }

    /// 检查是否存在环
    pub fn has_cycle(&self) -> bool {
        let mut visited = HashSet::new();
        let mut rec_stack = HashSet::new();

        for node_id in self.nodes.keys() {
            if self.has_cycle_util(node_id, &mut visited, &mut rec_stack) {
                return true;
            }
        }
        false
    }

    fn has_cycle_util(
        &self,
        node_id: &str,
        visited: &mut HashSet<String>,
        rec_stack: &mut HashSet<String>,
    ) -> bool {
        if !visited.contains(node_id) {
            visited.insert(node_id.to_string());
            rec_stack.insert(node_id.to_string());

            if let Some(succs) = self.adjacency.get(node_id) {
                for succ in succs {
                    if !visited.contains(succ)
                        && self.has_cycle_util(succ, visited, rec_stack)
                    {
                        return true;
                    } else if rec_stack.contains(succ) {
                        return true;
                    }
                }
            }
        }
        rec_stack.remove(node_id);
        false
    }

    /// 查找环
    pub fn find_cycle(&self) -> Option<Vec<String>> {
        let mut visited = HashSet::new();
        let mut rec_stack = HashSet::new();
        let mut parent: HashMap<String, String> = HashMap::new();
        let mut cycle = Vec::new();

        for node_id in self.nodes.keys() {
            if self.find_cycle_util(node_id, &mut visited, &mut rec_stack, &mut parent, &mut cycle) {
                return Some(cycle);
            }
        }
        None
    }

    fn find_cycle_util(
        &self,
        node_id: &str,
        visited: &mut HashSet<String>,
        rec_stack: &mut HashSet<String>,
        parent: &mut HashMap<String, String>,
        cycle: &mut Vec<String>,
    ) -> bool {
        if !visited.contains(node_id) {
            visited.insert(node_id.to_string());
            rec_stack.insert(node_id.to_string());
            cycle.push(node_id.to_string());

            if let Some(succs) = self.adjacency.get(node_id) {
                for succ in succs {
                    parent.insert(succ.to_string(), node_id.to_string());
                    if !visited.contains(succ)
                        && self.find_cycle_util(succ, visited, rec_stack, parent, cycle)
                    {
                        return true;
                    } else if rec_stack.contains(succ) {
                        let mut current = succ.to_string();
                        while let Some(prev) = parent.get(&current) {
                            cycle.push(prev.to_string());
                            if prev == succ {
                                break;
                            }
                            current = prev.to_string();
                        }
                        return true;
                    }
                }
            }
        }
        cycle.pop();
        rec_stack.remove(node_id);
        false
    }

    /// 计算拓扑排序（Kahn算法）
    pub fn topological_sort(&self) -> Result<Vec<String>> {
        if self.has_cycle() {
            return Err(Error::DAGCycle("Cannot perform topological sort on DAG with cycle".to_string()));
        }

        let mut in_degree = self.in_degree.clone();
        let mut queue: VecDeque<String> = in_degree
            .iter()
            .filter(|(_, &deg)| deg == 0)
            .map(|(id, _)| id.clone())
            .collect();

        let mut result = Vec::new();

        while let Some(node_id) = queue.pop_front() {
            result.push(node_id.clone());
            if let Some(succs) = self.adjacency.get(&node_id) {
                for succ in succs {
                    if let Some(deg) = in_degree.get_mut(succ) {
                        *deg -= 1;
                        if *deg == 0 {
                            queue.push_back(succ.clone());
                        }
                    }
                }
            }
        }

        if result.len() != self.nodes.len() {
            return Err(Error::DAGCycle("Failed to compute topological order".to_string()));
        }
        Ok(result)
    }

    /// 计算层级（同一层节点可并行执行）
    pub fn compute_layers(&self) -> Result<Vec<Vec<String>>> {
        if self.has_cycle() {
            return Err(Error::DAGCycle("Cannot compute layers for DAG with cycle".to_string()));
        }

        let mut layers: Vec<Vec<String>> = Vec::new();
        let mut current_in_degree = self.in_degree.clone();
        let mut processed: HashSet<String> = HashSet::new();

        while processed.len() < self.nodes.len() {
            // 找到入度为0的节点
            let layer: Vec<String> = current_in_degree
                .iter()
                .filter(|(id, &deg)| deg == 0 && !processed.contains(*id))
                .map(|(id, _)| id.clone())
                .collect();

            if layer.is_empty() {
                return Err(Error::DAGCycle("Failed to compute layers - possible cycle".to_string()));
            }

            // 标记为已处理并更新入度
            for node_id in &layer {
                processed.insert(node_id.clone());
                if let Some(succs) = self.adjacency.get(node_id) {
                    for succ in succs {
                        if let Some(deg) = current_in_degree.get_mut(succ) {
                            *deg -= 1;
                        }
                    }
                }
            }

            layers.push(layer);
        }

        Ok(layers)
    }

    /// 验证DAG结构
    pub fn validate(&self) -> Result<()> {
        if self.nodes.is_empty() {
            return Err(Error::DAGValidation("DAG has no nodes".to_string()));
        }
        if self.has_cycle() {
            let cycle = self.find_cycle();
            return Err(Error::DAGCycle(format!("DAG contains cycle: {:?}", cycle)));
        }
        for edge in &self.edges {
            if !self.nodes.contains_key(&edge.from) {
                return Err(Error::DAGValidation(format!("Source node not found: {}", edge.from)));
            }
            if !self.nodes.contains_key(&edge.to) {
                return Err(Error::DAGValidation(format!("Target node not found: {}", edge.to)));
            }
        }
        if self.root_count() == 0 {
            return Err(Error::DAGValidation("DAG has no root nodes".to_string()));
        }
        Ok(())
    }

    /// 创建执行器
    pub fn executor(&self) -> DAGExecutor {
        DAGExecutor::new(self.clone())
    }

    /// 构建执行计划
    pub fn build_execution_plan(&self) -> Result<ExecutionPlan> {
        self.validate()?;
        let order = self.topological_sort()?;
        let layers = self.compute_layers()?;
        let layers_count = layers.len();
        let root_nodes = self.roots().iter().map(|n| n.id.clone()).collect();
        let nodes: Vec<ExecutionNode> = order.iter()
            .filter_map(|id| self.nodes.get(id))
            .map(|n| ExecutionNode {
                id: n.id.clone(),
                name: n.name.clone(),
                dependencies: n.dependencies.clone(),
                task: Arc::clone(&n.task),
            })
            .collect();

        Ok(ExecutionPlan {
            nodes,
            root_nodes,
            execution_order: order,
            layers,
            estimated_duration_ms: Some(layers_count as u64 * 1000),
            dag: None,
        })
    }

    /// 从另一个DAG合并节点
    pub fn merge(&mut self, other: &DAG) {
        for (id, node) in &other.nodes {
            if !self.nodes.contains_key(id) {
                self.add_node(node.clone());
            }
        }
        for edge in other.edges() {
            if !self.edges.contains(edge) {
                self.add_edge(edge.clone());
            }
        }
    }

    /// 导出为 DOT 格式（用于 Graphviz 可视化）
    ///
    /// 生成符合 Graphviz DOT 语言的图形描述，可用于可视化 DAG 结构。
    ///
    /// # 示例
    ///
    /// ```rust,ignore
    /// let dot = dag.to_dot();
    /// std::fs::write("dag.dot", &dot)?;
    /// // 使用 dot -Tpng dag.dot > dag.png 生成 PNG 图片
    /// ```
    pub fn to_dot(&self) -> String {
        let mut dot = String::new();
        dot.push_str("digraph DAG {\n");
        dot.push_str("    rankdir=TB;\n");
        dot.push_str("    node [shape=box, style=rounded];\n");
        dot.push_str("    edge [arrowhead=normal];\n\n");

        // 节点定义
        for (id, node) in &self.nodes {
            let escaped_name = node.name.replace('"', "\\\"").replace('\n', "\\n");
            let label = if escaped_name != *id {
                format!("{}\n({})", escaped_name, id)
            } else {
                escaped_name
            };
            let node_shape = if self.leaves().contains(&node) {
                "box"
            } else if self.roots().contains(&node) {
                "ellipse"
            } else {
                "box"
            };
            dot.push_str(&format!(
                "    \"{}\" [label=\"{}\", shape={}];\n",
                id, label, node_shape
            ));
        }

        dot.push('\n');

        // 边定义
        for edge in &self.edges {
            let edge_label = edge
                .label
                .as_ref()
                .map(|c| format!(" [label=\"{}\"]", c.replace('"', "\\\"")))
                .unwrap_or_default();
            dot.push_str(&format!("    \"{}\" -> \"{}\"{};\n", edge.from, edge.to, edge_label));
        }

        dot.push_str("\n}\n");
        dot
    }

    /// 导出为 Mermaid 格式（用于文档）
    ///
    /// 生成符合 Mermaid 图语法的描述，可用于 GitHub README 等场景。
    ///
    /// # 示例
    ///
    /// ```rust,ignore
    /// let mermaid = dag.to_mermaid();
    /// // 在 Markdown 中使用 ```mermaid 代码块
    /// ```
    pub fn to_mermaid(&self) -> String {
        let mut md = String::new();
        md.push_str("```mermaid\n");
        md.push_str("flowchart LR\n");
        md.push_str("    direction LR\n\n");

        // 定义子图（按层级分组）
        if let Ok(layers) = self.compute_layers() {
            for (i, layer) in layers.iter().enumerate() {
                md.push_str(&format!("    subgraph layer{} [{:?}]\n", i, format!("Layer {}", i)));
                for node_id in layer {
                    if let Some(node) = self.nodes.get(node_id) {
                        md.push_str(&format!(
                            "        {}({})\n",
                            node_id,
                            node.name.replace('"', "'")
                        ));
                    }
                }
                md.push_str("    end\n\n");
            }
        }

        // 定义边
        for edge in &self.edges {
            md.push_str(&format!("    {} --> {}\n", edge.from, edge.to));
        }

        md.push_str("```\n");
        md
    }

    /// 获取简短的图形化表示（用于控制台输出）
    ///
    /// 使用 ASCII 字符绘制 DAG 的简化视图。
    pub fn to_ascii(&self) -> String {
        let mut ascii = String::new();
        ascii.push_str("DAG Structure:\n");
        ascii.push_str(&format!("  Nodes: {}\n", self.node_count()));
        ascii.push_str(&format!("  Edges: {}\n", self.edge_count()));
        ascii.push_str(&format!("  Roots: {:?}\n", self.roots().iter().map(|n| n.name.as_str()).collect::<Vec<_>>()));
        ascii.push_str(&format!("  Leaves: {:?}\n", self.leaves().iter().map(|n| n.name.as_str()).collect::<Vec<_>>()));
        
        if let Ok(layers) = self.compute_layers() {
            ascii.push_str("\nLayers (nodes in same layer can execute in parallel):\n");
            for (i, layer) in layers.iter().enumerate() {
                let names: Vec<String> = layer.iter()
                    .filter_map(|id| self.nodes.get(id))
                    .map(|n| n.name.clone())
                    .collect();
                ascii.push_str(&format!("  Layer {}: {:?}\n", i, names));
            }
        }
        ascii
    }
}
