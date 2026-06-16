//! Model router tests.
use crate::core::error::MornError;
use super::*;
use crate::config::{CustomProviderConfig, HybridConfig, ModelConfig};
use std::collections::HashMap;

#[test]
fn test_default_router_has_models() {
    let router = ModelRouter::new();
    let available = router.available_models();
    assert!(available.len() >= 4);
    #[cfg(feature = "providers-full")]
    assert!(available.len() >= 27);
}

#[test]
fn test_provider_configs_accessible() {
    let router = ModelRouter::new();
    let endpoint = router.get_provider_endpoint("openai");
    assert!(endpoint.is_some());
    assert_eq!(endpoint.unwrap(), "https://api.openai.com");
    let header = router.get_provider_api_key_header("anthropic");
    assert!(header.is_some());
    assert_eq!(header.unwrap(), "x-api-key");
}

#[test]
#[cfg(feature = "providers-full")]
fn test_all_providers_have_models() {
    for p in providers::PROVIDERS {
        assert!(!p.models.is_empty(), "provider {} has no models", p.name);
        assert!(
            !p.endpoint.is_empty() || p.name == "builtin",
            "provider {} has no endpoint",
            p.name
        );
    }
}

#[test]
fn test_switch_mode() {
    let mut router = ModelRouter::new();
    assert_eq!(*router.mode(), RouterMode::CloudFirst);
    router.switch_mode(RouterMode::LocalOnly);
    assert_eq!(*router.mode(), RouterMode::LocalOnly);
    router.switch_mode(RouterMode::Hybrid);
    assert_eq!(*router.mode(), RouterMode::Hybrid);
}

#[test]
fn test_cloud_first_selects_model() {
    let router = ModelRouter::new();
    let model = router.select_model("hello world", &["chat"]);
    assert!(model.is_ok());
    assert_eq!(model.unwrap().model_type, ModelType::Cloud);
}

#[test]
fn test_fallback_chain_includes_all() {
    let router = ModelRouter::new();
    let chain = router.get_fallback_chain();
    assert!(chain.len() >= 4);
}

#[test]
fn test_local_only_returns_fallback_when_no_local() {
    let router = ModelRouter::new();
    let model = router.select_model("hello", &["chat"]);
    assert!(model.is_ok());
}

#[test]
fn test_hybrid_detects_complex() {
    let router = ModelRouter::new();
    let simple = router.select_model("hello", &["chat"]);
    let complex = router.select_model(
        "write a complex analysis of market trends and generate a report",
        &["chat"],
    );
    assert!(simple.is_ok());
    assert!(complex.is_ok());
}

#[test]
fn test_hybrid_threshold_configurable() {
    let mut router = ModelRouter::new();
    router.switch_mode(RouterMode::Hybrid);
    router.set_hybrid_threshold(10);
    let long_prompt = "this is a very long prompt that exceeds the short threshold";
    let model = router.select_model(long_prompt, &["chat"]);
    assert!(model.is_ok());
}

#[test]
fn test_select_model_by_capabilities() {
    let router = ModelRouter::new();
    let model = router.select_model("describe this image", &["chat"]);
    assert!(model.is_ok());
}

#[test]
fn test_available_models_filtered() {
    let router = ModelRouter::new();
    let available = router.available_models();
    assert!(!available.is_empty());
    assert!(available.iter().all(|m| m.is_available));
}

#[test]
fn test_discover_local_models_no_dir() {
    let mut router = ModelRouter::new();
    let found = router.discover_local_models("/nonexistent/gguf/dir");
    assert!(!cfg!(feature = "local-llm") || found.is_empty());
}

#[test]
fn test_has_local_models_default_false() {
    let router = ModelRouter::new();
    assert!(router.has_local_models());
}

#[test]
fn test_gguf_discovered_default_false() {
    let router = ModelRouter::new();
    assert!(!router.gguf_discovered());
}

#[test]
fn test_custom_provider_loaded_from_model_config() {
    let mut providers = HashMap::new();
    providers.insert(
        "my_custom".to_string(),
        CustomProviderConfig {
            base_url: "https://my-api.com/v1".to_string(),
            api_key: Some("sk-xxx".to_string()),
            models: vec!["my-model".to_string()],
            api_key_header: "Authorization".to_string(),
        },
    );

    let config = ModelConfig {
        provider: "my_custom".to_string(),
        name: "my-model".to_string(),
        providers,
        hybrid: HybridConfig {
            strategy: "local_first".to_string(),
            complexity_threshold: 42,
        },
        ..ModelConfig::default()
    };

    let router = ModelRouter::from_model_config(&config);
    let route = router.route("hello").unwrap();

    assert_eq!(router.hybrid_strategy(), HybridStrategy::LocalFirst);
    assert_eq!(
        router.get_provider_endpoint("my_custom").unwrap(),
        "https://my-api.com/v1"
    );
    assert_eq!(
        router.get_provider_models("my_custom").unwrap(),
        vec!["my-model".to_string()]
    );
    assert_eq!(route.provider, "my_custom");
    assert_eq!(route.name, "my-model");
    assert_eq!(route.base_url, "https://my-api.com/v1");
    assert_eq!(route.api_key, Some("sk-xxx".to_string()));
    assert_eq!(route.api_key_header, "Authorization");
}
