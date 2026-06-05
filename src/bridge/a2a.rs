use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentCard {
    pub id: String,
    pub name: String,
    pub description: String,
    pub version: String,
    pub capabilities: Vec<String>,
    pub endpoint: String,
    pub public_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum A2AMessage {
    TaskAssign {
        task_id: String,
        input: String,
        max_tokens: u32,
    },
    TaskStatus {
        task_id: String,
        status: String,
        progress: f64,
    },
    TaskResult {
        task_id: String,
        output: String,
        success: bool,
    },
    AgentDiscovery {
        query: String,
    },
    AgentList {
        agents: Vec<AgentCard>,
    },
    Heartbeat,
    Error {
        code: u32,
        message: String,
    },
}

pub struct A2AProtocol;

impl A2AProtocol {
    pub fn serialize(msg: &A2AMessage) -> Result<String, String> {
        serde_json::to_string(msg).map_err(|e| format!("A2A serialize error: {}", e))
    }

    pub fn deserialize(data: &str) -> Result<A2AMessage, String> {
        serde_json::from_str(data).map_err(|e| format!("A2A deserialize error: {}", e))
    }

    pub fn send(endpoint: &str, msg: &A2AMessage) -> Result<A2AMessage, String> {
        let payload = Self::serialize(msg)?;
        let client = reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .map_err(|e| format!("Failed to create HTTP client: {}", e))?;
        let resp = client
            .post(endpoint)
            .header("Content-Type", "application/json")
            .body(payload)
            .send()
            .map_err(|e| format!("A2A send error: {}", e))?;
        let body = resp
            .text()
            .map_err(|e| format!("A2A read response error: {}", e))?;
        Self::deserialize(&body)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialize_deserialize_task_assign() {
        let msg = A2AMessage::TaskAssign {
            task_id: "task-1".into(),
            input: "hello".into(),
            max_tokens: 4096,
        };
        let data = A2AProtocol::serialize(&msg).unwrap();
        let back = A2AProtocol::deserialize(&data).unwrap();
        match back {
            A2AMessage::TaskAssign {
                task_id,
                input,
                max_tokens,
            } => {
                assert_eq!(task_id, "task-1");
                assert_eq!(input, "hello");
                assert_eq!(max_tokens, 4096);
            }
            _ => panic!("Wrong variant"),
        }
    }

    #[test]
    fn test_serialize_deserialize_heartbeat() {
        let msg = A2AMessage::Heartbeat;
        let data = A2AProtocol::serialize(&msg).unwrap();
        let back = A2AProtocol::deserialize(&data).unwrap();
        assert!(matches!(back, A2AMessage::Heartbeat));
    }

    #[test]
    fn test_serialize_deserialize_agent_card() {
        let card = AgentCard {
            id: "agent-1".into(),
            name: "Research Agent".into(),
            description: "Does research".into(),
            version: "1.0.0".into(),
            capabilities: vec!["web_search".into(), "data_analysis".into()],
            endpoint: "http://localhost:9090".into(),
            public_key: "pubkey123".into(),
        };
        let msg = A2AMessage::AgentList { agents: vec![card] };
        let data = A2AProtocol::serialize(&msg).unwrap();
        let back = A2AProtocol::deserialize(&data).unwrap();
        match back {
            A2AMessage::AgentList { agents } => {
                assert_eq!(agents.len(), 1);
                assert_eq!(agents[0].name, "Research Agent");
            }
            _ => panic!("Wrong variant"),
        }
    }
}
