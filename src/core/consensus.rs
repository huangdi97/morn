//! consensus — Aggregates multiple agent outputs into consensus decisions.
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct ConsensusFile {
    pub session_id: String,
    pub agent_id: String,
    pub content: String,
    pub stage: String,
    pub created_at: i64,
}

#[derive(Clone)]
pub struct ConsensusManager {
    base_dir: PathBuf,
    sessions: Arc<Mutex<HashMap<String, Vec<ConsensusFile>>>>,
}

impl ConsensusManager {
    pub fn new(base_dir: &Path) -> Self {
        fs::create_dir_all(base_dir).ok();
        ConsensusManager {
            base_dir: base_dir.to_path_buf(),
            sessions: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn publish(
        &self,
        session_id: &str,
        agent_id: &str,
        content: &str,
        stage: &str,
    ) -> Result<String, String> {
        let file = ConsensusFile {
            session_id: session_id.to_string(),
            agent_id: agent_id.to_string(),
            content: content.to_string(),
            stage: stage.to_string(),
            created_at: chrono::Utc::now().timestamp(),
        };

        let file_name = format!("{}_{}_{}.consensus", session_id, agent_id, stage);
        let file_path = self.base_dir.join(&file_name);

        let file_content = serde_json::to_string(&file).map_err(|e| e.to_string())?;
        fs::write(&file_path, &file_content).map_err(|e| e.to_string())?;

        {
            let mut sessions = self.sessions.lock().map_err(|e| e.to_string())?;
            sessions
                .entry(session_id.to_string())
                .or_default()
                .push(file);
        }

        Ok(file_name)
    }

    pub fn read(
        &self,
        session_id: &str,
        agent_id: &str,
        stage: &str,
    ) -> Result<Option<ConsensusFile>, String> {
        let file_name = format!("{}_{}_{}.consensus", session_id, agent_id, stage);
        let file_path = self.base_dir.join(&file_name);

        if !file_path.exists() {
            return Ok(None);
        }

        let content = fs::read_to_string(&file_path).map_err(|e| e.to_string())?;
        let file: ConsensusFile = serde_json::from_str(&content).map_err(|e| e.to_string())?;
        Ok(Some(file))
    }

    pub fn get_session_chain(&self, session_id: &str) -> Result<Vec<ConsensusFile>, String> {
        let sessions = self.sessions.lock().map_err(|e| e.to_string())?;
        Ok(sessions.get(session_id).cloned().unwrap_or_default())
    }

    pub fn replay_session(&self, session_id: &str) -> Result<String, String> {
        let sessions = self.sessions.lock().map_err(|e| e.to_string())?;
        let files = sessions.get(session_id);
        match files {
            None => Ok(format!("No consensus files for session '{}'", session_id)),
            Some(files) => {
                let mut output = format!("=== Consensus Replay: Session {} ===\n", session_id);
                for file in files {
                    output.push_str(&format!(
                        "[{} | {}] stage={}: {}\n",
                        file.created_at, file.agent_id, file.stage, file.content
                    ));
                }
                Ok(output)
            }
        }
    }

    pub fn relay_to_next(
        &self,
        from_session: &str,
        from_agent: &str,
        to_session: &str,
        to_agent: &str,
        content: &str,
    ) -> Result<String, String> {
        self.publish(from_session, from_agent, content, "relay_out")?;
        self.publish(to_session, to_agent, content, "relay_in")?;
        Ok(format!(
            "Relayed consensus from {}/{} to {}/{}",
            from_session, from_agent, to_session, to_agent
        ))
    }

    pub fn find_consensus(&self, keywords: &[&str]) -> Result<Vec<ConsensusFile>, String> {
        let sessions = self.sessions.lock().map_err(|e| e.to_string())?;
        let mut results = Vec::new();
        for files in sessions.values() {
            for file in files {
                if keywords.iter().any(|k| file.content.contains(k)) {
                    results.push(file.clone());
                }
            }
        }
        Ok(results)
    }

    pub fn derive_summary(&self, session_id: &str) -> Result<String, String> {
        let sessions = self.sessions.lock().map_err(|e| e.to_string())?;
        let files = sessions.get(session_id);
        match files {
            None => Ok(String::new()),
            Some(files) => {
                let mut summary = String::new();
                let mut stages: Vec<&str> = files.iter().map(|f| f.stage.as_str()).collect();
                stages.sort();
                stages.dedup();
                for stage in &stages {
                    let stage_files: Vec<&ConsensusFile> = files
                        .iter()
                        .filter(|f| f.stage.as_str() == *stage)
                        .collect();
                    summary.push_str(&format!(
                        "Stage '{}' ({} contributors):\n",
                        stage,
                        stage_files.len()
                    ));
                    for file in &stage_files {
                        summary.push_str(&format!("  - {}: {}\n", file.agent_id, file.content));
                    }
                }
                Ok(summary)
            }
        }
    }

    pub fn scan_persisted(&self) -> Result<Vec<String>, String> {
        let mut files = Vec::new();
        if let Ok(entries) = fs::read_dir(&self.base_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|e| e.to_str()) == Some("consensus") {
                    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                        files.push(name.to_string());
                    }
                }
            }
        }
        Ok(files)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_manager() -> ConsensusManager {
        let dir = std::env::temp_dir().join(format!("consensus_test_{}", uuid::Uuid::new_v4()));
        ConsensusManager::new(&dir)
    }

    #[test]
    fn test_publish_and_read() {
        let mgr = create_test_manager();
        mgr.publish("session-1", "agent-a", "data analysis complete", "analysis")
            .unwrap();
        let read = mgr.read("session-1", "agent-a", "analysis").unwrap();
        assert!(read.is_some());
        assert_eq!(read.unwrap().content, "data analysis complete");
    }

    #[test]
    fn test_read_nonexistent() {
        let mgr = create_test_manager();
        let read = mgr.read("nosession", "noagent", "nostage").unwrap();
        assert!(read.is_none());
    }

    #[test]
    fn test_get_session_chain() {
        let mgr = create_test_manager();
        mgr.publish("session-1", "agent-a", "step1", "plan")
            .unwrap();
        mgr.publish("session-1", "agent-b", "step2", "execute")
            .unwrap();
        mgr.publish("session-2", "agent-c", "other", "test")
            .unwrap();
        let chain = mgr.get_session_chain("session-1").unwrap();
        assert_eq!(chain.len(), 2);
    }

    #[test]
    fn test_replay_session() {
        let mgr = create_test_manager();
        mgr.publish("session-1", "agent-a", "result", "done")
            .unwrap();
        let replay = mgr.replay_session("session-1").unwrap();
        assert!(replay.contains("session-1"));
        assert!(replay.contains("agent-a"));
    }

    #[test]
    fn test_relay_to_next() {
        let mgr = create_test_manager();
        let relay = mgr
            .relay_to_next(
                "session-1",
                "agent-a",
                "session-2",
                "agent-b",
                "handoff data",
            )
            .unwrap();
        assert!(relay.contains("Relayed"));
        let s1 = mgr.get_session_chain("session-1").unwrap();
        let s2 = mgr.get_session_chain("session-2").unwrap();
        assert_eq!(s1.len(), 1);
        assert_eq!(s2.len(), 1);
    }

    #[test]
    fn test_find_consensus() {
        let mgr = create_test_manager();
        mgr.publish("s1", "a1", "revenue analysis shows growth", "report")
            .unwrap();
        mgr.publish("s2", "a2", "code review complete", "qa")
            .unwrap();
        let found = mgr.find_consensus(&["revenue", "growth"]).unwrap();
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].agent_id, "a1");
    }

    #[test]
    fn test_derive_summary() {
        let mgr = create_test_manager();
        mgr.publish("s1", "a1", "plan step 1", "plan").unwrap();
        mgr.publish("s1", "a2", "execute step 2", "execute")
            .unwrap();
        mgr.publish("s1", "a1", "review results", "review").unwrap();
        let summary = mgr.derive_summary("s1").unwrap();
        assert!(summary.contains("plan"));
        assert!(summary.contains("execute"));
        assert!(summary.contains("review"));
    }

    #[test]
    fn test_scan_persisted() {
        let mgr = create_test_manager();
        mgr.publish("s1", "a1", "data", "stage1").unwrap();
        mgr.publish("s1", "a2", "data2", "stage2").unwrap();
        let files = mgr.scan_persisted().unwrap();
        assert_eq!(files.len(), 2);
    }

    #[test]
    fn test_empty_replay() {
        let mgr = create_test_manager();
        let replay = mgr.replay_session("empty").unwrap();
        assert!(replay.contains("No consensus files"));
    }

    #[test]
    fn test_empty_summary() {
        let mgr = create_test_manager();
        let summary = mgr.derive_summary("empty").unwrap();
        assert!(summary.is_empty());
    }
}
