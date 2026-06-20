//! console — Exposes console modules for governance and cost visibility.
pub mod cost;
pub mod governance;
pub mod security;

mod backend;
mod types;

pub use backend::handle_security_command;
pub use backend::ConsoleBackend;
pub use types::*;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::dual_llm::DualLlmGuard;

    #[test]
    fn new_creates_backend_with_defaults() {
        let backend = ConsoleBackend::new(None, None, None, None, None, None);

        assert!(backend.registry.is_none());
        assert!(backend.storage.is_none());
        assert_eq!(backend.get_dashboard().total_tasks, 0);
    }

    #[test]
    fn backend_fields_are_accessible() {
        let backend = ConsoleBackend::new(None, None, None, None, None, None);

        assert!(backend.supervisor.is_none());
        assert!(backend.event_bus.is_none());
        assert!(backend.dual_llm.is_none());
        assert!(backend.marketplace.is_none());
    }

    #[test]
    fn security_command_routes_to_summary() {
        let view = security::SecurityView::new(
            DualLlmGuard::new(None, None),
            vec![security::PolicyDef {
                name: "chat".to_string(),
                level: "L4Free".to_string(),
                pattern: "chat".to_string(),
                description: "Chat".to_string(),
            }],
        );

        let output = handle_security_command("/security summary", &view);

        assert!(output.contains("policies: 1"));
    }

    #[test]
    fn security_command_routes_to_incidents_and_usage() {
        let view = security::SecurityView::new(DualLlmGuard::new(None, None), vec![]);

        assert_eq!(handle_security_command("/security incidents", &view), "[]");
        assert!(handle_security_command("/security unknown", &view).contains("Usage"));
    }
}
