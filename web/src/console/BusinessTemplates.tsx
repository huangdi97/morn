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