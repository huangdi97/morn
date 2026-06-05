interface QuickActionsProps {
  onSend: (text: string) => void;
}

const ACTIONS = [
  { emoji: "📊", label: "数据分析", prompt: "帮我分析最近的市场趋势数据，包括行情指标和图表" },
  { emoji: "📝", label: "写作", prompt: "帮我写一篇关于最新科技发展的短文，需要清晰的结构和流畅的表达" },
  { emoji: "🔍", label: "搜索研究", prompt: "搜索并整理关于人工智能领域的最新研究进展" },
  { emoji: "💻", label: "编码", prompt: "帮我检查这段代码的质量，找出潜在问题并给出改进建议" },
  { emoji: "🌐", label: "翻译", prompt: "请帮我进行翻译，确保专业术语准确" },
];

export function QuickActions({ onSend }: QuickActionsProps) {
  return (
    <div style={{
      display: "flex", gap: "8px", padding: "8px 16px",
      flexWrap: "wrap", justifyContent: "center",
    }}>
      {ACTIONS.map((action) => (
        <button
          key={action.label}
          onClick={() => onSend(action.prompt)}
          style={{
            display: "flex", alignItems: "center", gap: "6px",
            padding: "6px 14px", borderRadius: "20px",
            border: "1px solid var(--border)",
            background: "var(--bg-tertiary)",
            color: "var(--text-primary)",
            cursor: "pointer", fontSize: "13px",
            transition: "all 0.15s ease",
          }}
          onMouseEnter={(e) => { e.currentTarget.style.background = "var(--accent)"; e.currentTarget.style.color = "#fff"; }}
          onMouseLeave={(e) => { e.currentTarget.style.background = "var(--bg-tertiary)"; e.currentTarget.style.color = "var(--text-primary)"; }}
        >
          <span>{action.emoji}</span>
          <span>{action.label}</span>
        </button>
      ))}
    </div>
  );
}