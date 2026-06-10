//! personality_engine — Manages agent personality traits and communication style.
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum PersonalityTrait {
    Enthusiastic,
    Analytical,
    Empathetic,
    Playful,
    Professional,
    Casual,
}

#[derive(Debug, Clone)]
pub struct PersonalityEngine {
    traits: Vec<PersonalityTrait>,
    style_overrides: HashMap<String, String>,
    energy_level: f64,
}

impl Default for PersonalityEngine {
    fn default() -> Self {
        Self {
            traits: vec![PersonalityTrait::Professional],
            style_overrides: HashMap::new(),
            energy_level: 0.5,
        }
    }
}

impl PersonalityEngine {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_trait(mut self, trait_: PersonalityTrait) -> Self {
        self.traits.push(trait_);
        self
    }

    pub fn set_energy(&mut self, level: f64) {
        self.energy_level = level.clamp(0.0, 1.0);
    }

    pub fn greeting_prefix(&self) -> &str {
        if self.traits.contains(&PersonalityTrait::Enthusiastic) {
            "Hey there!"
        } else if self.traits.contains(&PersonalityTrait::Playful) {
            "Well, well, well!"
        } else if self.traits.contains(&PersonalityTrait::Casual) {
            "Hey"
        } else {
            "Hello"
        }
    }

    pub fn traits(&self) -> &[PersonalityTrait] {
        &self.traits
    }

    pub fn energy(&self) -> f64 {
        self.energy_level
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_is_professional() {
        let engine = PersonalityEngine::new();
        assert_eq!(engine.traits(), &[PersonalityTrait::Professional]);
        assert_eq!(engine.energy(), 0.5);
    }

    #[test]
    fn test_with_trait_adds_to_traits() {
        let engine = PersonalityEngine::new()
            .with_trait(PersonalityTrait::Analytical)
            .with_trait(PersonalityTrait::Playful);
        assert_eq!(
            engine.traits(),
            &[
                PersonalityTrait::Professional,
                PersonalityTrait::Analytical,
                PersonalityTrait::Playful
            ]
        );
    }

    #[test]
    fn test_set_energy_clamps() {
        let mut engine = PersonalityEngine::new();
        engine.set_energy(1.5);
        assert_eq!(engine.energy(), 1.0);
        engine.set_energy(-0.1);
        assert_eq!(engine.energy(), 0.0);
    }

    #[test]
    fn test_greeting_prefix_by_trait() {
        let enthusiastic = PersonalityEngine::new().with_trait(PersonalityTrait::Enthusiastic);
        assert_eq!(enthusiastic.greeting_prefix(), "Hey there!");

        let playful = PersonalityEngine::new().with_trait(PersonalityTrait::Playful);
        assert_eq!(playful.greeting_prefix(), "Well, well, well!");

        let casual = PersonalityEngine::new().with_trait(PersonalityTrait::Casual);
        assert_eq!(casual.greeting_prefix(), "Hey");

        let professional = PersonalityEngine::new();
        assert_eq!(professional.greeting_prefix(), "Hello");
    }
}
