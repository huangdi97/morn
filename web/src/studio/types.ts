export interface AgentDef {
  name: string;
  persona: string;
  model: string;
  tools: string[];
  knowledge: string[];
  skills: string[];
}

export interface TeamTemplate {
  id: string;
  name: string;
  description: string;
  members: string[];
  mode: string;
  consensus: string;
}

export interface NodeData {
  nodeType: string;
  label: string;
  name: string;
  detail: string;
  expanded?: boolean;
  snapshot?: Record<string, unknown>;
}

export interface ComponentSummary {
  id: string;
  name: string;
  component_type: string;
  status: string;
}