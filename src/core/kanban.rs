use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum CardStatus {
    Todo,
    InProgress,
    Review,
    Done,
}

impl CardStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            CardStatus::Todo => "todo",
            CardStatus::InProgress => "in_progress",
            CardStatus::Review => "review",
            CardStatus::Done => "done",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "in_progress" => CardStatus::InProgress,
            "review" => CardStatus::Review,
            "done" => CardStatus::Done,
            _ => CardStatus::Todo,
        }
    }

    pub fn transitions(&self) -> Vec<CardStatus> {
        match self {
            CardStatus::Todo => vec![CardStatus::InProgress],
            CardStatus::InProgress => vec![CardStatus::Review, CardStatus::Todo],
            CardStatus::Review => vec![CardStatus::Done, CardStatus::InProgress],
            CardStatus::Done => vec![CardStatus::Todo],
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Priority {
    Low,
    Medium,
    High,
    Critical,
}

impl Priority {
    pub fn as_str(&self) -> &'static str {
        match self {
            Priority::Low => "low",
            Priority::Medium => "medium",
            Priority::High => "high",
            Priority::Critical => "critical",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "critical" => Priority::Critical,
            "high" => Priority::High,
            "medium" => Priority::Medium,
            _ => Priority::Low,
        }
    }

    pub fn rank(&self) -> u8 {
        match self {
            Priority::Low => 0,
            Priority::Medium => 1,
            Priority::High => 2,
            Priority::Critical => 3,
        }
    }
}

impl Ord for Priority {
    fn cmp(&self, other: &Self) -> Ordering {
        self.rank().cmp(&other.rank())
    }
}

impl PartialOrd for Priority {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

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

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct KanbanBoard {
    pub name: String,
    cards: HashMap<String, KanbanCard>,
}

impl KanbanBoard {
    pub fn new(name: &str) -> Self {
        KanbanBoard {
            name: name.to_string(),
            cards: HashMap::new(),
        }
    }

    pub fn add_card(&mut self, card: KanbanCard) {
        let task_id = card.task_id.clone();
        self.cards.insert(task_id, card);
    }

    pub fn remove_card(&mut self, task_id: &str) -> Option<KanbanCard> {
        self.cards.remove(task_id)
    }

    pub fn get_card(&self, task_id: &str) -> Option<&KanbanCard> {
        self.cards.get(task_id)
    }

    pub fn get_card_mut(&mut self, task_id: &str) -> Option<&mut KanbanCard> {
        self.cards.get_mut(task_id)
    }

    pub fn move_card(&mut self, task_id: &str, status: CardStatus) -> Result<(), String> {
        let card = self
            .cards
            .get_mut(task_id)
            .ok_or_else(|| format!("Card '{}' not found", task_id))?;
        card.move_to(status)
    }

    pub fn assign_card(&mut self, task_id: &str, agent_id: String) -> Result<(), String> {
        let card = self
            .cards
            .get_mut(task_id)
            .ok_or_else(|| format!("Card '{}' not found", task_id))?;
        card.assign(agent_id);
        Ok(())
    }

    pub fn cards_by_status(&self, status: CardStatus) -> Vec<&KanbanCard> {
        self.cards.values().filter(|c| c.status == status).collect()
    }

    pub fn cards_by_agent(&self, agent_id: &str) -> Vec<&KanbanCard> {
        self.cards
            .values()
            .filter(|c| c.agent_id.as_deref() == Some(agent_id))
            .collect()
    }

    pub fn cards_sorted_by_priority(&self) -> Vec<&KanbanCard> {
        let mut cards: Vec<&KanbanCard> = self.cards.values().collect();
        cards.sort_by(|a, b| b.priority.cmp(&a.priority));
        cards
    }

    pub fn all_cards(&self) -> Vec<&KanbanCard> {
        self.cards.values().collect()
    }

    pub fn all_cards_mut(&mut self) -> Vec<&mut KanbanCard> {
        self.cards.values_mut().collect()
    }

    pub fn card_count(&self) -> usize {
        self.cards.len()
    }

    pub fn agent_ids(&self) -> HashSet<String> {
        self.cards
            .values()
            .filter_map(|c| c.agent_id.clone())
            .collect()
    }

    pub fn status_counts(&self) -> HashMap<CardStatus, usize> {
        let mut counts = HashMap::new();
        for card in self.cards.values() {
            *counts.entry(card.status.clone()).or_insert(0) += 1;
        }
        counts
    }

    pub fn clear(&mut self) {
        self.cards.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_card_status_transitions() {
        let todo = CardStatus::Todo;
        assert_eq!(todo.transitions(), vec![CardStatus::InProgress]);

        let in_progress = CardStatus::InProgress;
        assert_eq!(
            in_progress.transitions(),
            vec![CardStatus::Review, CardStatus::Todo]
        );

        let review = CardStatus::Review;
        assert_eq!(
            review.transitions(),
            vec![CardStatus::Done, CardStatus::InProgress]
        );

        let done = CardStatus::Done;
        assert_eq!(done.transitions(), vec![CardStatus::Todo]);
    }

    #[test]
    fn test_card_move_to_valid() {
        let mut card = KanbanCard::new("task-1".to_string(), "Test task".to_string());
        assert!(card.move_to(CardStatus::InProgress).is_ok());
        assert_eq!(card.status, CardStatus::InProgress);
    }

    #[test]
    fn test_card_move_to_invalid() {
        let mut card = KanbanCard::new("task-1".to_string(), "Test task".to_string());
        assert!(card.move_to(CardStatus::Done).is_err());
        assert_eq!(card.status, CardStatus::Todo);
    }

    #[test]
    fn test_priority_ordering() {
        assert!(Priority::Critical > Priority::High);
        assert!(Priority::High > Priority::Medium);
        assert!(Priority::Medium > Priority::Low);
    }

    #[test]
    fn test_board_add_and_remove() {
        let mut board = KanbanBoard::new("test");
        let card = KanbanCard::new("task-1".to_string(), "Test".to_string());
        board.add_card(card);
        assert_eq!(board.card_count(), 1);
        assert!(board.remove_card("task-1").is_some());
        assert_eq!(board.card_count(), 0);
    }

    #[test]
    fn test_board_move_card() {
        let mut board = KanbanBoard::new("test");
        board.add_card(KanbanCard::new("task-1".to_string(), "Test".to_string()));
        assert!(board.move_card("task-1", CardStatus::InProgress).is_ok());
        assert_eq!(
            board.get_card("task-1").unwrap().status,
            CardStatus::InProgress
        );
    }

    #[test]
    fn test_board_move_card_invalid() {
        let mut board = KanbanBoard::new("test");
        board.add_card(KanbanCard::new("task-1".to_string(), "Test".to_string()));
        assert!(board.move_card("task-1", CardStatus::Done).is_err());
    }

    #[test]
    fn test_cards_by_agent() {
        let mut board = KanbanBoard::new("test");
        let mut card = KanbanCard::new("task-1".to_string(), "Test".to_string());
        card.assign("agent-1".to_string());
        board.add_card(card);
        board.add_card(KanbanCard::new("task-2".to_string(), "Test 2".to_string()));
        assert_eq!(board.cards_by_agent("agent-1").len(), 1);
        assert_eq!(board.cards_by_agent("agent-2").len(), 0);
    }

    #[test]
    fn test_cards_sorted_by_priority() {
        let mut board = KanbanBoard::new("test");
        let mut high = KanbanCard::new("task-1".to_string(), "High".to_string());
        high.set_priority(Priority::High);
        let mut low = KanbanCard::new("task-2".to_string(), "Low".to_string());
        low.set_priority(Priority::Low);
        board.add_card(high);
        board.add_card(low);
        let sorted = board.cards_sorted_by_priority();
        assert_eq!(sorted[0].priority, Priority::High);
        assert_eq!(sorted[1].priority, Priority::Low);
    }

    #[test]
    fn test_status_counts() {
        let mut board = KanbanBoard::new("test");
        board.add_card(KanbanCard::new("task-1".to_string(), "T1".to_string()));
        board.add_card(KanbanCard::new("task-2".to_string(), "T2".to_string()));
        let mut card = KanbanCard::new("task-3".to_string(), "T3".to_string());
        card.move_to(CardStatus::InProgress).unwrap();
        board.add_card(card);
        let counts = board.status_counts();
        assert_eq!(counts.get(&CardStatus::Todo), Some(&2));
        assert_eq!(counts.get(&CardStatus::InProgress), Some(&1));
    }

    #[test]
    fn test_card_status_str_roundtrip() {
        for status in &[
            CardStatus::Todo,
            CardStatus::InProgress,
            CardStatus::Review,
            CardStatus::Done,
        ] {
            assert_eq!(CardStatus::from_str(status.as_str()), *status);
        }
    }

    #[test]
    fn test_priority_str_roundtrip() {
        for priority in &[
            Priority::Low,
            Priority::Medium,
            Priority::High,
            Priority::Critical,
        ] {
            assert_eq!(Priority::from_str(priority.as_str()), *priority);
        }
    }

    #[test]
    fn test_agent_ids() {
        let mut board = KanbanBoard::new("test");
        let mut c1 = KanbanCard::new("task-1".to_string(), "T1".to_string());
        c1.assign("agent-a".to_string());
        let mut c2 = KanbanCard::new("task-2".to_string(), "T2".to_string());
        c2.assign("agent-b".to_string());
        board.add_card(c1);
        board.add_card(c2);
        board.add_card(KanbanCard::new("task-3".to_string(), "T3".to_string()));
        let ids = board.agent_ids();
        assert_eq!(ids.len(), 2);
        assert!(ids.contains("agent-a"));
        assert!(ids.contains("agent-b"));
    }

    #[test]
    fn test_assign_card() {
        let mut board = KanbanBoard::new("test");
        board.add_card(KanbanCard::new("task-1".to_string(), "T1".to_string()));
        assert!(board.assign_card("task-1", "agent-x".to_string()).is_ok());
        assert_eq!(
            board.get_card("task-1").unwrap().agent_id.as_deref(),
            Some("agent-x")
        );
    }
}
