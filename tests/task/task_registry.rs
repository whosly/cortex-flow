//! TaskRegistry 集成测试
//!
//! 使用 `tests/fixtures/task/` 下的 JSON fixture 定义任务数据。

use crate::common::fixture_loader;
use cortex_flow::task::{SimpleTask, TaskMetadata, TaskRegistry};
use std::sync::Arc;

// ---- register tests ----

/// 注册任务后可通过 ID 查找，验证 contains/get/len
#[test]
fn test_registry_register_and_get() {
    let mut registry = TaskRegistry::new();
    let task = crate::common::simple_task("t1", "Task 1");
    registry.register(Arc::new(task)).unwrap();

    assert!(registry.contains("t1"));
    assert!(registry.get("t1").is_some());
    assert_eq!(registry.len(), 1);
}

/// 带元数据注册任务，验证元数据的 ID、名称、标签
#[test]
fn test_registry_register_with_metadata() {
    let mut registry = TaskRegistry::new();
    let task = crate::common::simple_task("t1", "Task 1");
    let meta = TaskMetadata::new("t1", "Task 1")
        .with_description("A test task")
        .with_tags(vec!["test".to_string()]);

    registry
        .register_with_metadata(Arc::new(task), meta)
        .unwrap();

    let meta = registry.get_metadata("t1").unwrap();
    assert_eq!(meta.id, "t1");
    assert!(meta.has_tag("test"));
}

/// 空 ID 注册应返回错误
#[test]
fn test_registry_register_validates_empty_id() {
    let mut registry = TaskRegistry::new();
    let task = SimpleTask::new("", "Task", |_input, _ctx| {
        Box::pin(async { Ok(serde_json::json!({})) })
    });
    let result = registry.register(Arc::new(task));
    assert!(result.is_err());
}

// ---- unregister tests ----

/// 注销任务后不再可查找，注册表为空
#[test]
fn test_registry_unregister() {
    let mut registry = TaskRegistry::new();
    let task = crate::common::simple_task("t1", "Task 1");
    registry.register(Arc::new(task)).unwrap();

    let removed = registry.unregister("t1");
    assert!(removed.is_some());
    assert!(!registry.contains("t1"));
    assert!(registry.is_empty());
}

// ---- find tests ----

/// 按标签查找任务，验证返回匹配标签的任务列表
#[test]
fn test_registry_find_by_tag() {
    let mut registry = TaskRegistry::new();
    let task = crate::common::simple_task("t1", "Fetch Data");
    let meta = TaskMetadata::new("t1", "Fetch Data").with_tags(vec!["fetch".to_string()]);
    registry
        .register_with_metadata(Arc::new(task), meta)
        .unwrap();

    let task2 = crate::common::simple_task("t2", "Process Data");
    let meta2 = TaskMetadata::new("t2", "Process Data").with_tags(vec!["process".to_string()]);
    registry
        .register_with_metadata(Arc::new(task2), meta2)
        .unwrap();

    let fetch_tasks = registry.find_by_tag("fetch");
    assert_eq!(fetch_tasks.len(), 1);
}

/// 按名称模糊查找任务，验证返回匹配名称的任务列表
#[test]
fn test_registry_find_by_name() {
    let mut registry = TaskRegistry::new();
    let task = crate::common::simple_task("t1", "Fetch Data");
    registry.register(Arc::new(task)).unwrap();

    let found = registry.find_by_name("fetch");
    assert_eq!(found.len(), 1);
}

// ---- metadata tests ----

/// 获取任务元数据，验证名称字段
#[test]
fn test_registry_metadata() {
    let mut registry = TaskRegistry::new();
    let task = crate::common::described_task("t1", "Task 1", "Description");
    registry.register(Arc::new(task)).unwrap();

    let meta = registry.get_metadata("t1").unwrap();
    assert_eq!(meta.name, "Task 1");
}

/// 获取所有任务元数据，验证数量
#[test]
fn test_registry_all_metadata() {
    let mut registry = TaskRegistry::new();
    registry
        .register(Arc::new(crate::common::simple_task("t1", "Task 1")))
        .unwrap();
    registry
        .register(Arc::new(crate::common::simple_task("t2", "Task 2")))
        .unwrap();

    assert_eq!(registry.all_metadata().len(), 2);
}

// ---- Fixture-driven task registration tests ----

/// Fixture 驱动：从 task_definitions.json 批量注册任务，验证标签查找
#[test]
fn test_registry_from_fixture() {
    let fixture = fixture_loader::load_json_fixture::<fixture_loader::TaskDefinitionsFixture>(
        "task/task_definitions.json",
    );

    let mut registry = TaskRegistry::new();
    for def in &fixture.definitions {
        let task = crate::common::simple_task(&def.id, &def.name);
        let meta = TaskMetadata::new(&def.id, &def.name);
        let meta = match &def.description {
            Some(desc) => meta.with_description(desc),
            None => meta,
        };
        let meta = if def.tags.is_empty() {
            meta
        } else {
            meta.with_tags(def.tags.clone())
        };
        registry
            .register_with_metadata(Arc::new(task), meta)
            .unwrap();
    }

    assert_eq!(registry.len(), fixture.definitions.len());

    // 验证标签查找
    let fetch_tasks = registry.find_by_tag("fetch");
    assert_eq!(fetch_tasks.len(), 1);

    let io_tasks = registry.find_by_tag("io");
    assert_eq!(io_tasks.len(), 2); // fetch_data + write_output
}

/// Fixture 驱动：从 task_definitions.json 注册后按名称模糊查找
#[test]
fn test_registry_find_by_name_from_fixture() {
    let fixture = fixture_loader::load_json_fixture::<fixture_loader::TaskDefinitionsFixture>(
        "task/task_definitions.json",
    );

    let mut registry = TaskRegistry::new();
    for def in &fixture.definitions {
        let task = crate::common::simple_task(&def.id, &def.name);
        registry.register(Arc::new(task)).unwrap();
    }

    // 名称模糊查找
    let found = registry.find_by_name("fetch");
    assert!(!found.is_empty());
}
