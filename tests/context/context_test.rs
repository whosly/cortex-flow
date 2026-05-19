//! ExecutionContext 和 ContextScope 集成测试
//!
//! 使用 `tests/fixtures/context/` 下的 JSON fixture 定义作用域测试数据。

use cortex_flow::context::{ContextScope, ExecutionContext};

// ---- ExecutionContext tests ----

/// 上下文 set/get 基本操作，验证键值存取
#[test]
fn test_context_set_and_get() {
    let mut ctx = ExecutionContext::new();
    ctx.set("key", "value").unwrap();
    let val: Option<String> = ctx.get("key");
    assert_eq!(val, Some("value".to_string()));
}

/// 上下文快照与恢复，验证 snapshot 保存后 restore 可回滚
#[test]
fn test_context_snapshot_and_restore() {
    let mut ctx = ExecutionContext::new();
    ctx.set("key1", "value1").unwrap();
    ctx.set("key2", 42i32).unwrap();

    let snapshot = ctx.snapshot();
    assert_eq!(snapshot.len(), 2);

    ctx.set("key3", "value3").unwrap();
    assert_eq!(ctx.len(), 3);

    ctx.restore(&snapshot);
    assert_eq!(ctx.len(), 2);
    let val: Option<String> = ctx.get("key1");
    assert_eq!(val, Some("value1".to_string()));
}

/// 上下文清空，验证 clear 后 is_empty
#[test]
fn test_context_clear() {
    let mut ctx = ExecutionContext::new();
    ctx.set("key", "value").unwrap();
    assert!(!ctx.is_empty());

    ctx.clear();
    assert!(ctx.is_empty());
}

/// 上下文 contains/remove 操作，验证键存在性判断与删除
#[test]
fn test_context_contains_and_remove() {
    let mut ctx = ExecutionContext::new();
    ctx.set("key", "value").unwrap();
    assert!(ctx.contains("key"));
    assert!(!ctx.contains("nonexistent"));

    ctx.remove("key");
    assert!(!ctx.contains("key"));
}

/// 上下文 keys 操作，验证返回所有键
#[test]
fn test_context_keys() {
    let mut ctx = ExecutionContext::new();
    ctx.set("a", 1i32).unwrap();
    ctx.set("b", 2i32).unwrap();
    let keys = ctx.keys();
    assert_eq!(keys.len(), 2);
}

// ---- ContextScope isolation tests ----

/// 作用域隔离：子作用域覆盖父数据但不影响父级，父看不到子数据
#[test]
fn test_scope_isolation() {
    let mut parent = ContextScope::new("parent");
    parent.set("shared", serde_json::json!("from_parent"));
    parent.set("parent_only", serde_json::json!("only_in_parent"));

    let mut child = parent.child("child");
    child.set("shared", serde_json::json!("from_child"));
    child.set("child_only", serde_json::json!("only_in_child"));

    // Child reads its own override
    assert_eq!(child.get("shared"), Some(&serde_json::json!("from_child")));
    // Child reads parent's data
    assert_eq!(
        child.get("parent_only"),
        Some(&serde_json::json!("only_in_parent"))
    );
    // Parent unaffected
    assert_eq!(
        parent.get("shared"),
        Some(&serde_json::json!("from_parent"))
    );
    // Parent can't see child data
    assert!(parent.get("child_only").is_none());
}

/// 作用域层级：根 level=0，子 level=1，is_root 判断
#[test]
fn test_scope_level() {
    let root = ContextScope::new("root");
    assert_eq!(root.level, 0);
    assert!(root.is_root());

    let child = root.child("child");
    assert_eq!(child.level, 1);
    assert!(!child.is_root());
}

// ---- Fixture-driven scope isolation tests ----

/// Fixture 驱动：从 scope_isolation.json 验证父子作用域数据覆盖/继承/不可见
#[test]
fn test_scope_isolation_from_fixture() {
    use crate::common::fixture_loader;

    let fixture = fixture_loader::load_fixture("context/scope_isolation.json");
    let data: serde_json::Value = serde_json::from_str(&fixture).unwrap();

    let scopes = data.get("scopes").unwrap().as_array().unwrap();

    // 测试父子作用域隔离
    let isolation = scopes
        .iter()
        .find(|s| s["name"] == "parent_child_isolation")
        .unwrap();
    let parent_data = isolation.get("parent").unwrap();
    let child_data = isolation.get("child").unwrap();
    let expected = isolation.get("expected").unwrap();

    let mut parent = ContextScope::new(parent_data["name"].as_str().unwrap());
    for (key, value) in parent_data["data"].as_object().unwrap() {
        parent.set(key, value.clone());
    }

    let mut child = parent.child(child_data["name"].as_str().unwrap());
    for (key, value) in child_data["data"].as_object().unwrap() {
        child.set(key, value.clone());
    }

    // 验证子作用域读取自身覆盖
    let override_key = expected["child_sees_own_override"]["key"].as_str().unwrap();
    let override_val = &expected["child_sees_own_override"]["value"];
    assert_eq!(child.get(override_key), Some(override_val));

    // 验证子作用域读取父数据
    let parent_key = expected["child_sees_parent_data"]["key"].as_str().unwrap();
    let parent_val = &expected["child_sees_parent_data"]["value"];
    assert_eq!(child.get(parent_key), Some(parent_val));

    // 验证父作用域不受影响
    let unaffected_key = expected["parent_unaffected"]["key"].as_str().unwrap();
    let unaffected_val = &expected["parent_unaffected"]["value"];
    assert_eq!(parent.get(unaffected_key), Some(unaffected_val));

    // 验证父作用域看不到子作用域数据
    let invisible_key = expected["parent_cannot_see_child"]["key"].as_str().unwrap();
    let invisible_exists = expected["parent_cannot_see_child"]["exists"]
        .as_bool()
        .unwrap();
    assert_eq!(parent.get(invisible_key).is_some(), invisible_exists);
}
