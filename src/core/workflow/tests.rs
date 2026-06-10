use super::*;

#[test]
fn test_list_builtin_templates() {
    let templates = WorkflowTemplate::list_builtin();
    assert_eq!(templates.len(), 8);
}

#[test]
fn test_get_template_by_id() {
    let template = WorkflowTemplate::get_by_id("workflow-task-execution");
    assert!(template.is_some());
    assert_eq!(template.unwrap().name, "Task Execution");
}

#[test]
fn test_template_categories() {
    let templates = WorkflowTemplate::list_builtin();
    let categories: Vec<&str> = templates.iter().map(|t| t.category.as_str()).collect();
    assert!(categories.contains(&"general"));
    assert!(categories.contains(&"research"));
    assert!(categories.contains(&"development"));
    assert!(categories.contains(&"operations"));
}

#[test]
fn test_task_execution_has_six_steps() {
    let t = WorkflowTemplate::get_by_id("workflow-task-execution").unwrap();
    assert_eq!(t.steps.len(), 6);
}

#[test]
fn test_code_delivery_has_seven_steps() {
    let t = WorkflowTemplate::get_by_id("workflow-code-delivery").unwrap();
    assert_eq!(t.steps.len(), 7);
}

#[test]
fn test_all_templates_have_steps() {
    for t in WorkflowTemplate::list_builtin() {
        assert!(!t.steps.is_empty(), "Template '{}' has no steps", t.id);
    }
}

#[test]
fn test_workflow_action_serialization() {
    let action = WorkflowAction::ToolCall {
        tool_id: "web_search".into(),
        params: serde_json::json!({"q": "test"}),
    };
    let json = serde_json::to_string(&action).unwrap();
    let deserialized: WorkflowAction = serde_json::from_str(&json).unwrap();
    match deserialized {
        WorkflowAction::ToolCall { tool_id, .. } => assert_eq!(tool_id, "web_search"),
        _ => panic!("Wrong variant"),
    }
}

#[test]
fn test_variable_store_set_and_get() {
    let mut store = VariableStore::new();
    store
        .set("step1", "result", serde_json::json!("hello"))
        .unwrap();
    let var = store.get("result").unwrap();
    assert_eq!(var.value, "hello");
    assert_eq!(var.source_step.unwrap(), "step1");
}

#[test]
fn test_variable_store_get_missing() {
    let store = VariableStore::new();
    let result = store.get("nonexistent");
    assert!(result.is_err());
}

#[test]
fn test_variable_store_type_detection() {
    let mut store = VariableStore::new();
    store.set("s1", "str", serde_json::json!("text")).unwrap();
    store.set("s1", "num", serde_json::json!(42)).unwrap();
    store.set("s1", "flag", serde_json::json!(true)).unwrap();
    store
        .set("s1", "arr", serde_json::json!([1, 2, 3]))
        .unwrap();
    store
        .set("s1", "obj", serde_json::json!({"k": "v"}))
        .unwrap();
    store.set("s1", "null", serde_json::json!(null)).unwrap();

    assert!(matches!(
        store.get("str").unwrap().var_type,
        VarType::String
    ));
    assert!(matches!(
        store.get("num").unwrap().var_type,
        VarType::Number
    ));
    assert!(matches!(
        store.get("flag").unwrap().var_type,
        VarType::Boolean
    ));
    assert!(matches!(store.get("arr").unwrap().var_type, VarType::Array));
    assert!(matches!(
        store.get("obj").unwrap().var_type,
        VarType::Object
    ));
    assert!(matches!(store.get("null").unwrap().var_type, VarType::Null));
}

#[test]
fn test_variable_store_convert_string_to_number() {
    let var = Variable {
        name: "score".into(),
        var_type: VarType::String,
        value: serde_json::json!("95.5"),
        source_step: None,
    };
    let store = VariableStore::new();
    let converted = store.convert(&var, VarType::Number).unwrap();
    assert_eq!(converted.value, 95.5);
}

#[test]
fn test_variable_store_convert_number_to_string() {
    let var = Variable {
        name: "count".into(),
        var_type: VarType::Number,
        value: serde_json::json!(42),
        source_step: None,
    };
    let store = VariableStore::new();
    let converted = store.convert(&var, VarType::String).unwrap();
    assert_eq!(converted.value, "42");
}

#[test]
fn test_variable_store_convert_bool_to_string() {
    let var = Variable {
        name: "flag".into(),
        var_type: VarType::Boolean,
        value: serde_json::json!(true),
        source_step: None,
    };
    let store = VariableStore::new();
    let converted = store.convert(&var, VarType::String).unwrap();
    assert_eq!(converted.value, "true");
}

#[test]
fn test_variable_store_all() {
    let mut store = VariableStore::new();
    store.set("a", "x", serde_json::json!(1)).unwrap();
    store.set("b", "y", serde_json::json!(2)).unwrap();
    assert_eq!(store.all().len(), 2);
}

#[test]
fn test_variable_store_clear() {
    let mut store = VariableStore::new();
    store.set("a", "x", serde_json::json!(1)).unwrap();
    store.clear();
    assert!(store.get("x").is_err());
}
