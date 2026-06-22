import { useTranslation } from '../i18n';

const TEMPLATES = [
  {
    id: "workflow-crm",
    nameKey: "console.business.crm",
    description: "Customer relationship management workflow with follow-up tracking and notifications",
  },
  {
    id: "workflow-invoice",
    nameKey: "console.business.invoice",
    description: "Invoice generation and payment collection workflow",
  },
  {
    id: "workflow-email-marketing",
    nameKey: "console.business.email_marketing",
    description: "Email marketing campaign workflow from creation to reporting",
  },
  {
    id: "workflow-client-portal",
    nameKey: "console.business.client_portal",
    description: "Client portal workflow for file sharing, requirements gathering, and deliverable management",
  },
  {
    id: "workflow-schedule",
    nameKey: "console.business.schedule",
    description: "Meeting scheduling and management workflow with auto-scheduling and minutes generation",
  },

  // 财务管理类
  {
    id: "workflow-financial-reconciliation",
    nameKey: "console.business.financial_reconciliation",
    description: "Invoice matching, expense categorization, and monthly financial report generation",
  },

  // 项目管理类
  {
    id: "workflow-project-management",
    nameKey: "console.business.project_management",
    description: "Task breakdown, Gantt chart, milestone tracking, and progress reporting",
  },
  {
    id: "workflow-content-calendar",
    nameKey: "console.business.content_calendar",
    description: "Content scheduling, auto-publishing, and multi-platform distribution",
  },

  // 客户关系类
  {
    id: "workflow-customer-support",
    nameKey: "console.business.customer_support",
    description: "Ticket system, auto-reply, escalation strategies, and satisfaction tracking",
  },
  {
    id: "workflow-recruitment",
    nameKey: "console.business.recruitment",
    description: "Job posting, resume screening, interview arrangement, and offer management",
  },

  // 运营管理类
  {
    id: "workflow-social-media",
    nameKey: "console.business.social_media",
    description: "Multi-platform scheduling, batch publishing, and engagement analytics",
  },
  {
    id: "workflow-inventory",
    nameKey: "console.business.inventory",
    description: "Stock in/out tracking, low-stock alerts, and periodic inventory audit",
  },

  // 文档与合规类
  {
    id: "workflow-contract-management",
    nameKey: "console.business.contract_management",
    description: "Template library, approval workflow, expiration reminders, and version control",
  },
  {
    id: "workflow-market-research",
    nameKey: "console.business.market_research",
    description: "Survey design, data collection, analysis report, and insight generation",
  },
  {
    id: "workflow-knowledge-base",
    nameKey: "console.business.knowledge_base",
    description: "Document center, full-text search, version management, and team collaboration",
  },
];

export default function BusinessTemplates() {
  const { t } = useTranslation();
  const handlePreview = (id: string) => {
    alert(`Preview for template: ${id}`);
  };

  return (
    <div className="business-templates">
      <h2>{t('console.business.title')}</h2>
      <div className="template-grid">
        {TEMPLATES.map((tpl) => (
          <div key={tpl.id} className="template-card">
            <h3 className="template-name">{t(tpl.nameKey)}</h3>
            <p className="template-desc">{tpl.description}</p>
            <button className="template-btn" onClick={() => handlePreview(tpl.id)}>
              {t('console.business.preview')}
            </button>
          </div>
        ))}
      </div>
    </div>
  );
}