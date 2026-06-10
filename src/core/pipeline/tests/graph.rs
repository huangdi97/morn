use crate::core::pipeline::*;
use serde_json::Value;

#[test]
fn test_branching_pipeline() {
    let mut pipeline = Pipeline::new();
    pipeline.add_node(PipelineNode::Input {
        id: "in".to_string(),
        source: "user".to_string(),
    });
    pipeline.add_node(PipelineNode::Transform {
        id: "upper".to_string(),
        operation: "to_upper".to_string(),
        params: Value::Null,
    });
    pipeline.add_node(PipelineNode::Transform {
        id: "double".to_string(),
        operation: "double".to_string(),
        params: Value::Null,
    });
    pipeline.add_node(PipelineNode::Output {
        id: "out1".to_string(),
        target: "file1".to_string(),
    });
    pipeline.add_node(PipelineNode::Output {
        id: "out2".to_string(),
        target: "file2".to_string(),
    });

    pipeline.add_connection(Connection {
        from: "in".to_string(),
        to: "upper".to_string(),
        from_port: "out".to_string(),
        to_port: "in".to_string(),
    });
    pipeline.add_connection(Connection {
        from: "upper".to_string(),
        to: "out1".to_string(),
        from_port: "out".to_string(),
        to_port: "in".to_string(),
    });
    pipeline.add_connection(Connection {
        from: "in".to_string(),
        to: "double".to_string(),
        from_port: "out".to_string(),
        to_port: "in".to_string(),
    });
    pipeline.add_connection(Connection {
        from: "double".to_string(),
        to: "out2".to_string(),
        from_port: "out".to_string(),
        to_port: "in".to_string(),
    });

    let result = pipeline.execute_simple_chain(PipelineData::Text("5".to_string()));
    assert!(result.is_ok());
}

#[test]
fn test_timer_node() {
    let mut pipeline = Pipeline::new();
    pipeline.add_node(PipelineNode::Timer {
        id: "timer1".to_string(),
        interval_secs: 0,
    });
    pipeline.add_node(PipelineNode::Output {
        id: "out".to_string(),
        target: "trigger".to_string(),
    });
    pipeline.add_connection(Connection {
        from: "timer1".to_string(),
        to: "out".to_string(),
        from_port: "out".to_string(),
        to_port: "in".to_string(),
    });

    let result = pipeline.execute_simple_chain(PipelineData::Text("start".to_string()));
    assert!(result.is_ok());
}

#[test]
fn test_cycle_detection() {
    let mut pipeline = Pipeline::new();
    pipeline.add_node(PipelineNode::Transform {
        id: "a".to_string(),
        operation: "to_upper".to_string(),
        params: Value::Null,
    });
    pipeline.add_node(PipelineNode::Transform {
        id: "b".to_string(),
        operation: "to_lower".to_string(),
        params: Value::Null,
    });
    pipeline.add_connection(Connection {
        from: "a".to_string(),
        to: "b".to_string(),
        from_port: "out".to_string(),
        to_port: "in".to_string(),
    });
    pipeline.add_connection(Connection {
        from: "b".to_string(),
        to: "a".to_string(),
        from_port: "out".to_string(),
        to_port: "in".to_string(),
    });

    let result = pipeline.execute(None);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("cycle"));
}
