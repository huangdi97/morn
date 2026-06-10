use super::Persona;

#[derive(Debug, Clone)]
pub struct CombinedPersona {
    pub base: Persona,
    pub traits: Vec<String>,
    pub blend_ratio: f64,
}

impl CombinedPersona {
    pub fn new(base: Persona) -> Self {
        CombinedPersona {
            base,
            traits: Vec::new(),
            blend_ratio: 0.5,
        }
    }

    pub fn combine(&mut self, other: &Persona, ratio: f64) {
        self.blend_ratio = ratio.clamp(0.0, 1.0);
        self.traits.push(other.name.clone());
        self.base.parameters.temperature = self.base.parameters.temperature
            * (1.0 - self.blend_ratio)
            + other.parameters.temperature * self.blend_ratio;
        for principle in &other.core_principles {
            if !self.base.core_principles.contains(principle) {
                self.base.core_principles.push(principle.clone());
            }
        }
    }

    pub fn to_persona(&self) -> Persona {
        let mut persona = self.base.clone();
        persona.name = format!("{} (blended)", persona.name);
        persona
    }

    pub fn blended_traits(&self) -> &[String] {
        &self.traits
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_combined_persona_new() {
        let base = Persona::new("base-1", "BasePersona");
        let combined = CombinedPersona::new(base);
        assert!(combined.traits.is_empty());
        assert_eq!(combined.blend_ratio, 0.5);
    }

    #[test]
    fn test_combine_adds_trait() {
        let base = Persona::new("base-1", "BasePersona");
        let other = Persona::new("other-1", "OtherPersona");
        let mut combined = CombinedPersona::new(base);
        combined.combine(&other, 0.8);
        assert!(combined.traits.contains(&"OtherPersona".to_string()));
        assert_eq!(combined.blend_ratio, 0.8);
    }

    #[test]
    fn test_to_persona_includes_blend_name() {
        let base = Persona::new("base-1", "BasePersona");
        let mut combined = CombinedPersona::new(base);
        let other = Persona::new("other-1", "OtherPersona");
        combined.combine(&other, 0.3);
        let persona = combined.to_persona();
        assert!(persona.name.contains("blended"));
    }
}
