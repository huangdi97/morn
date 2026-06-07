use super::WorkflowTemplateEntry;

pub(super) fn builtin_templates() -> Vec<WorkflowTemplateEntry> {
    vec![
        WorkflowTemplateEntry {
            workflow_id: "data-analysis".into(),
            name: "数据分析".into(),
            description: "数据导入、清洗、分析与可视化的一站式工作流".into(),
            category: "data".into(),
            nodes: serde_json::json!([
                {"id": "load_data", "type": "tool", "tool": "data_loader", "params": {"source": ""}},
                {"id": "clean_data", "type": "tool", "tool": "data_cleaner", "params": {}, "depends_on": ["load_data"]},
                {"id": "analyze", "type": "llm", "prompt": "对清洗后的数据进行统计分析", "model": "default", "depends_on": ["clean_data"]},
                {"id": "visualize", "type": "tool", "tool": "chart", "params": {"type": "auto"}, "depends_on": ["analyze"]},
                {"id": "report", "type": "llm", "prompt": "生成数据分析报告", "model": "default", "depends_on": ["visualize"]}
            ]),
            tags: vec!["data".into(), "analysis".into(), "visualization".into()],
        },
        WorkflowTemplateEntry {
            workflow_id: "code-review".into(),
            name: "代码审查".into(),
            description: "自动代码审查工作流，包括静态分析、安全检查与优化建议".into(),
            category: "development".into(),
            nodes: serde_json::json!([
                {"id": "fetch_code", "type": "tool", "tool": "git_clone", "params": {"repo": ""}},
                {"id": "static_analysis", "type": "tool", "tool": "linter", "params": {"rules": "default"}, "depends_on": ["fetch_code"]},
                {"id": "security_scan", "type": "tool", "tool": "security_checker", "params": {}, "depends_on": ["fetch_code"]},
                {"id": "llm_review", "type": "llm", "prompt": "审查代码质量、可维护性与最佳实践", "model": "default", "depends_on": ["static_analysis", "security_scan"]},
                {"id": "summary", "type": "llm", "prompt": "生成代码审查摘要与修改建议", "model": "default", "depends_on": ["llm_review"]}
            ]),
            tags: vec!["code".into(), "review".into(), "security".into()],
        },
        WorkflowTemplateEntry {
            workflow_id: "report-generation".into(),
            name: "报告生成".into(),
            description: "自动收集数据并生成格式化报告".into(),
            category: "reporting".into(),
            nodes: serde_json::json!([
                {"id": "collect", "type": "tool", "tool": "web_search", "params": {"query": ""}},
                {"id": "research", "type": "agent", "agent_id": "researcher", "input": "", "depends_on": ["collect"]},
                {"id": "draft", "type": "llm", "prompt": "撰写包含摘要、发现、分析与建议的完整报告", "model": "default", "depends_on": ["research"]},
                {"id": "format", "type": "tool", "tool": "formatter", "params": {"style": "markdown"}, "depends_on": ["draft"]},
                {"id": "deliver", "type": "notification", "channel": "email", "message": "报告已生成", "depends_on": ["format"]}
            ]),
            tags: vec!["report".into(), "generate".into(), "document".into()],
        },
        WorkflowTemplateEntry {
            workflow_id: "web-scraping".into(),
            name: "网页抓取".into(),
            description: "网页内容抓取、解析与结构化存储工作流".into(),
            category: "data".into(),
            nodes: serde_json::json!([
                {"id": "fetch_page", "type": "tool", "tool": "http_request", "params": {"url": "", "method": "GET"}},
                {"id": "parse_html", "type": "tool", "tool": "html_parser", "params": {"selector": ""}, "depends_on": ["fetch_page"]},
                {"id": "extract_data", "type": "llm", "prompt": "从解析后的HTML中提取结构化数据", "model": "default", "depends_on": ["parse_html"]},
                {"id": "transform", "type": "tool", "tool": "data_transformer", "params": {"format": "json"}, "depends_on": ["extract_data"]},
                {"id": "save", "type": "tool", "tool": "file_writer", "params": {"path": ""}, "depends_on": ["transform"]}
            ]),
            tags: vec!["web".into(), "scraping".into(), "crawl".into()],
        },
        WorkflowTemplateEntry {
            workflow_id: "scheduled-monitoring".into(),
            name: "定时监控".into(),
            description: "定时检查系统健康状态并在异常时发送告警".into(),
            category: "operations".into(),
            nodes: serde_json::json!([
                {"id": "health_check", "type": "tool", "tool": "http_request", "params": {"url": "", "method": "GET", "timeout": 10}},
                {"id": "check_metrics", "type": "tool", "tool": "metric_collector", "params": {}, "depends_on": ["health_check"]},
                {"id": "evaluate", "type": "llm", "prompt": "根据收集到的指标评估系统健康状态", "model": "default", "depends_on": ["check_metrics"]},
                {"id": "conditional_alert", "type": "condition", "expression": "status != healthy", "true_branch": [{"id": "alert", "type": "notification", "channel": "default", "message": "系统异常：请立即检查"}], "false_branch": [], "depends_on": ["evaluate"]},
                {"id": "log_result", "type": "tool", "tool": "logger", "params": {"level": "info"}, "depends_on": ["conditional_alert"]}
            ]),
            tags: vec!["monitor".into(), "scheduled".into(), "ops".into()],
        },
    ]
}
