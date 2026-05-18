//! # DAG Node 定义
//!
//! 定义 DAG 中的节点类型。

use std::sync::Arc;
use serde::{Deserialize, Deserializer, Serialize};
use crate::task::TaskExecutor;

/// DAG节点
///
/// 表示 DAG 中的一个执行节点。
pub struct DAGNode {
    /// 节点ID
    pub id: String,
    /// 节点名称
    pub name: String,
    /// 任务执行器（手动实现 Serialize/Deserialize，跳过此字段）
    pub task: Arc<dyn TaskExecutor>,
    /// 输入依赖
    pub dependencies: Vec<String>,
    /// 执行结果
    pub result: Option<serde_json::Value>,
}

impl DAGNode {
    /// 创建新节点
    pub fn new(id: impl Into<String>, task: Arc<dyn TaskExecutor>) -> Self {
        Self {
            id: id.into(),
            name: task.task_name().to_string(),
            task,
            dependencies: Vec::new(),
            result: None,
        }
    }

    /// 创建带名称的节点
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = name.into();
        self
    }

    /// 添加依赖
    pub fn depends_on(mut self, dependency_id: impl Into<String>) -> Self {
        self.dependencies.push(dependency_id.into());
        self
    }
}

/// 手动实现 Serialize，跳过 task 字段
impl Serialize for DAGNode {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("DAGNode", 4)?;
        state.serialize_field("id", &self.id)?;
        state.serialize_field("name", &self.name)?;
        state.serialize_field("dependencies", &self.dependencies)?;
        state.serialize_field("result", &self.result)?;
        state.end()
    }
}

/// 手动实现 Deserialize，反序列化时 task 字段使用占位符
impl<'de> Deserialize<'de> for DAGNode {
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
            Result,
        }

        struct DAGNodeVisitor;

        impl<'de> serde::de::Visitor<'de> for DAGNodeVisitor {
            type Value = DAGNode;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct DAGNode")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::SeqAccess<'de>,
            {
                let id = seq.next_element()?
                    .ok_or_else(|| serde::de::Error::invalid_length(0, &self))?;
                let name = seq.next_element()?
                    .ok_or_else(|| serde::de::Error::invalid_length(1, &self))?;
                let dependencies: Vec<String> = seq.next_element()?
                    .ok_or_else(|| serde::de::Error::invalid_length(2, &self))?;
                let result: Option<serde_json::Value> = seq.next_element()?;
                Ok(DAGNode {
                    id,
                    name,
                    task: Arc::new(crate::task::EmptyTaskExecutor),
                    dependencies,
                    result,
                })
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::MapAccess<'de>,
            {
                let mut id = None;
                let mut name = None;
                let mut dependencies = None;
                let mut result = None;

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
                        Field::Result => {
                            result = map.next_value()?;
                        }
                    }
                }

                Ok(DAGNode {
                    id: id.ok_or_else(|| serde::de::Error::missing_field("id"))?,
                    name: name.ok_or_else(|| serde::de::Error::missing_field("name"))?,
                    task: Arc::new(crate::task::EmptyTaskExecutor),
                    dependencies: dependencies.unwrap_or_default(),
                    result,
                })
            }
        }

        const FIELDS: &'static [&'static str] = &["id", "name", "dependencies", "result"];
        deserializer.deserialize_struct("DAGNode", FIELDS, DAGNodeVisitor)
    }
}

impl Clone for DAGNode {
    fn clone(&self) -> Self {
        DAGNode {
            id: self.id.clone(),
            name: self.name.clone(),
            task: Arc::clone(&self.task),  // 正确克隆 Arc<TaskExecutor>
            dependencies: self.dependencies.clone(),
            result: self.result.clone(),
        }
    }
}

impl PartialEq for DAGNode {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for DAGNode {}

impl std::hash::Hash for DAGNode {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl std::fmt::Debug for DAGNode {
    /// 自定义 Debug 实现，跳过无法格式化的 task 字段
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DAGNode")
            .field("id", &self.id)
            .field("name", &self.name)
            .field("task", &"<TaskExecutor>")
            .field("dependencies", &self.dependencies)
            .field("result", &self.result)
            .finish()
    }
}
