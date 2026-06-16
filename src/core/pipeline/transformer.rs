//! 数据转换器 — 流水线中数据格式转换与校验
use crate::core::error::MornError;
use serde_json::{Map, Value};

pub trait DataTransformer {
    fn name(&self) -> &str;
    fn transform(&self, input: Value) -> Result<Value, MornError>;
    fn input_schema(&self) -> Value;
    fn output_schema(&self) -> Value;
}

pub struct TransformPipeline {
    stages: Vec<Box<dyn DataTransformer>>,
}

impl TransformPipeline {
    pub fn new() -> Self {
        TransformPipeline { stages: Vec::new() }
    }

    pub fn add_stage(&mut self, stage: Box<dyn DataTransformer>) {
        self.stages.push(stage);
    }

    pub fn execute(&self, input: Value) -> Result<Value, MornError> {
        let mut current = input;
        for stage in &self.stages {
            current = stage.transform(current)?;
        }
        Ok(current)
    }

    pub fn stage_count(&self) -> usize {
        self.stages.len()
    }
}

impl Default for TransformPipeline {
    fn default() -> Self {
        Self::new()
    }
}

pub struct FieldMapper {
    name: String,
    mappings: Vec<(String, String)>,
}

impl FieldMapper {
    pub fn new(name: &str, mappings: Vec<(String, String)>) -> Self {
        FieldMapper {
            name: name.to_string(),
            mappings,
        }
    }
}

impl DataTransformer for FieldMapper {
    fn name(&self) -> &str {
        &self.name
    }

    fn transform(&self, input: Value) -> Result<Value, MornError> {
        let obj = input
            .as_object()
            .ok_or_else(|| "input must be a JSON object".to_string())?;
        let mut output = Map::new();
        for (src, dst) in &self.mappings {
            let val = obj
                .get(src)
                .ok_or_else(|| format!("source field '{}' not found", src))?;
            output.insert(dst.clone(), val.clone());
        }
        Ok(Value::Object(output))
    }

    fn input_schema(&self) -> Value {
        serde_json::json!({ "type": "object", "properties": {} })
    }

    fn output_schema(&self) -> Value {
        serde_json::json!({ "type": "object", "properties": {} })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_pipeline_empty() {
        let pipeline = TransformPipeline::new();
        let input = json!({"key": "value"});
        let result = pipeline.execute(input.clone()).unwrap();
        assert_eq!(result, input);
    }

    #[test]
    fn test_pipeline_execute_passthrough() {
        let mut pipeline = TransformPipeline::new();
        let mapper = FieldMapper::new("passthrough", vec![("a".to_string(), "x".to_string())]);
        pipeline.add_stage(Box::new(mapper));

        let input = json!({"a": 1, "b": 2});
        let result = pipeline.execute(input).unwrap();
        assert_eq!(result, json!({"x": 1}));
    }

    #[test]
    fn test_mapper_name() {
        let mapper = FieldMapper::new("my_mapper", vec![]);
        assert_eq!(mapper.name(), "my_mapper");
    }
}
