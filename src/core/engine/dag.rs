//! dag — Executes task engines over directed acyclic workflow graphs.
use crate::core::error::MornError;
use super::TaskEngine;
use crate::core::supervisor::SubTaskDef;
use std::collections::{HashMap, VecDeque};

impl TaskEngine {
    pub fn compute_topological_order(
        &self,
        subtasks: &[SubTaskDef],
    ) -> Result<Vec<Vec<SubTaskDef>>, String> {
        let mut in_degree: HashMap<String, usize> = HashMap::new();
        let mut adj: HashMap<String, Vec<String>> = HashMap::new();
        let mut subtask_map: HashMap<String, &SubTaskDef> = HashMap::new();

        for s in subtasks {
            in_degree.insert(s.id.clone(), 0);
            adj.insert(s.id.clone(), Vec::new());
            subtask_map.insert(s.id.clone(), s);
        }

        for s in subtasks {
            for dep in &s.depends_on {
                if let Some(children) = adj.get_mut(dep) {
                    children.push(s.id.clone());
                }
                *in_degree.entry(s.id.clone()).or_insert(0) += 1;
            }
        }

        let mut levels: Vec<Vec<SubTaskDef>> = Vec::new();
        let mut queue: VecDeque<String> = VecDeque::new();

        for (id, deg) in in_degree.iter() {
            if *deg == 0 {
                queue.push_back(id.clone());
            }
        }

        let mut visited = 0;
        while !queue.is_empty() {
            let mut level = Vec::new();
            for _ in 0..queue.len() {
                if let Some(node) = queue.pop_front() {
                    if let Some(sub) = subtask_map.get(&node) {
                        level.push((*sub).clone());
                    }
                    visited += 1;
                    if let Some(children) = adj.get(&node) {
                        for child in children {
                            if let Some(deg) = in_degree.get_mut(child) {
                                *deg -= 1;
                                if *deg == 0 {
                                    queue.push_back(child.clone());
                                }
                            }
                        }
                    }
                }
            }
            if !level.is_empty() {
                levels.push(level);
            }
        }

        if visited != subtasks.len() {
            return Err("Circular dependency detected in subtask DAG".into());
        }

        Ok(levels)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::engine::TaskEngine;
    use serde_json::json;

    fn subtask(id: &str, depends_on: Vec<&str>) -> SubTaskDef {
        SubTaskDef {
            id: id.to_string(),
            agent_id: "agent-test".to_string(),
            action: "run".to_string(),
            params: json!({}),
            depends_on: depends_on.into_iter().map(str::to_string).collect(),
        }
    }

    #[test]
    fn topological_order_accepts_empty_dag() {
        let engine = TaskEngine::new(None, None);

        let levels = engine.compute_topological_order(&[]).unwrap();

        assert!(levels.is_empty());
    }

    #[test]
    fn topological_order_places_single_node_in_one_level() {
        let engine = TaskEngine::new(None, None);
        let tasks = vec![subtask("a", vec![])];

        let levels = engine.compute_topological_order(&tasks).unwrap();

        assert_eq!(levels.len(), 1);
        assert_eq!(levels[0][0].id, "a");
    }

    #[test]
    fn topological_order_keeps_linear_chain_order() {
        let engine = TaskEngine::new(None, None);
        let tasks = vec![
            subtask("a", vec![]),
            subtask("b", vec!["a"]),
            subtask("c", vec!["b"]),
        ];

        let levels = engine.compute_topological_order(&tasks).unwrap();

        assert_eq!(levels.len(), 3);
        assert_eq!(levels[0][0].id, "a");
        assert_eq!(levels[1][0].id, "b");
        assert_eq!(levels[2][0].id, "c");
    }

    #[test]
    fn topological_order_groups_independent_dependents() {
        let engine = TaskEngine::new(None, None);
        let tasks = vec![
            subtask("root", vec![]),
            subtask("left", vec!["root"]),
            subtask("right", vec!["root"]),
        ];

        let levels = engine.compute_topological_order(&tasks).unwrap();
        let second_level: Vec<&str> = levels[1].iter().map(|task| task.id.as_str()).collect();

        assert_eq!(levels.len(), 2);
        assert_eq!(levels[0][0].id, "root");
        assert!(second_level.contains(&"left"));
        assert!(second_level.contains(&"right"));
    }

    #[test]
    fn topological_order_rejects_cycle() {
        let engine = TaskEngine::new(None, None);
        let tasks = vec![subtask("a", vec!["b"]), subtask("b", vec!["a"])];

        let err = engine.compute_topological_order(&tasks).unwrap_err();

        assert!(err.contains("Circular dependency"));
    }
}
