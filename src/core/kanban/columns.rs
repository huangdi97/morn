//! columns — Defines kanban columns and their status ordering.
use std::cmp::Ordering;

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

    pub fn from_str_value(s: &str) -> Self {
        match s {
            "in_progress" => CardStatus::InProgress,
            "review" => CardStatus::Review,
            "done" => CardStatus::Done,
            _ => CardStatus::Todo,
        }
    }

    #[allow(clippy::should_implement_trait)] /* 预留：兼容旧调用入口 */
    pub fn from_str(s: &str) -> Self {
        Self::from_str_value(s)
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

    pub fn from_str_value(s: &str) -> Self {
        match s {
            "critical" => Priority::Critical,
            "high" => Priority::High,
            "medium" => Priority::Medium,
            _ => Priority::Low,
        }
    }

    #[allow(clippy::should_implement_trait)] /* 预留：兼容旧调用入口 */
    pub fn from_str(s: &str) -> Self {
        Self::from_str_value(s)
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
