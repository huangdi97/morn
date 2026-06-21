//! Proactive agent engine that manages timer-based and event-based agents.
//!
//! Agents are registered with a [`Trigger`] condition. The engine advances
//! internal counters on each [`tick`](ProactiveEngine::tick) and produces
//! ready agents whose trigger conditions have been met.

use std::collections::HashMap;
use std::sync::Arc;

use crate::core::storage::Storage;

/// Condition that determines when a [`ProactiveAgent`] fires.
#[derive(Debug, Clone)]
pub enum Trigger {
    /// Fires every `n` ticks of the engine.
    Timer(u64),
    /// Fires when the engine receives a matching event name.
    Event(String),
}

/// An agent that fires automatically when its trigger condition is met.
#[derive(Debug, Clone)]
pub struct ProactiveAgent {
    /// Unique identifier for this agent.
    pub id: String,
    /// The condition that triggers this agent.
    pub trigger: Trigger,
    /// Opaque action string (e.g. a hook name or command) to execute.
    pub action: String,
    /// Internal tick counter (managed by [`ProactiveEngine`]).
    pub counter: u64,
}

/// Engine that registers, ticks, and evaluates [`ProactiveAgent`] instances.
///
/// Agents are stored by unique `id`. Timer agents accumulate ticks until
/// reaching their interval, at which point they are yielded by [`check_ready`](ProactiveEngine::check_ready).
/// Event agents yield immediately when a matching event string is passed.
#[derive(Debug, Default)]
pub struct ProactiveEngine {
    agents: HashMap<String, ProactiveAgent>,
    storage: Option<Arc<Storage>>,
}

impl ProactiveEngine {
    pub fn new(storage: Option<Arc<Storage>>) -> Self {
        let mut engine = ProactiveEngine {
            agents: HashMap::new(),
            storage,
        };
        engine.load_rules();
        engine
    }

    /// Loads enabled rules from storage into the engine.
    pub fn load_rules(&mut self) {
        if let Some(ref storage) = self.storage {
            if let Ok(rules) = storage.list_proactive_rules() {
                for rule in rules {
                    if rule.enabled {
                        let trigger = match rule.trigger_type.as_str() {
                            "timer" => {
                                let interval = rule.trigger_config.parse::<u64>().unwrap_or(60);
                                Trigger::Timer(interval)
                            }
                            "event" => Trigger::Event(rule.trigger_config.clone()),
                            _ => continue,
                        };
                        self.agents.insert(
                            rule.id.clone(),
                            ProactiveAgent {
                                id: rule.id,
                                trigger,
                                action: rule.action,
                                counter: 0,
                            },
                        );
                    }
                }
            }
        }
    }

    /// Saves a rule to storage and registers it in the engine.
    pub fn save_rule(&mut self, rule: &crate::core::storage::ProactiveRule) {
        if let Some(ref storage) = self.storage {
            let _ = storage.create_proactive_rule(rule);
        }
        if rule.enabled {
            let trigger = match rule.trigger_type.as_str() {
                "timer" => {
                    let interval = rule.trigger_config.parse::<u64>().unwrap_or(60);
                    Trigger::Timer(interval)
                }
                "event" => Trigger::Event(rule.trigger_config.clone()),
                _ => return,
            };
            self.agents.insert(
                rule.id.clone(),
                ProactiveAgent {
                    id: rule.id.clone(),
                    trigger,
                    action: rule.action.clone(),
                    counter: 0,
                },
            );
        }
    }

    /// Deletes a rule from storage and removes it from the engine.
    pub fn delete_rule(&mut self, id: &str) {
        if let Some(ref storage) = self.storage {
            let _ = storage.delete_proactive_rule(id);
        }
        self.agents.remove(id);
    }

    /// Registers a proactive agent, replacing any existing agent with the same `id`.
    pub fn register(&mut self, agent: ProactiveAgent) {
        self.agents.insert(agent.id.clone(), agent);
    }

    /// Removes a registered agent by `id` and returns it, or `None` if not found.
    pub fn remove(&mut self, id: &str) -> Option<ProactiveAgent> {
        self.agents.remove(id)
    }

    /// Returns all agents whose trigger conditions are currently met.
    ///
    /// For timer-based triggers the internal counter is reset so the agent
    /// will not fire again until the full interval has elapsed. Event-based
    /// agents fire only when `event` matches their pattern exactly.
    ///
    /// Ready agents are removed from the engine; the caller must re-register
    /// them if they should continue to be managed.
    pub fn check_ready(&mut self, event: Option<&str>) -> Vec<ProactiveAgent> {
        let mut ready = Vec::new();
        self.agents.retain(|_id, agent| {
            let is_ready = match &agent.trigger {
                Trigger::Timer(interval) if agent.counter >= *interval => {
                    agent.counter = 0;
                    true
                }
                Trigger::Timer(_) => false,
                Trigger::Event(pattern) => {
                    if let Some(ev) = event {
                        ev == pattern
                    } else {
                        false
                    }
                }
            };
            if is_ready {
                ready.push(agent.clone());
                false // remove; caller must re-register if needed
            } else {
                true
            }
        });
        ready
    }

    /// Advance internal counters for all timer-based agents by 1.
    /// Returns agents whose timer has reached their interval.
    pub fn tick(&mut self) -> Vec<ProactiveAgent> {
        for agent in self.agents.values_mut() {
            if let Trigger::Timer(interval) = agent.trigger {
                if agent.counter < interval {
                    agent.counter += 1;
                }
            }
        }
        self.check_ready(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_and_tick_returns_ready_after_interval() {
        let mut engine = ProactiveEngine::new(None);
        let agent = ProactiveAgent {
            id: "ticker".into(),
            trigger: Trigger::Timer(3),
            action: "do_something".into(),
            counter: 0,
        };
        engine.register(agent);

        // not ready before 3 ticks
        let ready = engine.tick();
        assert!(ready.is_empty());
        let ready = engine.tick();
        assert!(ready.is_empty());

        // third tick triggers it
        let ready = engine.tick();
        assert_eq!(ready.len(), 1);
        assert_eq!(ready[0].id, "ticker");
        assert_eq!(ready[0].action, "do_something");
    }

    #[test]
    fn test_event_trigger_match() {
        let mut engine = ProactiveEngine::new(None);
        engine.register(ProactiveAgent {
            id: "event_agent".into(),
            trigger: Trigger::Event("config_changed".into()),
            action: "reload_config".into(),
            counter: 0,
        });

        // different event does not trigger
        let ready = engine.check_ready(Some("file_updated"));
        assert!(ready.is_empty());

        // matching event triggers
        let ready = engine.check_ready(Some("config_changed"));
        assert_eq!(ready.len(), 1);
        assert_eq!(ready[0].id, "event_agent");
    }

    #[test]
    fn test_remove_agent() {
        let mut engine = ProactiveEngine::new(None);
        engine.register(ProactiveAgent {
            id: "removable".into(),
            trigger: Trigger::Timer(1),
            action: "gone".into(),
            counter: 0,
        });
        let removed = engine.remove("removable");
        assert!(removed.is_some());
        assert!(engine.remove("nonexistent").is_none());
    }
}
