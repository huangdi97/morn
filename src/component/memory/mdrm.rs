//! mdrm — Multi-Dimensional Relationship Memory graph storage.
use std::collections::{HashMap, HashSet, VecDeque};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MDRMGraph {
    entities: HashMap<String, Entity>,
    relations: Vec<Relation>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Entity {
    pub id: String,
    pub name: String,
    pub entity_type: String,
    pub properties: HashMap<String, String>,
    pub created_at: String,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Relation {
    pub source_id: String,
    pub target_id: String,
    pub relation_type: String,
    pub weight: f64,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct CausalChain {
    pub id: String,
    pub events: Vec<String>,
    pub description: String,
}

impl MDRMGraph {
    pub fn new() -> Self {
        Self {
            entities: HashMap::new(),
            relations: Vec::new(),
        }
    }

    pub fn add_entity(&mut self, entity: Entity) -> Result<(), String> {
        if self.entities.contains_key(&entity.id) {
            return Err(format!("Duplicate entity id: {}", entity.id));
        }

        self.entities.insert(entity.id.clone(), entity);
        Ok(())
    }

    pub fn add_relation(
        &mut self,
        source_id: impl Into<String>,
        target_id: impl Into<String>,
        relation_type: impl Into<String>,
        weight: f64,
    ) -> Result<(), String> {
        let source_id = source_id.into();
        let target_id = target_id.into();

        if !self.entities.contains_key(&source_id) {
            return Err(format!("Unknown source entity id: {source_id}"));
        }
        if !self.entities.contains_key(&target_id) {
            return Err(format!("Unknown target entity id: {target_id}"));
        }

        self.relations.push(Relation {
            source_id,
            target_id,
            relation_type: relation_type.into(),
            weight,
            metadata: HashMap::new(),
        });
        Ok(())
    }

    pub fn query_related(&self, entity_id: &str, max_depth: usize) -> Vec<&Entity> {
        if max_depth == 0 || !self.entities.contains_key(entity_id) {
            return Vec::new();
        }

        let mut related = Vec::new();
        let mut visited = HashSet::from([entity_id.to_string()]);
        let mut queue = VecDeque::from([(entity_id, 0usize)]);

        while let Some((current_id, depth)) = queue.pop_front() {
            if depth >= max_depth {
                continue;
            }

            for relation in self.relations_involving(current_id) {
                let next_id = if relation.source_id == current_id {
                    relation.target_id.as_str()
                } else {
                    relation.source_id.as_str()
                };

                if visited.insert(next_id.to_string()) {
                    if let Some(entity) = self.entities.get(next_id) {
                        related.push(entity);
                        queue.push_back((next_id, depth + 1));
                    }
                }
            }
        }

        related
    }

    pub fn find_causal_chains(&self, entity_id: &str) -> Vec<CausalChain> {
        if !self.entities.contains_key(entity_id) {
            return Vec::new();
        }

        let causal_relations: Vec<&Relation> = self
            .relations
            .iter()
            .filter(|relation| Self::is_causal_relation(relation))
            .collect();

        let mut chains = Vec::new();
        for relation in &causal_relations {
            if self.has_causal_predecessor(&relation.source_id, &causal_relations) {
                continue;
            }

            let mut path = vec![relation.source_id.clone()];
            self.collect_causal_paths(
                &relation.source_id,
                &causal_relations,
                &mut path,
                &mut chains,
                entity_id,
            );
        }

        chains
    }

    pub fn search(&self, query: &str, limit: usize) -> Vec<&Entity> {
        let query = query.to_lowercase();
        self.entities
            .values()
            .filter(|entity| {
                entity.name.to_lowercase().contains(&query)
                    || entity.entity_type.to_lowercase().contains(&query)
            })
            .take(limit)
            .collect()
    }

    pub fn get_timeline(&self, entity_id: &str) -> Vec<(String, String)> {
        let mut events: Vec<(String, String)> = self
            .relations_involving(entity_id)
            .filter_map(|relation| {
                let timestamp = relation
                    .metadata
                    .get("created_at")
                    .cloned()
                    .or_else(|| self.relation_entity_created_at(relation));

                timestamp.map(|created_at| {
                    (
                        created_at,
                        format!(
                            "{} -> {} [{}]",
                            relation.source_id, relation.target_id, relation.relation_type
                        ),
                    )
                })
            })
            .collect();

        events.sort_by(|left, right| left.0.cmp(&right.0));
        events
    }

    fn relations_involving<'a>(
        &'a self,
        entity_id: &'a str,
    ) -> impl Iterator<Item = &'a Relation> + 'a {
        self.relations.iter().filter(move |relation| {
            relation.source_id == entity_id || relation.target_id == entity_id
        })
    }

    fn relation_entity_created_at(&self, relation: &Relation) -> Option<String> {
        self.entities
            .get(&relation.target_id)
            .or_else(|| self.entities.get(&relation.source_id))
            .map(|entity| entity.created_at.clone())
    }

    fn is_causal_relation(relation: &Relation) -> bool {
        let relation_type = relation.relation_type.to_lowercase();
        relation_type.contains("caus")
            || relation_type == "leads_to"
            || relation_type == "triggers"
            || relation
                .metadata
                .get("causal")
                .is_some_and(|value| value == "true")
    }

    fn has_causal_predecessor(&self, entity_id: &str, causal_relations: &[&Relation]) -> bool {
        causal_relations
            .iter()
            .any(|relation| relation.target_id == entity_id)
    }

    fn collect_causal_paths(
        &self,
        current_id: &str,
        causal_relations: &[&Relation],
        path: &mut Vec<String>,
        chains: &mut Vec<CausalChain>,
        requested_entity_id: &str,
    ) {
        let mut extended = false;
        for relation in causal_relations
            .iter()
            .filter(|relation| relation.source_id == current_id)
        {
            if path.contains(&relation.target_id) {
                continue;
            }

            path.push(relation.target_id.clone());
            self.collect_causal_paths(
                &relation.target_id,
                causal_relations,
                path,
                chains,
                requested_entity_id,
            );
            path.pop();
            extended = true;
        }

        if !extended && path.len() > 1 && path.iter().any(|id| id == requested_entity_id) {
            let id = format!("causal-chain-{}", chains.len());
            chains.push(CausalChain {
                id,
                events: path.clone(),
                description: self.describe_chain(path),
            });
        }
    }

    fn describe_chain(&self, path: &[String]) -> String {
        path.iter()
            .filter_map(|id| self.entities.get(id).map(|entity| entity.name.as_str()))
            .collect::<Vec<_>>()
            .join(" -> ")
    }
}

impl Default for MDRMGraph {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entity(id: &str, name: &str, entity_type: &str, created_at: &str) -> Entity {
        Entity {
            id: id.to_string(),
            name: name.to_string(),
            entity_type: entity_type.to_string(),
            properties: HashMap::new(),
            created_at: created_at.to_string(),
        }
    }

    #[test]
    fn test_new_graph_is_empty() {
        let graph = MDRMGraph::new();

        assert!(graph.entities.is_empty());
        assert!(graph.relations.is_empty());
    }

    #[test]
    fn test_add_entity_and_retrieve_via_search() {
        let mut graph = MDRMGraph::new();
        graph
            .add_entity(entity("person-1", "Ada Lovelace", "person", "2026-01-01"))
            .unwrap();

        let results = graph.search("ada", 10);

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "person-1");
    }

    #[test]
    fn test_add_relation_and_query_related() {
        let mut graph = MDRMGraph::new();
        graph
            .add_entity(entity("person-1", "Ada", "person", "2026-01-01"))
            .unwrap();
        graph
            .add_entity(entity("project-1", "Compiler", "project", "2026-01-02"))
            .unwrap();
        graph
            .add_relation("person-1", "project-1", "created", 0.9)
            .unwrap();

        let results = graph.query_related("person-1", 1);

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "project-1");
    }

    #[test]
    fn test_bfs_traversal_respects_max_depth() {
        let mut graph = MDRMGraph::new();
        graph
            .add_entity(entity("a", "A", "event", "2026-01-01"))
            .unwrap();
        graph
            .add_entity(entity("b", "B", "event", "2026-01-02"))
            .unwrap();
        graph
            .add_entity(entity("c", "C", "event", "2026-01-03"))
            .unwrap();
        graph.add_relation("a", "b", "related", 1.0).unwrap();
        graph.add_relation("b", "c", "related", 1.0).unwrap();

        let depth_one = graph.query_related("a", 1);
        let depth_two = graph.query_related("a", 2);

        assert_eq!(
            depth_one
                .iter()
                .map(|entity| entity.id.as_str())
                .collect::<Vec<_>>(),
            vec!["b"]
        );
        assert_eq!(
            depth_two
                .iter()
                .map(|entity| entity.id.as_str())
                .collect::<Vec<_>>(),
            vec!["b", "c"]
        );
    }

    #[test]
    fn test_causal_chain_detection() {
        let mut graph = MDRMGraph::new();
        graph
            .add_entity(entity("spark", "Spark", "event", "2026-01-01"))
            .unwrap();
        graph
            .add_entity(entity("fire", "Fire", "event", "2026-01-02"))
            .unwrap();
        graph
            .add_entity(entity("alarm", "Alarm", "event", "2026-01-03"))
            .unwrap();
        graph.add_relation("spark", "fire", "causes", 1.0).unwrap();
        graph.add_relation("fire", "alarm", "causes", 1.0).unwrap();

        let chains = graph.find_causal_chains("fire");

        assert_eq!(chains.len(), 1);
        assert_eq!(chains[0].events, vec!["spark", "fire", "alarm"]);
        assert_eq!(chains[0].description, "Spark -> Fire -> Alarm");
    }
}
