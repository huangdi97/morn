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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Mood {
    Cheerful,
    Neutral,
    Serious,
    Frustrated,
    Curious,
    Empathetic,
    Calm,
    Energetic,
}

impl Default for Mood {
    fn default() -> Self {
        Mood::Neutral
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CommunicationStyle {
    Casual,
    Formal,
    Playful,
    Concise,
    Encouraging,
    Direct,
    Storyteller,
}

impl Default for CommunicationStyle {
    fn default() -> Self {
        CommunicationStyle::Casual
    }
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonalityProfile {
    pub traits: OCEANTraits,
    pub mood: Mood,
    pub communication_style: CommunicationStyle,
}

impl Default for PersonalityProfile {
    fn default() -> Self {
        PersonalityProfile {
            traits: OCEANTraits::default(),
            mood: Mood::default(),
            communication_style: CommunicationStyle::default(),
        }
    }
}

#[derive(Debug, Clone)]
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

impl Default for PersonalityEngine {
    fn default() -> Self {
        PersonalityEngine {
            profile: PersonalityProfile::default(),
        }
    }
}
