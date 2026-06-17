//! business — Loads business workflow templates from JSON data.
use super::WorkflowTemplate;

impl WorkflowTemplate {
    pub(super) fn crm_template() -> WorkflowTemplate {
        Self::load_template("workflow-crm")
    }

    pub(super) fn invoice_template() -> WorkflowTemplate {
        Self::load_template("workflow-invoice")
    }

    pub(super) fn email_marketing_template() -> WorkflowTemplate {
        Self::load_template("workflow-email-marketing")
    }

    pub(super) fn client_portal_template() -> WorkflowTemplate {
        Self::load_template("workflow-client-portal")
    }

    pub(super) fn schedule_template() -> WorkflowTemplate {
        Self::load_template("workflow-schedule")
    }

    fn load_template(id: &str) -> WorkflowTemplate {
        let data: Vec<WorkflowTemplate> = serde_json::from_str(
            include_str!("workflow_templates.json")
        ).expect("Failed to parse workflow_templates.json");
        data.into_iter().find(|t| t.id == id)
            .unwrap_or_else(|| panic!("Template '{}' not found in workflow_templates.json", id))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crm_has_six_steps() {
        let t = WorkflowTemplate::crm_template();
        assert_eq!(t.id, "workflow-crm");
        assert_eq!(t.name, "客户管理 (CRM)");
        assert_eq!(t.steps.len(), 6);
    }

    #[test]
    fn invoice_has_five_steps() {
        let t = WorkflowTemplate::invoice_template();
        assert_eq!(t.id, "workflow-invoice");
        assert_eq!(t.name, "发票/收款");
        assert_eq!(t.steps.len(), 5);
    }

    #[test]
    fn email_marketing_has_six_steps() {
        let t = WorkflowTemplate::email_marketing_template();
        assert_eq!(t.id, "workflow-email-marketing");
        assert_eq!(t.name, "邮件营销");
        assert_eq!(t.steps.len(), 6);
    }

    #[test]
    fn client_portal_has_five_steps() {
        let t = WorkflowTemplate::client_portal_template();
        assert_eq!(t.id, "workflow-client-portal");
        assert_eq!(t.name, "客户门户");
        assert_eq!(t.steps.len(), 5);
    }

    #[test]
    fn schedule_has_five_steps() {
        let t = WorkflowTemplate::schedule_template();
        assert_eq!(t.id, "workflow-schedule");
        assert_eq!(t.name, "日程/会议管理");
        assert_eq!(t.steps.len(), 5);
    }

    #[test]
    fn all_business_templates_have_business_category() {
        for t in [
            WorkflowTemplate::crm_template(),
            WorkflowTemplate::invoice_template(),
            WorkflowTemplate::email_marketing_template(),
            WorkflowTemplate::client_portal_template(),
            WorkflowTemplate::schedule_template(),
        ] {
            assert_eq!(t.category, "business");
            assert!(!t.tags.is_empty());
            assert!(!t.version.is_empty());
        }
    }
}
