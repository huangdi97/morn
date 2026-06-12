use super::super::{ModelSpec, ProviderCatalogEntry};

pub(super) fn has_all_capabilities(model: &ModelSpec, required: &[&str]) -> bool {
    required
        .iter()
        .all(|cap| model.capabilities.iter().any(|c| c == cap))
}

pub(super) fn default_provider_catalog() -> Vec<ProviderCatalogEntry> {
    super::super::providers::PROVIDERS
        .iter()
        .map(|provider| ProviderCatalogEntry {
            name: provider.name.to_string(),
            endpoint: provider.endpoint.to_string(),
            api_key_header: provider.api_key_header.to_string(),
            models: provider
                .models
                .iter()
                .map(|model| model.to_string())
                .collect(),
            api_key: None,
        })
        .collect()
}

pub(super) fn is_local_provider(provider: &str) -> bool {
    provider == "ollama" || provider == "lm_studio" || provider == "local"
}

pub(super) fn estimate_complexity(prompt: &str) -> usize {
    let lower = prompt.to_ascii_lowercase();
    let keyword_score = [
        "analyze", "compare", "explain", "write", "code", "generate", "design", "plan",
    ]
    .iter()
    .filter(|keyword| lower.contains(**keyword))
    .count()
        * 100;

    prompt.len() + keyword_score
}

pub(super) fn endpoint_from_provider(provider: &str) -> String {
    if provider.starts_with("http://") || provider.starts_with("https://") {
        provider.to_string()
    } else if provider.contains('.') {
        format!("https://{}", provider)
    } else {
        String::new()
    }
}
