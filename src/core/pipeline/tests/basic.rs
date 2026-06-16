//! Pipeline basic functionality tests.
#![cfg(test)]

use crate::core::error::MornError;
use crate::core::pipeline::*;
use serde_json::Value;

#[test]
fn test_simple_chain() {
    let mut pipeline = Pipeline::new();
    pipeline.add_node(PipelineNode::Input {
        id: "in1".to_string(),
        source: "user".to_string(),
    });
    pipeline.add_node(PipelineNode::Transform {
        id: "t1".to_string(),
        operation: "to_upper".to_string(),
        params: Value::Null,
    });
    pipeline.add_node(PipelineNode::Output {
        id: "out1".to_string(),
        target: "file".to_string(),
    });
    pipeline.add_connection(Connection {
        from: "in1".to_string(),
        to: "t1".to_string(),
        from_port: "out".to_string(),
        to_port: "in".to_string(),
    });
    pipeline.add_connection(Connection {
        from: "t1".to_string(),
        to: "out1".to_string(),
        from_port: "out".to_string(),
        to_port: "in".to_string(),
    });

    let result = pipeline.execute_simple_chain(PipelineData::Text("hello".to_string()));
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), PipelineData::Text("HELLO".to_string()));
}

#[test]
fn test_to_lower_chain() {
    let mut pipeline = Pipeline::new();
    pipeline.add_node(PipelineNode::Input {
        id: "in".to_string(),
        source: "user".to_string(),
    });
    pipeline.add_node(PipelineNode::Transform {
        id: "lower".to_string(),
        operation: "to_lower".to_string(),
        params: Value::Null,
    });
    pipeline.add_node(PipelineNode::Output {
        id: "out".to_string(),
        target: "console".to_string(),
    });
    pipeline.add_connection(Connection {
        from: "in".to_string(),
        to: "lower".to_string(),
        from_port: "out".to_string(),
        to_port: "in".to_string(),
    });
    pipeline.add_connection(Connection {
        from: "lower".to_string(),
        to: "out".to_string(),
        from_port: "out".to_string(),
        to_port: "in".to_string(),
    });

    let result = pipeline.execute_simple_chain(PipelineData::Text("HELLO WORLD".to_string()));
    assert!(result.is_ok());
    assert_eq!(
        result.unwrap(),
        PipelineData::Text("hello world".to_string())
    );
}

#[test]
fn test_transform_number_double() {
    let mut pipeline = Pipeline::new();
    pipeline.add_node(PipelineNode::Input {
        id: "in".to_string(),
        source: "calc".to_string(),
    });
    pipeline.add_node(PipelineNode::Transform {
        id: "double".to_string(),
        operation: "double".to_string(),
        params: Value::Null,
    });
    pipeline.add_node(PipelineNode::Output {
        id: "out".to_string(),
        target: "result".to_string(),
    });
    pipeline.add_connection(Connection {
        from: "in".to_string(),
        to: "double".to_string(),
        from_port: "out".to_string(),
        to_port: "in".to_string(),
    });
    pipeline.add_connection(Connection {
        from: "double".to_string(),
        to: "out".to_string(),
        from_port: "out".to_string(),
        to_port: "in".to_string(),
    });

    let result = pipeline.execute_simple_chain(PipelineData::Number(21.0));
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), PipelineData::Number(42.0));
}

#[test]
fn test_transform_to_number() {
    let mut pipeline = Pipeline::new();
    pipeline.add_node(PipelineNode::Input {
        id: "in".to_string(),
        source: "user".to_string(),
    });
    pipeline.add_node(PipelineNode::Transform {
        id: "to_num".to_string(),
        operation: "to_number".to_string(),
        params: Value::Null,
    });
    pipeline.add_node(PipelineNode::Output {
        id: "out".to_string(),
        target: "calc".to_string(),
    });
    pipeline.add_connection(Connection {
        from: "in".to_string(),
        to: "to_num".to_string(),
        from_port: "out".to_string(),
        to_port: "in".to_string(),
    });
    pipeline.add_connection(Connection {
        from: "to_num".to_string(),
        to: "out".to_string(),
        from_port: "out".to_string(),
        to_port: "in".to_string(),
    });

    let result = pipeline.execute_simple_chain(PipelineData::Text("3.14".to_string()));
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), PipelineData::Number(3.14));
}

#[test]
fn test_custom_executor() {
    struct AppendEx;

    impl PipelineNodeExecutor for AppendEx {
        fn execute(&self, input: PipelineData) -> Result<PipelineData, MornError> {
            match input {
                PipelineData::Text(t) => Ok(PipelineData::Text(t + "_custom")),
                _ => Err(MornError::Internal("expected text".to_string())),
            }
        }
    }

    let mut pipeline = Pipeline::new();
    pipeline.add_node(PipelineNode::Input {
        id: "in".to_string(),
        source: "user".to_string(),
    });
    pipeline.add_node(PipelineNode::Transform {
        id: "custom".to_string(),
        operation: "custom".to_string(),
        params: Value::Null,
    });
    pipeline.add_node(PipelineNode::Output {
        id: "out".to_string(),
        target: "result".to_string(),
    });
    pipeline.register_executor("custom", Box::new(AppendEx));
    pipeline.add_connection(Connection {
        from: "in".to_string(),
        to: "custom".to_string(),
        from_port: "out".to_string(),
        to_port: "in".to_string(),
    });
    pipeline.add_connection(Connection {
        from: "custom".to_string(),
        to: "out".to_string(),
        from_port: "out".to_string(),
        to_port: "in".to_string(),
    });

    let result = pipeline.execute_simple_chain(PipelineData::Text("data".to_string()));
    assert!(result.is_ok());
    assert_eq!(
        result.unwrap(),
        PipelineData::Text("data_custom".to_string())
    );
}

#[test]
fn test_empty_pipeline() {
    let mut pipeline = Pipeline::new();
    let result = pipeline.execute(None);
    assert!(result.is_ok());
    assert!(pipeline.collect_all_outputs().is_empty());
}

#[test]
fn test_pipeline_data_conversions() {
    let text = PipelineData::Text("42".to_string());
    assert_eq!(text.as_number().unwrap(), 42.0);

    let num = PipelineData::Number(3.14);
    assert_eq!(num.as_text().unwrap(), "3.14");

    let bad = PipelineData::Bytes(vec![1, 2, 3]);
    assert!(bad.as_text().is_err());
}
