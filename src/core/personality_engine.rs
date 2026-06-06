use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OCEANTraits {
    pub openness: f64,
    pub conscientiousness: f64,
    pub extraversion: f64,
    pub agreeableness: f64,
    pub neuroticism: f64,
}

impl Default for OCEANTraits {
    fn default() -> Self {
        OCEANTraits {
            openness: 0.6,
            conscientiousness: 0.6,
            extraversion: 0.5,
            agreeableness: 0.5,
            neuroticism: 0.3,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum Mood {
    Cheerful,
    #[default]
    Neutral,
    Serious,
    Frustrated,
    Curious,
    Empathetic,
    Calm,
    Energetic,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum CommunicationStyle {
    #[default]
    Casual,
    Formal,
    Playful,
    Concise,
    Encouraging,
    Direct,
    Storyteller,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMParameters {
    pub temperature: f64,
    pub verbosity: f64,
    pub style: CommunicationStyle,
}

impl Default for LLMParameters {
    fn default() -> Self {
        LLMParameters {
            temperature: 0.7,
            verbosity: 0.5,
            style: CommunicationStyle::Casual,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PersonalityProfile {
    pub traits: OCEANTraits,
    pub mood: Mood,
    pub communication_style: CommunicationStyle,
}

#[derive(Debug, Clone, Default)]
pub struct PersonalityEngine {
    profile: PersonalityProfile,
}

impl PersonalityEngine {
    pub fn new(profile: PersonalityProfile) -> Self {
        PersonalityEngine { profile }
    }

    pub fn profile(&self) -> &PersonalityProfile {
        &self.profile
    }

    pub fn set_traits(&mut self, traits: OCEANTraits) {
        self.profile.traits = traits;
    }

    pub fn set_mood(&mut self, mood: Mood) {
        self.profile.mood = mood;
    }

    pub fn set_communication_style(&mut self, style: CommunicationStyle) {
        self.profile.communication_style = style;
    }

    pub fn derive_llm_parameters(&self) -> LLMParameters {
        let temp_base = 0.5;
        let temp_range = 0.4;

        let openness_factor = self.profile.traits.openness * 0.3;
        let extraversion_factor = self.profile.traits.extraversion * 0.2;
        let neuroticism_factor = (1.0 - self.profile.traits.neuroticism) * 0.2;
        let mood_factor = match self.profile.mood {
            Mood::Cheerful | Mood::Energetic => 0.15,
            Mood::Curious => 0.1,
            Mood::Neutral | Mood::Calm => 0.0,
            Mood::Serious | Mood::Empathetic => -0.1,
            Mood::Frustrated => -0.2,
        };
        let temp_adjustment =
            (openness_factor + extraversion_factor + neuroticism_factor + mood_factor)
                .clamp(-temp_range, temp_range);
        let temperature = (temp_base + temp_adjustment).clamp(0.0, 1.0);

        let consc_factor = self.profile.traits.conscientiousness * 0.3;
        let agreeableness_factor = self.profile.traits.agreeableness * 0.2;
        let verbosity = ((consc_factor + agreeableness_factor) * 0.5).clamp(0.0, 1.0);

        LLMParameters {
            temperature,
            verbosity,
            style: self.profile.communication_style.clone(),
        }
    }

    pub fn adjust_temperature(&self, base: f64) -> f64 {
        let params = self.derive_llm_parameters();
        (base + params.temperature - 0.5).clamp(0.0, 1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_traits() {
        let traits = OCEANTraits::default();
        assert!((traits.openness - 0.6).abs() < f64::EPSILON);
        assert!((traits.conscientiousness - 0.6).abs() < f64::EPSILON);
        assert!((traits.extraversion - 0.5).abs() < f64::EPSILON);
        assert!((traits.agreeableness - 0.5).abs() < f64::EPSILON);
        assert!((traits.neuroticism - 0.3).abs() < f64::EPSILON);
    }

    #[test]
    fn test_default_mood() {
        assert!(matches!(Mood::default(), Mood::Neutral));
    }

    #[test]
    fn test_default_communication_style() {
        assert!(matches!(
            CommunicationStyle::default(),
            CommunicationStyle::Casual
        ));
    }

    #[test]
    fn test_derive_default_parameters() {
        let engine = PersonalityEngine::default();
        let params = engine.derive_llm_parameters();
        assert!(params.temperature >= 0.0 && params.temperature <= 1.0);
        assert!(params.verbosity >= 0.0 && params.verbosity <= 1.0);
        assert!(matches!(params.style, CommunicationStyle::Casual));
    }

    #[test]
    fn test_set_traits() {
        let mut engine = PersonalityEngine::default();

        let low_traits = OCEANTraits {
            openness: 0.1,
            conscientiousness: 0.1,
            extraversion: 0.1,
            agreeableness: 0.1,
            neuroticism: 0.9,
        };
        engine.set_traits(low_traits);
        let low_params = engine.derive_llm_parameters();

        let high_traits = OCEANTraits {
            openness: 0.9,
            conscientiousness: 0.9,
            extraversion: 0.9,
            agreeableness: 0.9,
            neuroticism: 0.1,
        };
        engine.set_traits(high_traits);
        let high_params = engine.derive_llm_parameters();

        assert!(high_params.temperature > low_params.temperature);
        assert!(high_params.verbosity > low_params.verbosity);
    }

    #[test]
    fn test_set_mood_cheerful_increases_temperature() {
        let mut engine = PersonalityEngine::default();
        engine.set_mood(Mood::Cheerful);
        let cheerful_params = engine.derive_llm_parameters();

        engine.set_mood(Mood::Neutral);
        let neutral_params = engine.derive_llm_parameters();

        assert!(cheerful_params.temperature >= neutral_params.temperature);
    }

    #[test]
    fn test_set_mood_frustrated_decreases_temperature() {
        let mut engine = PersonalityEngine::default();
        engine.set_mood(Mood::Frustrated);
        let frustrated_params = engine.derive_llm_parameters();

        engine.set_mood(Mood::Neutral);
        let neutral_params = engine.derive_llm_parameters();

        assert!(frustrated_params.temperature <= neutral_params.temperature);
    }

    #[test]
    fn test_adjust_temperature() {
        let engine = PersonalityEngine::default();
        let adjusted = engine.adjust_temperature(0.5);
        assert!(adjusted >= 0.0 && adjusted <= 1.0);
    }

    #[test]
    fn test_high_conscientiousness_increases_verbosity() {
        let mut engine = PersonalityEngine::default();
        let default_params = engine.derive_llm_parameters();

        let high_cons = OCEANTraits {
            conscientiousness: 1.0,
            ..OCEANTraits::default()
        };
        engine.set_traits(high_cons);
        let params = engine.derive_llm_parameters();
        assert!(params.verbosity > default_params.verbosity);
    }

    #[test]
    fn test_profile_accessor() {
        let engine = PersonalityEngine::default();
        let profile = engine.profile();
        assert!((profile.traits.openness - 0.6).abs() < f64::EPSILON);
    }

    #[test]
    fn test_set_communication_style() {
        let mut engine = PersonalityEngine::default();
        engine.set_communication_style(CommunicationStyle::Formal);
        let params = engine.derive_llm_parameters();
        assert!(matches!(params.style, CommunicationStyle::Formal));
    }

    #[test]
    fn test_temperature_stays_in_bounds() {
        let moods = vec![Mood::Cheerful, Mood::Energetic, Mood::Frustrated];
        for mood in &moods {
            let mut engine = PersonalityEngine::default();
            engine.set_mood(mood.clone());
            let params = engine.derive_llm_parameters();
            assert!(
                params.temperature >= 0.0 && params.temperature <= 1.0,
                "Temperature out of bounds for mood {:?}: {}",
                mood,
                params.temperature
            );
        }
    }
}
