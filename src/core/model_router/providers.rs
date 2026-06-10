#[derive(Debug, Clone)]
pub struct ProviderConfig {
    pub name: &'static str,
    pub endpoint: &'static str,
    pub api_key_header: &'static str,
    pub models: &'static [&'static str],
}

pub const PROVIDERS: &[ProviderConfig] = &[
    ProviderConfig {
        name: "deepseek",
        endpoint: "https://api.deepseek.com",
        api_key_header: "Authorization",
        models: &["deepseek-chat", "deepseek-coder"],
    },
    ProviderConfig {
        name: "openai",
        endpoint: "https://api.openai.com",
        api_key_header: "Authorization",
        models: &[
            "gpt-4o",
            "gpt-4o-mini",
            "gpt-4-turbo",
            "gpt-3.5-turbo",
            "o1",
            "o1-mini",
        ],
    },
    ProviderConfig {
        name: "anthropic",
        endpoint: "https://api.anthropic.com",
        api_key_header: "x-api-key",
        models: &["claude-3-opus", "claude-3-sonnet", "claude-3-haiku"],
    },
    ProviderConfig {
        name: "sensenova",
        endpoint: "https://api.sensenova.cn",
        api_key_header: "Authorization",
        models: &["sensenova"],
    },
    ProviderConfig {
        name: "builtin",
        endpoint: "",
        api_key_header: "",
        models: &["tiny-llm"],
    },
    ProviderConfig {
        name: "local",
        endpoint: "http://localhost:8080",
        api_key_header: "",
        models: &["local-gguf"],
    },
    ProviderConfig {
        name: "ollama",
        endpoint: "http://localhost:11434",
        api_key_header: "",
        models: &[
            "llama3",
            "llama3:70b",
            "mistral",
            "mixtral",
            "codellama",
            "phi-3",
            "gemma",
        ],
    },
    ProviderConfig {
        name: "lm_studio",
        endpoint: "http://localhost:1234",
        api_key_header: "",
        models: &["lm-studio-default"],
    },
    ProviderConfig {
        name: "groq",
        endpoint: "https://api.groq.com/openai/v1",
        api_key_header: "Authorization",
        models: &[
            "llama3-70b-8192",
            "llama3-8b-8192",
            "mixtral-8x7b-32768",
            "gemma2-9b-it",
        ],
    },
    ProviderConfig {
        name: "together",
        endpoint: "https://api.together.xyz",
        api_key_header: "Authorization",
        models: &[
            "together-gpt-jt-7b",
            "together-llama-3-70b",
            "together-mixtral-8x7b",
        ],
    },
    ProviderConfig {
        name: "perplexity",
        endpoint: "https://api.perplexity.ai",
        api_key_header: "Authorization",
        models: &[
            "sonar-small-chat",
            "sonar-medium-chat",
            "sonar-large-chat",
        ],
    },
    ProviderConfig {
        name: "mistral",
        endpoint: "https://api.mistral.ai",
        api_key_header: "Authorization",
        models: &[
            "mistral-tiny",
            "mistral-small",
            "mistral-medium",
            "mistral-large",
        ],
    },
    ProviderConfig {
        name: "cohere",
        endpoint: "https://api.cohere.com",
        api_key_header: "Authorization",
        models: &[
            "command-r",
            "command-r-plus",
            "command",
        ],
    },
    ProviderConfig {
        name: "gemini",
        endpoint: "https://generativelanguage.googleapis.com",
        api_key_header: "x-goog-api-key",
        models: &[
            "gemini-1.5-pro",
            "gemini-1.5-flash",
            "gemini-1.0-pro",
        ],
    },
    ProviderConfig {
        name: "xai",
        endpoint: "https://api.x.ai",
        api_key_header: "Authorization",
        models: &["grok-1", "grok-1-mini"],
    },
    ProviderConfig {
        name: "fireworks",
        endpoint: "https://api.fireworks.ai",
        api_key_header: "Authorization",
        models: &[
            "fireworks-llama-v3-70b",
            "fireworks-llama-v3-8b",
            "fireworks-mixtral-8x7b",
        ],
    },
    ProviderConfig {
        name: "replicate",
        endpoint: "https://api.replicate.com",
        api_key_header: "Authorization",
        models: &[
            "replicate-llama-3-70b",
            "replicate-mistral-7b",
            "replicate-stable-diffusion",
        ],
    },
    ProviderConfig {
        name: "huggingface",
        endpoint: "https://api-inference.huggingface.co",
        api_key_header: "Authorization",
        models: &[
            "huggingface-mistral-7b",
            "huggingface-llama-3-70b",
            "huggingface-zephyr-7b",
        ],
    },
    ProviderConfig {
        name: "anyscale",
        endpoint: "https://api.endpoints.anyscale.com",
        api_key_header: "Authorization",
        models: &[
            "anyscale-llama-3-70b",
            "anyscale-mixtral-8x7b",
            "anyscale-mistral-7b",
        ],
    },
    ProviderConfig {
        name: "together_coder",
        endpoint: "https://api.together.xyz",
        api_key_header: "Authorization",
        models: &[
            "together-coder-llama-34b",
            "together-coder-deepseek-33b",
            "together-coder-phind-34b",
        ],
    },
    ProviderConfig {
        name: "deepinfra",
        endpoint: "https://api.deepinfra.com",
        api_key_header: "Authorization",
        models: &[
            "deepinfra-llama-3-70b",
            "deepinfra-mixtral-8x7b",
            "deepinfra-mistral-7b",
        ],
    },
    ProviderConfig {
        name: "lepton",
        endpoint: "https://api.lepton.ai",
        api_key_header: "Authorization",
        models: &[
            "lepton-llama-3-70b",
            "lepton-mixtral-8x7b",
        ],
    },
    ProviderConfig {
        name: "novita",
        endpoint: "https://api.novita.ai",
        api_key_header: "Authorization",
        models: &[
            "novita-llama-3-70b",
            "novita-mixtral-8x7b",
        ],
    },
    ProviderConfig {
        name: "openrouter",
        endpoint: "https://openrouter.ai/api",
        api_key_header: "Authorization",
        models: &[
            "openrouter-auto",
            "openrouter-llama-3-70b",
            "openrouter-mixtral-8x7b",
        ],
    },
    ProviderConfig {
        name: "sambanova",
        endpoint: "https://api.sambanova.ai",
        api_key_header: "Authorization",
        models: &[
            "sambanova-llama-3-70b",
            "sambanova-mixtral-8x7b",
        ],
    },
    ProviderConfig {
        name: "together_embeddings",
        endpoint: "https://api.together.xyz",
        api_key_header: "Authorization",
        models: &[
            "together-embed-m2-bert",
            "together-embed-bge-base",
            "together-embed-bge-large",
        ],
    },
    ProviderConfig {
        name: "voyage",
        endpoint: "https://api.voyageai.com",
        api_key_header: "Authorization",
        models: &[
            "voyage-2",
            "voyage-large-2",
            "voyage-code-2",
        ],
    },
];

pub fn get_provider(name: &str) -> Option<&'static ProviderConfig> {
    PROVIDERS.iter().find(|p| p.name == name)
}
