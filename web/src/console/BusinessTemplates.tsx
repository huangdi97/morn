const TEMPLATES = [
  {
    id: "workflow-crm",
    name: "客户管理 (CRM)",
    description: "Customer relationship management workflow with follow-up tracking and notifications",
  },
  {
    id: "workflow-invoice",
    name: "发票/收款",
    description: "Invoice generation and payment collection workflow",
  },
  {
    id: "workflow-email-marketing",
    name: "邮件营销",
    description: "Email marketing campaign workflow from creation to reporting",
  },
  {
    id: "workflow-client-portal",
    name: "客户门户",
    description: "Client portal workflow for file sharing, requirements gathering, and deliverable management",
  },
  {
    id: "workflow-schedule",
    name: "日程/会议管理",
    description: "Meeting scheduling and management workflow with auto-scheduling and minutes generation",
  },
];

export default function BusinessTemplates() {
  const handlePreview = (id: string) => {
    alert(`Preview for template: ${id}`);
  };

  return (
    <div className="business-templates">
      <h2>Business Templates</h2>
      <div className="template-grid">
        {TEMPLATES.map((t) => (
          <div key={t.id} className="template-card">
            <h3 className="template-name">{t.name}</h3>
            <p className="template-desc">{t.description}</p>
            <button className="template-btn" onClick={() => handlePreview(t.id)}>
              Preview
            </button>
          </div>
        ))}
      </div>
    </div>
  );
}