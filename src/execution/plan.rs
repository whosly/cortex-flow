//! # 执行计划模块
//!
//! 定义执行计划和相关数据结构。
//!
//! ## 核心概念
//!
//! ### ExecutionPlan（执行计划）
//!
//! 执行计划描述了 DAG 的执行策略，包括：
//!
//! - **执行顺序**: 通过拓扑排序确定的节点执行顺序
//! - **执行层级**: 同一层级的节点可以并行执行
//! - **节点信息**: 包含执行节点列表和根节点信息
//! - **DAG引用**: 保留原始 DAG 引用，用于回滚等操作
//!
//! ### ExecutionNode（执行节点）
//!
//! 执行节点是 DAG 节点在执行计划中的表示：
//!
//! - 包含节点 ID、名称和依赖关系
//! - 包含任务的执行器引用

use std::sync::Arc;
use serde::{Deserialize, Deserializer, Serialize};
use crate::task::{EmptyTaskExecutor, TaskExecutor};
use crate::dag::DAG;

/// 执行计划
///
/// 描述 DAG 的完整执行策略，包括执行顺序、并行层级和节点信息。
///
/// # 字段说明
///
/// - `nodes`: 执行节点列表，包含节点ID、名称、依赖和任务执行器
/// - `root_nodes`: 根节点ID列表（无依赖的起始节点）
/// - `execution_order`: 拓扑排序确定的执行顺序
/// - `layers`: 可并行执行的层级（同一层内节点可并行）
/// - `estimated_duration_ms`: 预计执行时间（毫秒）
/// - `dag`: 原始 DAG 引用（用于回滚等操作）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionPlan {
    /// 执行节点列表
    pub nodes: Vec<ExecutionNode>,
    /// 根节点ID列表
    pub root_nodes: Vec<String>,
    /// 拓扑排序的执行顺序
    pub execution_order: Vec<String>,
    /// 可并行执行的层级
    #[serde(default)]
    pub layers: Vec<Vec<String>>,
    /// 预计执行时间（毫秒）
    #[serde(default)]
    pub estimated_duration_ms: Option<u64>,
    /// DAG引用（可选，用于回滚等操作）
    #[serde(skip)]
    pub dag: Option<Arc<DAG>>,
}

impl ExecutionPlan {
    /// 创建新的执行计划
    ///
    /// # 参数
    ///
    /// - `nodes`: 执行节点列表
    /// - `root_nodes`: 根节点ID列表
    /// - `execution_order`: 拓扑排序的执行顺序
    pub fn new(
        nodes: Vec<ExecutionNode>,
        root_nodes: Vec<String>,
        execution_order: Vec<String>,
    ) -> Self {
        Self {
            nodes,
            root_nodes,
            execution_order,
            layers: Vec::new(),
            estimated_duration_ms: None,
            dag: None,
        }
    }

    /// 设置 DAG 引用
    pub fn with_dag(mut self, dag: Arc<DAG>) -> Self {
        self.dag = Some(dag);
        self
    }

    /// 设置执行层级
    pub fn with_layers(mut self, layers: Vec<Vec<String>>) -> Self {
        self.layers = layers;
        self
    }

    /// 设置预计执行时间
    pub fn with_estimated_duration(mut self, duration_ms: u64) -> Self {
        self.estimated_duration_ms = Some(duration_ms);
        self
    }

    /// 获取节点数量
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    /// 获取根节点数量
    pub fn root_count(&self) -> usize {
        self.root_nodes.len()
    }

    /// 获取层级数量
    pub fn layer_count(&self) -> usize {
        self.layers.len()
    }

    /// 检查执行计划是否为空
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    /// 根据ID获取节点
    pub fn get_node(&self, id: &str) -> Option<&ExecutionNode> {
        self.nodes.iter().find(|n| n.id == id)
    }

    /// 获取执行顺序
    pub fn order(&self) -> &[String] {
        &self.execution_order
    }

    /// 检查是否包含指定节点
    pub fn contains_node(&self, id: &str) -> bool {
        self.nodes.iter().any(|n| n.id == id)
    }

    /// 获取 DAG 引用
    pub fn dag(&self) -> Option<&Arc<DAG>> {
        self.dag.as_ref()
    }

    /// 获取指定层级的节点
    ///
    /// # 参数
    ///
    /// - `layer_index`: 层级索引（从0开始）
    pub fn get_layer(&self, layer_index: usize) -> Option<&[String]> {
        self.layers.get(layer_index).map(|l| l.as_slice())
    }

    /// 获取最大并行度
    ///
    /// 返回所有层级中节点数量最多的层级的节点数。
    pub fn max_parallelism(&self) -> usize {
        self.layers.iter().map(|l| l.len()).max().unwrap_or(1)
    }
}

/// 执行节点
///
/// 在执行计划中表示一个节点。
pub struct ExecutionNode {
    /// 节点ID
    pub id: String,
    /// 节点名称
    pub name: String,
    /// 依赖节点ID列表
    pub dependencies: Vec<String>,
    /// 任务执行器（手动实现 Serialize/Deserialize，跳过此字段）
    pub task: Arc<dyn TaskExecutor>,
}

/// 手动实现 Serialize，跳过 task 字段
impl Serialize for ExecutionNode {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("ExecutionNode", 3)?;
        state.serialize_field("id", &self.id)?;
        state.serialize_field("name", &self.name)?;
        state.serialize_field("dependencies", &self.dependencies)?;
        state.end()
    }
}

/// 手动实现 Deserialize，反序列化时 task 字段使用占位符
impl<'de> Deserialize<'de> for ExecutionNode {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(field_identifier, rename_all = "camelCase")]
        enum Field {
            Id,
            Name,
            Dependencies,
        }

        struct ExecutionNodeVisitor;

        impl<'de> serde::de::Visitor<'de> for ExecutionNodeVisitor {
            type Value = ExecutionNode;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct ExecutionNode")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::SeqAccess<'de>,
            {
                let id = seq.next_element()?
                    .ok_or_else(|| serde::de::Error::invalid_length(0, &self))?;
                let name = seq.next_element()?
                    .ok_or_else(|| serde::de::Error::invalid_length(1, &self))?;
                let dependencies: Option<Vec<String>> = seq.next_element()?;
                Ok(ExecutionNode {
                    id,
                    name,
                    dependencies: dependencies.unwrap_or_default(),
                    task: Arc::new(EmptyTaskExecutor),
                })
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::MapAccess<'de>,
            {
                let mut id = None;
                let mut name = None;
                let mut dependencies = None;

                while let Some(key) = map.next_key()? {
                    match key {
                        Field::Id => {
                            id = Some(map.next_value()?);
                        }
                        Field::Name => {
                            name = Some(map.next_value()?);
                        }
                        Field::Dependencies => {
                            dependencies = Some(map.next_value()?);
                        }
                    }
                }

                Ok(ExecutionNode {
                    id: id.ok_or_else(|| serde::de::Error::missing_field("id"))?,
                    name: name.ok_or_else(|| serde::de::Error::missing_field("name"))?,
                    dependencies: dependencies.unwrap_or_default(),
                    task: Arc::new(EmptyTaskExecutor),
                })
            }
        }

        const FIELDS: &'static [&'static str] = &["id", "name", "dependencies"];
        deserializer.deserialize_struct("ExecutionNode", FIELDS, ExecutionNodeVisitor)
    }
}

impl Clone for ExecutionNode {
    fn clone(&self) -> Self {
        ExecutionNode {
            id: self.id.clone(),
            name: self.name.clone(),
            dependencies: self.dependencies.clone(),
            task: Arc::new(EmptyTaskExecutor),
        }
    }
}

impl std::fmt::Debug for ExecutionNode {
    /// 自定义 Debug 实现，跳过无法格式化的 task 字段
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ExecutionNode")
            .field("id", &self.id)
            .field("name", &self.name)
            .field("dependencies", &self.dependencies)
            .field("task", &"<TaskExecutor>")
            .finish()
    }
}

impl ExecutionNode {
    pub fn new(id: String, name: String, task: Arc<dyn TaskExecutor>) -> Self {
        Self {
            id,
            name,
            dependencies: Vec::new(),
            task,
        }
    }

    pub fn with_dependency(mut self, dep: impl Into<String>) -> Self {
        self.dependencies.push(dep.into());
        self
    }

    pub fn with_dependencies(mut self, deps: Vec<String>) -> Self {
        self.dependencies = deps;
        self
    }

    pub fn has_dependency(&self, dep: &str) -> bool {
        self.dependencies.contains(&dep.to_string())
    }
}
