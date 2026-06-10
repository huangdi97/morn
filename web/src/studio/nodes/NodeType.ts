export enum NodeType {
  HttpRequest = "HttpRequest",
  LLMCall = "LLMCall",
  Condition = "Condition",
  Loop = "Loop",
  Transform = "Transform",
  Merge = "Merge",
  Split = "Split",
  Code = "Code",
  Trigger = "Trigger",
  Wait = "Wait",
  Switch = "Switch",
}

export interface NodeDefinition {
  id: string;
  nodeType: NodeType;
  label: string;
  config: Record<string, unknown>;
  inputs: string[];
  outputs: string[];
}

export interface NodeEdge {
  id: string;
  source: string;
  sourceOutput: string;
  target: string;
  targetInput: string;
}

export interface NodeGraph {
  nodes: NodeDefinition[];
  edges: NodeEdge[];
}

export interface NodeTemplate {
  nodeType: NodeType;
  label: string;
  description: string;
  category: string;
  defaultConfig: Record<string, unknown>;
  inputs: string[];
  outputs: string[];
}

export interface ExecutionResult {
  nodeId: string;
  output: unknown;
  success: boolean;
  error: string | null;
}

export const NODE_COLORS: Record<NodeType, string> = {
  [NodeType.HttpRequest]: "#3b82f6",
  [NodeType.LLMCall]: "#8b5cf6",
  [NodeType.Condition]: "#f59e0b",
  [NodeType.Loop]: "#06b6d4",
  [NodeType.Transform]: "#22c55e",
  [NodeType.Merge]: "#ec4899",
  [NodeType.Split]: "#ef4444",
  [NodeType.Code]: "#f97316",
  [NodeType.Trigger]: "#14b8a6",
  [NodeType.Wait]: "#eab308",
  [NodeType.Switch]: "#a855f7",
};

export const NODE_CATEGORIES = [
  { name: "AI", color: "#8b5cf6" },
  { name: "Network", color: "#3b82f6" },
  { name: "Flow", color: "#06b6d4" },
  { name: "Data", color: "#22c55e" },
  { name: "Control", color: "#f97316" },
  { name: "Trigger", color: "#14b8a6" },
];

export const NODE_CATEGORY_MAP: Record<NodeType, string> = {
  [NodeType.HttpRequest]: "Network",
  [NodeType.LLMCall]: "AI",
  [NodeType.Condition]: "Flow",
  [NodeType.Loop]: "Flow",
  [NodeType.Transform]: "Data",
  [NodeType.Merge]: "Data",
  [NodeType.Split]: "Data",
  [NodeType.Code]: "Control",
  [NodeType.Trigger]: "Trigger",
  [NodeType.Wait]: "Flow",
  [NodeType.Switch]: "Flow",
};