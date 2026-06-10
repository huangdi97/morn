//! persona — Defines persona types, registries, presets, and component adapters.
mod builtin;
mod registry;
mod traits;
mod types;

pub mod presets_creative;
pub mod presets_general;
pub mod presets_industry;
pub mod presets_tech;

pub use builtin::*;
pub use presets_creative::*;
pub use presets_general::*;
pub use presets_industry::*;
pub use presets_tech::*;
pub use registry::*;
pub use types::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_persona_create() {
        let p = Persona::new("test-1", "TestBot");
        assert_eq!(p.id, "test-1");
        assert_eq!(p.name, "TestBot");
        assert!(p.core_principles.is_empty());
        assert!(p.anti_patterns.is_empty());
    }

    #[test]
    fn test_persona_default() {
        let params = PersonaParameters::default();
        assert_eq!(params.temperature, 0.6);
        assert_eq!(params.style, "professional");
        assert_eq!(params.verbosity, 0.5);
        assert_eq!(params.proactiveness, 0.3);

        let layers = PromptLayers::default();
        assert_eq!(layers.l1_core_identity, "You are a helpful AI assistant.");
        assert!(layers.l2_skill_instructions.is_none());
    }

    #[test]
    fn test_persona_to_system_prompt() {
        let persona = create_analyst_persona();
        let prompt = persona.build_system_prompt();
        assert!(prompt.contains("Analyst"));
        assert!(prompt.contains("data-driven"));

        let coder = create_coder_persona();
        let coder_prompt = coder.build_system_prompt();
        assert!(coder_prompt.contains("Coder"));
        assert!(coder_prompt.contains("error handling"));
    }

    #[test]
    fn test_cs_agent_persona() {
        let p = create_cs_agent_persona();
        assert_eq!(p.id, "persona-cs-agent");
        assert!(p.parameters.temperature == 0.5);
        assert_eq!(p.parameters.style, "friendly");
    }

    #[test]
    fn test_get_persona_cs_agent() {
        let p = get_persona("cs_agent");
        assert!(p.is_some());
        assert_eq!(p.unwrap().name, "客服");
    }

    #[test]
    fn test_default_personas_count() {
        let personas = create_default_personas();
        assert_eq!(personas.len(), 8);
    }

    #[test]
    fn test_preset_functions() {
        let presets = vec![
            preset_analyst(),
            preset_researcher(),
            preset_writer(),
            preset_coder(),
            preset_translator(),
            preset_assistant(),
            preset_reviewer(),
            preset_cs_agent(),
            preset_investment(),
            preset_medical(),
            preset_legal(),
            preset_tutor(),
            preset_marketing(),
            preset_hr(),
            preset_pm(),
            preset_product(),
            preset_ui_designer(),
            preset_data_engineer(),
            preset_devops(),
            preset_security(),
            preset_qa(),
            preset_tech_writer(),
            preset_social_media(),
            preset_copywriter(),
            preset_editor(),
            preset_journalist(),
            preset_philosopher(),
            preset_psychologist(),
            preset_career_coach(),
            preset_travel_guide(),
            preset_language_tutor(),
            preset_math_tutor(),
            preset_science_tutor(),
            preset_life_coach(),
            preset_fitness(),
            preset_chef(),
            preset_music(),
            preset_history(),
            preset_startup(),
            preset_architect(),
            preset_seo(),
            preset_business_analyst(),
            preset_financial_analyst(),
            preset_trainer(),
            preset_content_moderator(),
            preset_social_assistant(),
            preset_game_designer(),
            preset_video_editor(),
            preset_research_assistant(),
            preset_academic_writer(),
            preset_debate_coach(),
            preset_negotiator(),
        ];
        assert_eq!(presets.len(), 52);
        assert!(presets[0].id.contains("analyst"));
        assert!(presets[7].id.contains("cs-agent"));
        assert!(presets[51].id.contains("negotiator"));
    }

    #[test]
    fn test_list_preset_personas() {
        let list = list_preset_personas();
        assert_eq!(list.len(), 52);
        assert!(list[0].contains_key("id"));
        assert!(list[0].contains_key("name"));
        assert!(list[0].contains_key("description"));
    }

    #[test]
    fn test_get_preset_persona() {
        let p = get_preset_persona("preset-analyst");
        assert!(p.is_some());
        let p = p.unwrap();
        assert_eq!(p.name, "数据分析师");
        assert_eq!(p.parameters.temperature, 0.3);
        assert_eq!(p.parameters.style, "professional");

        let p = get_preset_persona("preset-negotiator");
        assert!(p.is_some());
        let p = p.unwrap();
        assert_eq!(p.name, "谈判专家");

        let invalid = get_preset_persona("nonexistent");
        assert!(invalid.is_none());
    }

    #[test]
    fn test_compose_persona_blends_parameters() {
        let composed = compose("preset-analyst", "preset-writer", 0.75).unwrap();
        let analyst = get_preset_persona("preset-analyst").unwrap();
        let writer = get_preset_persona("preset-writer").unwrap();
        let expected = analyst.parameters.temperature * 0.75 + writer.parameters.temperature * 0.25;

        assert_eq!(composed.parameters.temperature, expected);
        assert!(composed.id.starts_with("composite-"));
        assert!(composed.core_principles.len() >= analyst.core_principles.len());
        assert_eq!(
            composed.prompt_layers.l1_core_identity,
            analyst.prompt_layers.l1_core_identity
        );
    }

    #[test]
    fn test_composite_persona_type_accepts_refs() {
        let composite = CompositePersona {
            primary: "preset-analyst".to_string(),
            secondary: Some("preset-writer".to_string()),
            blend_ratio: 0.6,
            persona_ids: vec!["preset-analyst".to_string(), "preset-writer".to_string()],
            merge_strategy: MergeStrategy::WeightedAverage,
        };

        assert_eq!(composite.primary, "preset-analyst");
        assert_eq!(composite.blend_ratio, 0.6);
    }

    #[test]
    fn test_composite_persona_new() {
        let c = CompositePersona::new(
            vec!["a".to_string(), "b".to_string(), "c".to_string()],
            MergeStrategy::MajorityVote,
        );
        assert_eq!(c.persona_ids.len(), 3);
        assert_eq!(c.merge_strategy, MergeStrategy::MajorityVote);
        assert_eq!(c.primary, "a");
    }

    #[test]
    fn test_merge_majority_vote() {
        let outputs = vec![
            PersonaOutput { persona_id: "a".into(), response: "yes".into(), confidence: 0.8 },
            PersonaOutput { persona_id: "b".into(), response: "yes".into(), confidence: 0.7 },
            PersonaOutput { persona_id: "c".into(), response: "no".into(), confidence: 0.9 },
        ];
        let result = merge_responses(&MergeStrategy::MajorityVote, &outputs);
        assert_eq!(result, "yes");
    }

    #[test]
    fn test_merge_weighted_average() {
        let outputs = vec![
            PersonaOutput { persona_id: "a".into(), response: "answer a".into(), confidence: 0.9 },
            PersonaOutput { persona_id: "b".into(), response: "answer b".into(), confidence: 0.3 },
        ];
        let result = merge_responses(&MergeStrategy::WeightedAverage, &outputs);
        assert_eq!(result, "answer a");
    }

    #[test]
    fn test_merge_sequential() {
        let outputs = vec![
            PersonaOutput { persona_id: "a".into(), response: "first".into(), confidence: 1.0 },
            PersonaOutput { persona_id: "b".into(), response: "second".into(), confidence: 1.0 },
        ];
        let result = merge_responses(&MergeStrategy::Sequential, &outputs);
        assert!(result.contains("first"));
        assert!(result.contains("second"));
    }

    #[test]
    fn test_merge_debate() {
        let outputs = vec![
            PersonaOutput { persona_id: "p1".into(), response: "arg1".into(), confidence: 0.8 },
        ];
        let result = merge_responses(&MergeStrategy::Debate, &outputs);
        assert!(result.contains("Debate Summary"));
        assert!(result.contains("p1"));
    }

    #[test]
    fn test_merge_strategy_label() {
        assert_eq!(MergeStrategy::MajorityVote.label(), "majority_vote");
        assert_eq!(MergeStrategy::WeightedAverage.label(), "weighted_average");
        assert_eq!(MergeStrategy::Sequential.label(), "sequential");
        assert_eq!(MergeStrategy::Debate.label(), "debate");
    }

    #[test]
    fn test_persona_output_struct() {
        let o = PersonaOutput {
            persona_id: "test-id".into(),
            response: "test response".into(),
            confidence: 0.75,
        };
        assert_eq!(o.confidence, 0.75);
        assert_eq!(o.persona_id, "test-id");
    }

    #[test]
    fn test_composite_persona_default() {
        let c: CompositePersona = CompositePersona::default();
        assert!(c.persona_ids.is_empty());
        assert_eq!(c.merge_strategy, MergeStrategy::WeightedAverage);
    }

    #[test]
    fn test_merge_empty_outputs() {
        let result = merge_responses(&MergeStrategy::MajorityVote, &[]);
        assert!(result.is_empty());
    }
}
