//! learning::preferences — Component preference tracking and ranking.

use std::collections::HashMap;

use super::LearningEngine;

impl LearningEngine {
    pub fn learn_component_preference(
        &self,
        component_name: &str,
        added: bool,
    ) -> Result<Vec<String>, String> {
        let component_name = component_name.trim();
        if component_name.is_empty() {
            return Err("component preference name cannot be empty".to_string());
        }

        let mut preferences = self.load_component_preferences()?;
        let delta = if added { 1 } else { -1 };
        *preferences.entry(component_name.to_string()).or_insert(0) += delta;

        self.save_component_preferences(&preferences)?;
        Ok(rank_component_preferences(&preferences))
    }

    pub fn component_recommendations(&self) -> Vec<String> {
        self.load_component_preferences()
            .map(|preferences| rank_component_preferences(&preferences))
            .unwrap_or_default()
    }

    pub(crate) fn load_component_preferences(&self) -> Result<HashMap<String, i64>, String> {
        if let Some(storage) = &self.storage {
            let preferences = match storage.get_setting(super::COMPONENT_PREFERENCES_KEY)? {
                Some(value) => {
                    serde_json::from_str::<HashMap<String, i64>>(&value).unwrap_or_default()
                }
                None => HashMap::new(),
            };
            let mut cache = self
                .component_preferences
                .lock()
                .map_err(|e| e.to_string())?;
            *cache = preferences.clone();
            return Ok(preferences);
        }

        self.component_preferences
            .lock()
            .map(|cache| cache.clone())
            .map_err(|e| e.to_string())
    }

    pub(crate) fn save_component_preferences(
        &self,
        preferences: &HashMap<String, i64>,
    ) -> Result<(), String> {
        {
            let mut cache = self
                .component_preferences
                .lock()
                .map_err(|e| e.to_string())?;
            *cache = preferences.clone();
        }

        if let Some(storage) = &self.storage {
            let value = serde_json::to_string(preferences).map_err(|e| e.to_string())?;
            storage.set_setting(super::COMPONENT_PREFERENCES_KEY, &value)?;
        }
        Ok(())
    }
}

fn rank_component_preferences(preferences: &HashMap<String, i64>) -> Vec<String> {
    let mut ranked = preferences
        .iter()
        .filter(|(_, score)| **score > 0)
        .map(|(component, score)| (component.clone(), *score))
        .collect::<Vec<_>>();
    ranked.sort_by(|(left_name, left_score), (right_name, right_score)| {
        right_score
            .cmp(left_score)
            .then_with(|| left_name.cmp(right_name))
    });
    ranked
        .into_iter()
        .map(|(component, _score)| component)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::storage::Storage;

    #[test]
    fn learn_component_preference_ranks_added_components() {
        let engine = LearningEngine::new(None, 50.0);

        engine
            .learn_component_preference("web_search", true)
            .unwrap();
        let recommendations = engine
            .learn_component_preference("data_analysis", true)
            .unwrap();

        assert_eq!(recommendations, vec!["data_analysis", "web_search"]);
    }

    #[test]
    fn learn_component_preference_removes_from_recommendations() {
        let engine = LearningEngine::new(None, 50.0);

        engine
            .learn_component_preference("web_search", true)
            .unwrap();
        let recommendations = engine
            .learn_component_preference("web_search", false)
            .unwrap();

        assert!(recommendations.is_empty());
    }

    #[test]
    fn learn_component_preference_persists_with_storage() {
        let storage = Storage::new_in_memory().unwrap();
        let engine = LearningEngine::new(Some(storage.clone()), 50.0);
        engine.learn_component_preference("chart", true).unwrap();

        let next_engine = LearningEngine::new(Some(storage), 50.0);

        assert_eq!(next_engine.component_recommendations(), vec!["chart"]);
    }
}