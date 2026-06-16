//! registry — Maintains persona registration, lookup, and default selection.
use crate::core::error::MornError;
use super::*;

pub fn get_persona(id: &str) -> Option<Persona> {
    match id {
        "analyst" => Some(create_analyst_persona()),
        "researcher" => Some(create_researcher_persona()),
        "writer" => Some(create_writer_persona()),
        "coder" => Some(create_coder_persona()),
        "assistant" => Some(create_assistant_persona()),
        "translator" => Some(create_translator_persona()),
        "reviewer" => Some(create_reviewer_persona()),
        "cs_agent" => Some(create_cs_agent_persona()),
        _ => None,
    }
}

pub fn create_default_personas() -> Vec<Persona> {
    vec![
        create_analyst_persona(),
        create_researcher_persona(),
        create_writer_persona(),
        create_coder_persona(),
        create_assistant_persona(),
        create_translator_persona(),
        create_reviewer_persona(),
        create_cs_agent_persona(),
    ]
}

pub fn list_preset_personas() -> Vec<std::collections::HashMap<String, String>> {
    presets::list_preset_personas()
}

pub fn get_preset_persona(name: &str) -> Option<Persona> {
    presets::get_preset_persona(name)
}

pub fn compose(id1: &str, id2: &str, ratio: f32) -> Result<Persona, MornError> {
    let primary = lookup_persona(id1).ok_or_else(|| format!("Persona not found: {}", id1))?;
    let secondary = lookup_persona(id2).ok_or_else(|| format!("Persona not found: {}", id2))?;
    let primary_weight = ratio.clamp(0.0, 1.0) as f64;
    let secondary_weight = 1.0 - primary_weight;

    let mut composed = primary.clone();
    composed.id = format!("composite-{}-{}", primary.id, secondary.id);
    composed.name = format!("{} + {}", primary.name, secondary.name);
    composed.parameters.temperature = weighted(
        primary.parameters.temperature,
        secondary.parameters.temperature,
        primary_weight,
        secondary_weight,
    );
    composed.parameters.verbosity = weighted(
        primary.parameters.verbosity,
        secondary.parameters.verbosity,
        primary_weight,
        secondary_weight,
    );
    composed.parameters.proactiveness = weighted(
        primary.parameters.proactiveness,
        secondary.parameters.proactiveness,
        primary_weight,
        secondary_weight,
    );
    composed.core_principles = merge_unique(&primary.core_principles, &secondary.core_principles);
    composed.decision_framework =
        merge_unique(&primary.decision_framework, &secondary.decision_framework);
    composed.anti_patterns = merge_unique(&primary.anti_patterns, &secondary.anti_patterns);
    Ok(composed)
}

fn lookup_persona(id: &str) -> Option<Persona> {
    presets::get_preset_persona(id).or_else(|| get_persona(id))
}

fn weighted(primary: f64, secondary: f64, primary_weight: f64, secondary_weight: f64) -> f64 {
    primary * primary_weight + secondary * secondary_weight
}

fn merge_unique(primary: &[String], secondary: &[String]) -> Vec<String> {
    let mut merged = Vec::new();
    for value in primary.iter().chain(secondary.iter()) {
        if !merged.contains(value) {
            merged.push(value.clone());
        }
    }
    merged
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_persona_known_ids() {
        let analyst = get_persona("analyst");
        assert!(analyst.is_some());
        assert_eq!(analyst.unwrap().id, "persona-analyst");

        let coder = get_persona("coder");
        assert!(coder.is_some());
        assert_eq!(coder.unwrap().id, "persona-coder");
    }

    #[test]
    fn test_get_persona_unknown_id() {
        assert!(get_persona("nonexistent").is_none());
    }

    #[test]
    fn test_get_persona_all_ids() {
        for id in &[
            "analyst",
            "researcher",
            "writer",
            "coder",
            "assistant",
            "translator",
            "reviewer",
            "cs_agent",
        ] {
            let persona = get_persona(id);
            assert!(persona.is_some(), "Persona {} should exist", id);
        }
    }

    #[test]
    fn test_create_default_personas_count() {
        let personas = create_default_personas();
        assert_eq!(personas.len(), 8);
    }

    #[test]
    fn test_create_default_personas_unique_ids() {
        let personas = create_default_personas();
        let mut ids: Vec<&str> = personas.iter().map(|p| p.id.as_str()).collect();
        ids.sort();
        ids.dedup();
        assert_eq!(ids.len(), personas.len());
    }

    #[test]
    fn test_list_preset_personas_count() {
        let presets = list_preset_personas();
        assert!(presets.len() > 40);
    }

    #[test]
    fn test_list_preset_personas_has_expected_fields() {
        let presets = list_preset_personas();
        for map in &presets {
            assert!(map.contains_key("id"));
            assert!(map.contains_key("name"));
            assert!(map.contains_key("description"));
        }
    }

    #[test]
    fn test_get_preset_persona_known() {
        let p = get_preset_persona("preset-analyst");
        assert!(p.is_some());
        assert_eq!(p.unwrap().id, "preset-analyst");
    }

    #[test]
    fn test_get_preset_persona_unknown() {
        assert!(get_preset_persona("preset-nonexistent").is_none());
    }

    #[test]
    fn test_compose_returns_result() {
        let result = compose("analyst", "coder", 0.7);
        assert!(result.is_ok());
        let persona = result.unwrap();
        assert!(persona.id.starts_with("composite-"));
        assert!(persona.name.contains("+"));
    }

    #[test]
    fn test_compose_with_unknown_primary() {
        let result = compose("nonexistent", "coder", 0.5);
        assert!(result.is_err());
    }

    #[test]
    fn test_compose_with_unknown_secondary() {
        let result = compose("analyst", "nonexistent", 0.5);
        assert!(result.is_err());
    }

    #[test]
    fn test_compose_clamps_ratio() {
        let p = compose("analyst", "coder", 2.0).unwrap();
        assert!(p.parameters.temperature > 0.0);
        let p = compose("analyst", "coder", -1.0).unwrap();
        assert!(p.parameters.temperature > 0.0);
    }

    #[test]
    fn test_compose_merges_principles() {
        let p = compose("analyst", "coder", 0.5).unwrap();
        assert!(!p.core_principles.is_empty());
        assert!(!p.decision_framework.is_empty());
        assert!(!p.anti_patterns.is_empty());
    }

    #[test]
    fn test_weighted() {
        assert!((super::weighted(0.5, 0.7, 0.5, 0.5) - 0.6).abs() < f64::EPSILON);
        assert!((super::weighted(1.0, 0.0, 1.0, 0.0) - 1.0).abs() < f64::EPSILON);
        assert!((super::weighted(0.0, 1.0, 0.0, 1.0) - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_merge_unique() {
        let a = vec!["x".into(), "y".into()];
        let b = vec!["y".into(), "z".into()];
        let merged = super::merge_unique(&a, &b);
        assert_eq!(merged.len(), 3);
        assert!(merged.contains(&"x".into()));
        assert!(merged.contains(&"y".into()));
        assert!(merged.contains(&"z".into()));
    }

    #[test]
    fn test_merge_unique_empty() {
        let a: Vec<String> = vec![];
        let b: Vec<String> = vec!["a".into()];
        let merged = super::merge_unique(&a, &b);
        assert_eq!(merged.len(), 1);
    }
}
