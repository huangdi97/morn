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
