use super::{CardStatus, Priority};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct KanbanCard {
    pub task_id: String,
    pub agent_id: Option<String>,
    pub status: CardStatus,
    pub priority: Priority,
    pub title: String,
    pub description: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

impl KanbanCard {
    pub fn new(task_id: String, title: String) -> Self {
        let now = chrono::Utc::now().to_rfc3339();
        KanbanCard {
            task_id,
            agent_id: None,
            status: CardStatus::Todo,
            priority: Priority::Medium,
            title,
            description: None,
            created_at: now.clone(),
            updated_at: now,
        }
    }

    pub fn assign(&mut self, agent_id: String) {
        self.agent_id = Some(agent_id);
        self.updated_at = chrono::Utc::now().to_rfc3339();
    }

    pub fn move_to(&mut self, status: CardStatus) -> Result<(), String> {
        if !self.status.transitions().contains(&status) {
            return Err(format!(
                "Cannot transition from {} to {}",
                self.status.as_str(),
                status.as_str()
            ));
        }
        self.status = status;
        self.updated_at = chrono::Utc::now().to_rfc3339();
        Ok(())
    }

    pub fn set_priority(&mut self, priority: Priority) {
        self.priority = priority;
        self.updated_at = chrono::Utc::now().to_rfc3339();
    }

    pub fn set_description(&mut self, description: String) {
        self.description = Some(description);
        self.updated_at = chrono::Utc::now().to_rfc3339();
    }
}
