//! business — Provides workflow templates for solopreneur business workflows.
use crate::core::error::MornError;
use super::{WorkflowAction, WorkflowStep, WorkflowTemplate};
use std::collections::HashMap;

impl WorkflowTemplate {
    pub(super) fn crm_template() -> WorkflowTemplate {
        WorkflowTemplate {
            id: "workflow-crm".into(),
            name: "客户管理 (CRM)".into(),
            description: "Customer relationship management workflow with follow-up tracking and notifications".into(),
            category: "business".into(),
            tags: vec!["crm".into(), "customer".into(), "follow-up".into(), "business".into()],
            version: "1.0.0".into(),
            created_at: 0,
            updated_at: 0,
            fork_from: None,
            estimated_duration_secs: 60,
            steps: vec![
                WorkflowStep {
                    id: "query_followups".into(),
                    action: WorkflowAction::ToolCall {
                        tool_id: "web_search".into(),
                        params: serde_json::json!({"query": ""}),
                    },
                    depends_on: vec![],
                    timeout_secs: 15,
                    retry_count: 2,
                    approval_required: false,
                    input_mapping: HashMap::new(),
                    output_mapping: HashMap::new(),
                },
                WorkflowStep {
                    id: "analyze_customer".into(),
                    action: WorkflowAction::LLMCall {
                        system_prompt: "Analyze customer data and identify key insights, preferences, and follow-up needs.".into(),
                        model: "default".into(),
                    },
                    depends_on: vec!["query_followups".into()],
                    timeout_secs: 15,
                    retry_count: 1,
                    approval_required: false,
                    input_mapping: HashMap::new(),
                    output_mapping: HashMap::new(),
                },
                WorkflowStep {
                    id: "generate_reminder".into(),
                    action: WorkflowAction::LLMCall {
                        system_prompt: "Generate a personalized follow-up reminder message based on customer analysis.".into(),
                        model: "default".into(),
                    },
                    depends_on: vec!["analyze_customer".into()],
                    timeout_secs: 10,
                    retry_count: 1,
                    approval_required: true,
                    input_mapping: HashMap::new(),
                    output_mapping: HashMap::new(),
                },
                WorkflowStep {
                    id: "send_reminder".into(),
                    action: WorkflowAction::Notification {
                        channel: "email".into(),
                        message: "Customer follow-up reminder".into(),
                    },
                    depends_on: vec!["generate_reminder".into()],
                    timeout_secs: 10,
                    retry_count: 2,
                    approval_required: false,
                    input_mapping: HashMap::new(),
                    output_mapping: HashMap::new(),
                },
                WorkflowStep {
                    id: "log_activity".into(),
                    action: WorkflowAction::ToolCall {
                        tool_id: "http_request".into(),
                        params: serde_json::json!({"url": "", "method": "POST"}),
                    },
                    depends_on: vec!["send_reminder".into()],
                    timeout_secs: 10,
                    retry_count: 2,
                    approval_required: false,
                    input_mapping: HashMap::new(),
                    output_mapping: HashMap::new(),
                },
                WorkflowStep {
                    id: "notify_complete".into(),
                    action: WorkflowAction::Notification {
                        channel: "default".into(),
                        message: "CRM follow-up cycle completed".into(),
                    },
                    depends_on: vec!["log_activity".into()],
                    timeout_secs: 5,
                    retry_count: 1,
                    approval_required: false,
                    input_mapping: HashMap::new(),
                    output_mapping: HashMap::new(),
                },
            ],
        }
    }

    pub(super) fn invoice_template() -> WorkflowTemplate {
        WorkflowTemplate {
            id: "workflow-invoice".into(),
            name: "发票/收款".into(),
            description: "Invoice generation and payment collection workflow".into(),
            category: "business".into(),
            tags: vec!["invoice".into(), "payment".into(), "billing".into(), "business".into()],
            version: "1.0.0".into(),
            created_at: 0,
            updated_at: 0,
            fork_from: None,
            estimated_duration_secs: 45,
            steps: vec![
                WorkflowStep {
                    id: "select_template".into(),
                    action: WorkflowAction::LLMCall {
                        system_prompt: "Select the appropriate invoice template based on client type and project scope.".into(),
                        model: "default".into(),
                    },
                    depends_on: vec![],
                    timeout_secs: 10,
                    retry_count: 1,
                    approval_required: false,
                    input_mapping: HashMap::new(),
                    output_mapping: HashMap::new(),
                },
                WorkflowStep {
                    id: "fill_data".into(),
                    action: WorkflowAction::LLMCall {
                        system_prompt: "Fill invoice data including line items, amounts, taxes, and payment terms.".into(),
                        model: "default".into(),
                    },
                    depends_on: vec!["select_template".into()],
                    timeout_secs: 15,
                    retry_count: 1,
                    approval_required: true,
                    input_mapping: HashMap::new(),
                    output_mapping: HashMap::new(),
                },
                WorkflowStep {
                    id: "generate_pdf".into(),
                    action: WorkflowAction::ToolCall {
                        tool_id: "http_request".into(),
                        params: serde_json::json!({"url": "", "method": "POST"}),
                    },
                    depends_on: vec!["fill_data".into()],
                    timeout_secs: 10,
                    retry_count: 2,
                    approval_required: false,
                    input_mapping: HashMap::new(),
                    output_mapping: HashMap::new(),
                },
                WorkflowStep {
                    id: "send_email".into(),
                    action: WorkflowAction::Notification {
                        channel: "email".into(),
                        message: "Invoice ready for client".into(),
                    },
                    depends_on: vec!["generate_pdf".into()],
                    timeout_secs: 10,
                    retry_count: 2,
                    approval_required: false,
                    input_mapping: HashMap::new(),
                    output_mapping: HashMap::new(),
                },
                WorkflowStep {
                    id: "notify_complete".into(),
                    action: WorkflowAction::Notification {
                        channel: "default".into(),
                        message: "Invoice workflow completed".into(),
                    },
                    depends_on: vec!["send_email".into()],
                    timeout_secs: 5,
                    retry_count: 1,
                    approval_required: false,
                    input_mapping: HashMap::new(),
                    output_mapping: HashMap::new(),
                },
            ],
        }
    }

    pub(super) fn email_marketing_template() -> WorkflowTemplate {
        WorkflowTemplate {
            id: "workflow-email-marketing".into(),
            name: "邮件营销".into(),
            description: "Email marketing campaign workflow from creation to reporting".into(),
            category: "business".into(),
            tags: vec!["email".into(), "marketing".into(), "campaign".into(), "business".into()],
            version: "1.0.0".into(),
            created_at: 0,
            updated_at: 0,
            fork_from: None,
            estimated_duration_secs: 90,
            steps: vec![
                WorkflowStep {
                    id: "create_template".into(),
                    action: WorkflowAction::LLMCall {
                        system_prompt: "Create an email marketing template with compelling subject line and body content.".into(),
                        model: "default".into(),
                    },
                    depends_on: vec![],
                    timeout_secs: 20,
                    retry_count: 1,
                    approval_required: true,
                    input_mapping: HashMap::new(),
                    output_mapping: HashMap::new(),
                },
                WorkflowStep {
                    id: "select_recipients".into(),
                    action: WorkflowAction::ToolCall {
                        tool_id: "web_search".into(),
                        params: serde_json::json!({"query": ""}),
                    },
                    depends_on: vec!["create_template".into()],
                    timeout_secs: 15,
                    retry_count: 2,
                    approval_required: false,
                    input_mapping: HashMap::new(),
                    output_mapping: HashMap::new(),
                },
                WorkflowStep {
                    id: "review_content".into(),
                    action: WorkflowAction::HumanApproval {
                        message: "Please review the email campaign content before sending".into(),
                    },
                    depends_on: vec!["select_recipients".into()],
                    timeout_secs: 300,
                    retry_count: 0,
                    approval_required: true,
                    input_mapping: HashMap::new(),
                    output_mapping: HashMap::new(),
                },
                WorkflowStep {
                    id: "send_batch".into(),
                    action: WorkflowAction::Notification {
                        channel: "email".into(),
                        message: "Batch email campaign send".into(),
                    },
                    depends_on: vec!["review_content".into()],
                    timeout_secs: 30,
                    retry_count: 2,
                    approval_required: false,
                    input_mapping: HashMap::new(),
                    output_mapping: HashMap::new(),
                },
                WorkflowStep {
                    id: "track_opens".into(),
                    action: WorkflowAction::ToolCall {
                        tool_id: "web_search".into(),
                        params: serde_json::json!({"query": ""}),
                    },
                    depends_on: vec!["send_batch".into()],
                    timeout_secs: 15,
                    retry_count: 2,
                    approval_required: false,
                    input_mapping: HashMap::new(),
                    output_mapping: HashMap::new(),
                },
                WorkflowStep {
                    id: "generate_report".into(),
                    action: WorkflowAction::LLMCall {
                        system_prompt: "Generate an email marketing performance report with open rates, click rates, and recommendations.".into(),
                        model: "default".into(),
                    },
                    depends_on: vec!["track_opens".into()],
                    timeout_secs: 15,
                    retry_count: 1,
                    approval_required: false,
                    input_mapping: HashMap::new(),
                    output_mapping: HashMap::new(),
                },
            ],
        }
    }

    pub(super) fn client_portal_template() -> WorkflowTemplate {
        WorkflowTemplate {
            id: "workflow-client-portal".into(),
            name: "客户门户".into(),
            description: "Client portal workflow for file sharing, requirements gathering, and deliverable management".into(),
            category: "business".into(),
            tags: vec!["client".into(), "portal".into(), "deliverable".into(), "business".into()],
            version: "1.0.0".into(),
            created_at: 0,
            updated_at: 0,
            fork_from: None,
            estimated_duration_secs: 60,
            steps: vec![
                WorkflowStep {
                    id: "share_files".into(),
                    action: WorkflowAction::Notification {
                        channel: "email".into(),
                        message: "Share project files with client".into(),
                    },
                    depends_on: vec![],
                    timeout_secs: 10,
                    retry_count: 2,
                    approval_required: false,
                    input_mapping: HashMap::new(),
                    output_mapping: HashMap::new(),
                },
                WorkflowStep {
                    id: "collect_requirements".into(),
                    action: WorkflowAction::LLMCall {
                        system_prompt: "Analyze client requirements and feedback from shared materials.".into(),
                        model: "default".into(),
                    },
                    depends_on: vec!["share_files".into()],
                    timeout_secs: 15,
                    retry_count: 1,
                    approval_required: false,
                    input_mapping: HashMap::new(),
                    output_mapping: HashMap::new(),
                },
                WorkflowStep {
                    id: "confirm_status".into(),
                    action: WorkflowAction::HumanApproval {
                        message: "Please confirm project status and next steps with the client".into(),
                    },
                    depends_on: vec!["collect_requirements".into()],
                    timeout_secs: 300,
                    retry_count: 0,
                    approval_required: true,
                    input_mapping: HashMap::new(),
                    output_mapping: HashMap::new(),
                },
                WorkflowStep {
                    id: "package_deliverables".into(),
                    action: WorkflowAction::ToolCall {
                        tool_id: "http_request".into(),
                        params: serde_json::json!({"url": "", "method": "POST"}),
                    },
                    depends_on: vec!["confirm_status".into()],
                    timeout_secs: 15,
                    retry_count: 2,
                    approval_required: false,
                    input_mapping: HashMap::new(),
                    output_mapping: HashMap::new(),
                },
                WorkflowStep {
                    id: "notify_delivery".into(),
                    action: WorkflowAction::Notification {
                        channel: "email".into(),
                        message: "Deliverables ready for client pickup".into(),
                    },
                    depends_on: vec!["package_deliverables".into()],
                    timeout_secs: 10,
                    retry_count: 2,
                    approval_required: false,
                    input_mapping: HashMap::new(),
                    output_mapping: HashMap::new(),
                },
            ],
        }
    }

    pub(super) fn schedule_template() -> WorkflowTemplate {
        WorkflowTemplate {
            id: "workflow-schedule".into(),
            name: "日程/会议管理".into(),
            description: "Meeting scheduling and management workflow with auto-scheduling and minutes generation".into(),
            category: "business".into(),
            tags: vec!["schedule".into(), "meeting".into(), "calendar".into(), "business".into()],
            version: "1.0.0".into(),
            created_at: 0,
            updated_at: 0,
            fork_from: None,
            estimated_duration_secs: 45,
            steps: vec![
                WorkflowStep {
                    id: "view_schedule".into(),
                    action: WorkflowAction::ToolCall {
                        tool_id: "web_search".into(),
                        params: serde_json::json!({"query": ""}),
                    },
                    depends_on: vec![],
                    timeout_secs: 10,
                    retry_count: 2,
                    approval_required: false,
                    input_mapping: HashMap::new(),
                    output_mapping: HashMap::new(),
                },
                WorkflowStep {
                    id: "auto_schedule".into(),
                    action: WorkflowAction::LLMCall {
                        system_prompt: "Find optimal meeting times based on participant availability and preferences.".into(),
                        model: "default".into(),
                    },
                    depends_on: vec!["view_schedule".into()],
                    timeout_secs: 15,
                    retry_count: 1,
                    approval_required: true,
                    input_mapping: HashMap::new(),
                    output_mapping: HashMap::new(),
                },
                WorkflowStep {
                    id: "create_meeting".into(),
                    action: WorkflowAction::ToolCall {
                        tool_id: "http_request".into(),
                        params: serde_json::json!({"url": "", "method": "POST"}),
                    },
                    depends_on: vec!["auto_schedule".into()],
                    timeout_secs: 10,
                    retry_count: 2,
                    approval_required: false,
                    input_mapping: HashMap::new(),
                    output_mapping: HashMap::new(),
                },
                WorkflowStep {
                    id: "send_invites".into(),
                    action: WorkflowAction::Notification {
                        channel: "email".into(),
                        message: "Meeting invitation with agenda".into(),
                    },
                    depends_on: vec!["create_meeting".into()],
                    timeout_secs: 10,
                    retry_count: 2,
                    approval_required: false,
                    input_mapping: HashMap::new(),
                    output_mapping: HashMap::new(),
                },
                WorkflowStep {
                    id: "generate_minutes".into(),
                    action: WorkflowAction::LLMCall {
                        system_prompt: "Generate meeting minutes summarizing key discussion points, decisions, and action items.".into(),
                        model: "default".into(),
                    },
                    depends_on: vec!["send_invites".into()],
                    timeout_secs: 15,
                    retry_count: 1,
                    approval_required: false,
                    input_mapping: HashMap::new(),
                    output_mapping: HashMap::new(),
                },
            ],
        }
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
    fn crm_step_ids_in_order() {
        let t = WorkflowTemplate::crm_template();
        assert_eq!(t.steps[0].id, "query_followups");
        assert_eq!(t.steps[1].id, "analyze_customer");
        assert_eq!(t.steps[2].id, "generate_reminder");
        assert_eq!(t.steps[3].id, "send_reminder");
        assert_eq!(t.steps[4].id, "log_activity");
        assert_eq!(t.steps[5].id, "notify_complete");
    }

    #[test]
    fn crm_ends_with_notification() {
        let t = WorkflowTemplate::crm_template();
        assert!(matches!(
            t.steps[5].action,
            WorkflowAction::Notification { .. }
        ));
    }

    #[test]
    fn crm_category_and_tags() {
        let t = WorkflowTemplate::crm_template();
        assert_eq!(t.category, "business");
        assert!(t.tags.contains(&"crm".into()));
        assert!(t.tags.contains(&"business".into()));
    }

    #[test]
    fn invoice_has_five_steps() {
        let t = WorkflowTemplate::invoice_template();
        assert_eq!(t.id, "workflow-invoice");
        assert_eq!(t.name, "发票/收款");
        assert_eq!(t.steps.len(), 5);
    }

    #[test]
    fn invoice_step_ids_in_order() {
        let t = WorkflowTemplate::invoice_template();
        assert_eq!(t.steps[0].id, "select_template");
        assert_eq!(t.steps[1].id, "fill_data");
        assert_eq!(t.steps[2].id, "generate_pdf");
        assert_eq!(t.steps[3].id, "send_email");
        assert_eq!(t.steps[4].id, "notify_complete");
    }

    #[test]
    fn invoice_fill_data_requires_approval() {
        let t = WorkflowTemplate::invoice_template();
        assert!(t.steps[1].approval_required);
    }

    #[test]
    fn email_marketing_has_six_steps() {
        let t = WorkflowTemplate::email_marketing_template();
        assert_eq!(t.id, "workflow-email-marketing");
        assert_eq!(t.name, "邮件营销");
        assert_eq!(t.steps.len(), 6);
    }

    #[test]
    fn email_marketing_step_ids_in_order() {
        let t = WorkflowTemplate::email_marketing_template();
        assert_eq!(t.steps[0].id, "create_template");
        assert_eq!(t.steps[1].id, "select_recipients");
        assert_eq!(t.steps[2].id, "review_content");
        assert_eq!(t.steps[3].id, "send_batch");
        assert_eq!(t.steps[4].id, "track_opens");
        assert_eq!(t.steps[5].id, "generate_report");
    }

    #[test]
    fn email_marketing_includes_human_approval() {
        let t = WorkflowTemplate::email_marketing_template();
        assert!(matches!(
            t.steps[2].action,
            WorkflowAction::HumanApproval { .. }
        ));
        assert!(t.steps[2].approval_required);
    }

    #[test]
    fn email_marketing_ends_with_llm_call() {
        let t = WorkflowTemplate::email_marketing_template();
        assert!(matches!(t.steps[5].action, WorkflowAction::LLMCall { .. }));
    }

    #[test]
    fn client_portal_has_five_steps() {
        let t = WorkflowTemplate::client_portal_template();
        assert_eq!(t.id, "workflow-client-portal");
        assert_eq!(t.name, "客户门户");
        assert_eq!(t.steps.len(), 5);
    }

    #[test]
    fn client_portal_step_ids_in_order() {
        let t = WorkflowTemplate::client_portal_template();
        assert_eq!(t.steps[0].id, "share_files");
        assert_eq!(t.steps[1].id, "collect_requirements");
        assert_eq!(t.steps[2].id, "confirm_status");
        assert_eq!(t.steps[3].id, "package_deliverables");
        assert_eq!(t.steps[4].id, "notify_delivery");
    }

    #[test]
    fn client_portal_includes_human_approval() {
        let t = WorkflowTemplate::client_portal_template();
        assert!(matches!(
            t.steps[2].action,
            WorkflowAction::HumanApproval { .. }
        ));
    }

    #[test]
    fn client_portal_ends_with_notification() {
        let t = WorkflowTemplate::client_portal_template();
        assert!(matches!(
            t.steps[4].action,
            WorkflowAction::Notification { .. }
        ));
    }

    #[test]
    fn schedule_has_five_steps() {
        let t = WorkflowTemplate::schedule_template();
        assert_eq!(t.id, "workflow-schedule");
        assert_eq!(t.name, "日程/会议管理");
        assert_eq!(t.steps.len(), 5);
    }

    #[test]
    fn schedule_step_ids_in_order() {
        let t = WorkflowTemplate::schedule_template();
        assert_eq!(t.steps[0].id, "view_schedule");
        assert_eq!(t.steps[1].id, "auto_schedule");
        assert_eq!(t.steps[2].id, "create_meeting");
        assert_eq!(t.steps[3].id, "send_invites");
        assert_eq!(t.steps[4].id, "generate_minutes");
    }

    #[test]
    fn schedule_auto_schedule_requires_approval() {
        let t = WorkflowTemplate::schedule_template();
        assert!(t.steps[1].approval_required);
    }

    #[test]
    fn schedule_ends_with_llm_call() {
        let t = WorkflowTemplate::schedule_template();
        assert!(matches!(t.steps[4].action, WorkflowAction::LLMCall { .. }));
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
