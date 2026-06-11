//! 预设人格集合 — 内置的行业/通用/创意三类预设
use super::Persona;

pub fn load_preset_from_file(name: &str) -> Option<Persona> {
    let path = format!("src/component/persona/presets/{}.json", name);
    let content = std::fs::read_to_string(&path).ok()?;
    serde_json::from_str(&content).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_assistant_preset_from_file() {
        let persona = load_preset_from_file("assistant");
        assert!(persona.is_some());
        let persona = persona.unwrap();
        assert_eq!(persona.name, "系统管家");
        assert_eq!(persona.id, "preset-assistant");
    }
}
