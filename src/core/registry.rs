use std::collections::HashMap;

use crate::core::event_bus::{SimpleEventBus, EVENT_CHAT_AGENT_RESPONSE, EVENT_SYSTEM_READY};
use crate::core::storage::Storage;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AgentTemplate {
    pub id: String,
    pub name: String,
    pub icon: String,
    pub description: String,
    pub persona: String,
    pub model: String,
    pub tools: Vec<String>,
    pub knowledge: Vec<String>,
    pub skills: Vec<String>,
    pub version: String,
}

#[derive(Clone)]
pub struct Capability {
    pub id: String,
    pub name: String,
    pub domain: String,
    pub actions: Vec<String>,
    pub description: String,
    pub trust_score: f64,
    pub total_calls: u64,
    pub success_calls: u64,
    pub avg_latency_ms: f64,
    pub visibility: String,
    pub owner_id: Option<String>,
    pub team_id: Option<String>,
    pub daily_quota: u64,
    pub version: String,
}

#[derive(Clone)]
pub struct Registry {
    capabilities: HashMap<String, Capability>,
    templates: HashMap<String, AgentTemplate>,
    _storage: Option<Storage>,
    event_bus: Option<SimpleEventBus>,
}

impl Registry {
    pub fn new(storage: Option<Storage>, event_bus: Option<SimpleEventBus>) -> Self {
        let mut registry = Registry {
            capabilities: HashMap::new(),
            templates: HashMap::new(),
            _storage: storage,
            event_bus,
        };

        registry.register_defaults();

        if let Some(ref bus) = registry.event_bus {
            bus.publish_event(
                EVENT_SYSTEM_READY,
                "registry",
                serde_json::json!({"status": "ready"}),
            );
        }

        registry
    }

    fn register_defaults(&mut self) {
        let default_cap = Capability {
            id: "chat-agent".to_string(),
            name: "Chat Agent".to_string(),
            domain: "general".to_string(),
            actions: vec![
                "chat".to_string(),
                "analyze".to_string(),
                "report".to_string(),
            ],
            description: "General purpose chat agent powered by LLM".to_string(),
            trust_score: 70.0,
            total_calls: 0,
            success_calls: 0,
            avg_latency_ms: 0.0,
            visibility: "public".to_string(),
            owner_id: None,
            team_id: None,
            daily_quota: 0,
            version: "0.1.0".to_string(),
        };
        self.capabilities
            .insert(default_cap.id.clone(), default_cap);

        let templates = vec![
            AgentTemplate {
                id: "research-assistant".into(),
                name: "研究助手".into(),
                icon: "🔬".into(),
                description: "多源信息检索、知识整合与摘要生成，适合学术研究和文献综述".into(),
                persona: "researcher".into(),
                model: "deepseek-chat".into(),
                tools: vec!["web_search".into(), "read_file".into()],
                knowledge: vec!["docs".into(), "data_sources".into()],
                skills: vec!["summarization".into(), "report_generation".into()],
                version: "0.1.0".to_string(),
            },
            AgentTemplate {
                id: "data-analyst".into(),
                name: "数据分析师".into(),
                icon: "📊".into(),
                description: "获取行情数据、计算技术指标、生成图表和报告，适合金融与数据领域"
                    .into(),
                persona: "analyst".into(),
                model: "deepseek-chat".into(),
                tools: vec![
                    "get_kline".into(),
                    "calc_macd".into(),
                    "chart".into(),
                    "exec_python".into(),
                ],
                knowledge: vec!["docs".into(), "data_sources".into()],
                skills: vec!["report_generation".into()],
                version: "0.1.0".to_string(),
            },
            AgentTemplate {
                id: "writing-assistant".into(),
                name: "写作助手".into(),
                icon: "✍️".into(),
                description: "多语言翻译、语法检查、格式润色与风格优化，适合内容创作者".into(),
                persona: "writer".into(),
                model: "deepseek-chat".into(),
                tools: vec!["web_search".into(), "read_file".into(), "write_file".into()],
                knowledge: vec!["glossary".into()],
                skills: vec![
                    "translation".into(),
                    "grammar_check".into(),
                    "format".into(),
                    "style".into(),
                    "proofread".into(),
                ],
                version: "0.1.0".to_string(),
            },
            AgentTemplate {
                id: "coding-helper".into(),
                name: "编码助手".into(),
                icon: "💻".into(),
                description: "代码审查、调试、格式化和测试，适合软件开发与编程".into(),
                persona: "coder".into(),
                model: "deepseek-chat".into(),
                tools: vec![
                    "read_file".into(),
                    "write_file".into(),
                    "exec_python".into(),
                ],
                knowledge: vec!["docs".into()],
                skills: vec![
                    "code_review".into(),
                    "debug".into(),
                    "format".into(),
                    "test".into(),
                ],
                version: "0.1.0".to_string(),
            },
            AgentTemplate {
                id: "translation-agent".into(),
                name: "翻译 Agent".into(),
                icon: "🌐".into(),
                description: "多语言翻译、校对和专业术语管理，适合跨语言工作".into(),
                persona: "translator".into(),
                model: "deepseek-chat".into(),
                tools: vec!["web_search".into(), "read_file".into()],
                knowledge: vec!["glossary".into()],
                skills: vec!["translation".into(), "proofread".into()],
                version: "0.1.0".to_string(),
            },
            AgentTemplate {
                id: "general-assistant".into(),
                name: "通用助手".into(),
                icon: "🤖".into(),
                description: "混合工具集的通用助手，适合日常问答、搜索和简单任务".into(),
                persona: "assistant".into(),
                model: "deepseek-chat".into(),
                tools: vec![
                    "web_search".into(),
                    "read_file".into(),
                    "get_time".into(),
                    "calc".into(),
                ],
                knowledge: vec!["docs".into()],
                skills: vec![],
                version: "0.1.0".to_string(),
            },
        ];

        for t in templates {
            self.templates.insert(t.id.clone(), t);
        }
    }

    pub fn register(&mut self, capability: Capability) {
        if let Some(ref bus) = self.event_bus {
            bus.publish_event(
                EVENT_CHAT_AGENT_RESPONSE,
                "registry",
                serde_json::json!({"action": "register", "capability_id": capability.id}),
            );
        }
        self.capabilities.insert(capability.id.clone(), capability);
    }

    pub fn unregister(&mut self, id: &str) -> Option<Capability> {
        self.capabilities.remove(id)
    }

    pub fn find_by_domain(&self, domain: &str) -> Vec<&Capability> {
        self.capabilities
            .values()
            .filter(|c| c.domain == domain)
            .collect()
    }

    pub fn find_by_action(&self, action: &str) -> Vec<&Capability> {
        self.capabilities
            .values()
            .filter(|c| c.actions.iter().any(|a| a == action))
            .collect()
    }

    pub fn list_all(&self) -> Vec<&Capability> {
        self.capabilities.values().collect()
    }

    pub fn list_available(&self, user_id: Option<&str>, user_teams: &[String]) -> Vec<&Capability> {
        self.capabilities
            .values()
            .filter(|c| match c.visibility.as_str() {
                "public" => true,
                "private" => {
                    if let Some(uid) = user_id {
                        c.owner_id.as_deref() == Some(uid)
                    } else {
                        false
                    }
                }
                "team" => {
                    if let Some(ref tid) = c.team_id {
                        user_teams.iter().any(|ut| ut == tid)
                    } else {
                        false
                    }
                }
                _ => true,
            })
            .collect()
    }

    pub fn get(&self, id: &str) -> Option<&Capability> {
        self.capabilities.get(id)
    }

    pub fn get_mut(&mut self, id: &str) -> Option<&mut Capability> {
        self.capabilities.get_mut(id)
    }

    pub fn update_trust_score(&mut self, id: &str, success: bool, latency_ms: f64) {
        if let Some(cap) = self.capabilities.get_mut(id) {
            cap.total_calls += 1;
            if success {
                cap.success_calls += 1;
            }

            let execution_success = if cap.total_calls > 0 {
                cap.success_calls as f64 / cap.total_calls as f64
            } else {
                0.0
            };

            let latency_score = if latency_ms > 0.0 {
                (1000.0 / latency_ms).min(1.0)
            } else {
                0.0
            };

            cap.avg_latency_ms = if cap.total_calls > 1 {
                (cap.avg_latency_ms * (cap.total_calls as f64 - 1.0) + latency_ms)
                    / cap.total_calls as f64
            } else {
                latency_ms
            };

            cap.trust_score =
                70.0 * 0.3 + execution_success * 30.0 + latency_score * 20.0 + 50.0 * 0.2;
        }
    }

    pub fn list_templates(&self) -> Vec<&AgentTemplate> {
        self.templates.values().collect()
    }

    pub fn get_template(&self, id: &str) -> Option<&AgentTemplate> {
        self.templates.get(id)
    }

    pub fn get_version(&self, id: &str) -> Option<&str> {
        self.capabilities.get(id).map(|c| c.version.as_str())
    }

    pub fn list_by_version(&self, version: &str) -> Vec<&Capability> {
        self.capabilities
            .values()
            .filter(|c| c.version == version)
            .collect()
    }

    pub fn check_conflict(&self, id: &str, version: &str) -> bool {
        self.capabilities
            .get(id)
            .map(|c| c.version != version)
            .unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_cap(id: &str, domain: &str) -> Capability {
        Capability {
            id: id.to_string(),
            name: format!("cap-{}", id),
            domain: domain.to_string(),
            actions: vec!["act".to_string()],
            description: String::new(),
            trust_score: 50.0,
            total_calls: 0,
            success_calls: 0,
            avg_latency_ms: 0.0,
            visibility: "public".to_string(),
            owner_id: None,
            team_id: None,
            daily_quota: 0,
            version: "0.1.0".to_string(),
        }
    }

    #[test]
    fn test_register_and_get() {
        let mut reg = Registry::new(None, None);
        let cap = make_cap("test-cap-1", "test-domain");
        reg.register(cap);
        let got = reg.get("test-cap-1");
        assert!(got.is_some());
        assert_eq!(got.unwrap().domain, "test-domain");
    }

    #[test]
    fn test_unregister_returns_none() {
        let mut reg = Registry::new(None, None);
        let cap = make_cap("test-cap-2", "other");
        reg.register(cap);
        assert!(reg.get("test-cap-2").is_some());

        let removed = reg.unregister("test-cap-2");
        assert!(removed.is_some());
        assert!(reg.get("test-cap-2").is_none());
    }

    #[test]
    fn test_list_all() {
        let mut reg = Registry::new(None, None);
        let initial_count = reg.list_all().len();

        reg.register(make_cap("extra-1", "x"));
        reg.register(make_cap("extra-2", "y"));

        assert_eq!(reg.list_all().len(), initial_count + 2);
    }

    #[test]
    fn test_duplicate_register_overwrites() {
        let mut reg = Registry::new(None, None);
        let cap_a = make_cap("dup-key", "domain-a");
        reg.register(cap_a);

        let cap_b = make_cap("dup-key", "domain-b");
        reg.register(cap_b);

        let got = reg.get("dup-key").unwrap();
        assert_eq!(got.domain, "domain-b");
    }

    #[test]
    fn test_get_version() {
        let mut reg = Registry::new(None, None);
        let mut cap = make_cap("ver-cap", "v-domain");
        cap.version = "2.0.0".to_string();
        reg.register(cap);

        assert_eq!(reg.get_version("ver-cap"), Some("2.0.0"));
        assert_eq!(reg.get_version("nonexistent"), None);
    }

    #[test]
    fn test_list_by_version() {
        let mut reg = Registry::new(None, None);
        let mut cap1 = make_cap("v1", "d1");
        cap1.version = "1.0.0".to_string();
        let mut cap2 = make_cap("v2", "d2");
        cap2.version = "1.0.0".to_string();
        let mut cap3 = make_cap("v3", "d3");
        cap3.version = "2.0.0".to_string();

        reg.register(cap1);
        reg.register(cap2);
        reg.register(cap3);

        assert_eq!(reg.list_by_version("1.0.0").len(), 2);
        assert_eq!(reg.list_by_version("2.0.0").len(), 1);
        assert_eq!(reg.list_by_version("3.0.0").len(), 0);
    }

    #[test]
    fn test_check_conflict() {
        let mut reg = Registry::new(None, None);
        let mut cap = make_cap("conflict-cap", "d");
        cap.version = "1.5.0".to_string();
        reg.register(cap);

        assert!(!reg.check_conflict("conflict-cap", "1.5.0"));
        assert!(reg.check_conflict("conflict-cap", "2.0.0"));
        assert!(!reg.check_conflict("no-such-cap", "1.0.0"));
    }
}
