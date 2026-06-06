import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";

interface TestStep {
  name: string;
  description: string;
  duration_ms: number;
  success: boolean;
  tokens_used?: number | null;
  cost?: number | null;
  input_preview?: string | null;
  output_preview?: string | null;
}

interface TestResult {
  steps: TestStep[];
  total_duration_ms: number;
  total_tokens: number;
  total_cost: number;
  output: string;
}

interface ComponentType {
  type: string;
  label: string;
  icon: string;
}

export function TestPanel() {
  const [componentId, setComponentId] = useState("");
  const [componentType, setComponentType] = useState("agent");
  const [types, setTypes] = useState<ComponentType[]>([]);
  const [input, setInput] = useState("");
  const [running, setRunning] = useState(false);
  const [result, setResult] = useState<TestResult | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [expandedStep, setExpandedStep] = useState<number | null>(null);
  const [editingStep, setEditingStep] = useState<number | null>(null);
  const [editInput, setEditInput] = useState("");

  useState(() => {
    invoke<ComponentType[]>("list_component_types").then(setTypes).catch(() => {});
  });

  const runTest = async () => {
    if (!componentId.trim() || !input.trim()) return;
    setRunning(true);
    setResult(null);
    setError(null);
    setExpandedStep(null);
    setEditingStep(null);

    try {
      const res = await invoke<TestResult>("test_component", {
        id: componentId,
        input: input,
        componentType: componentType,
      });
      setResult(res);
    } catch (e: any) {
      setError(e.toString());
    } finally {
      setRunning(false);
    }
  };

  const toggleExpand = (idx: number) => {
    setExpandedStep(expandedStep === idx ? null : idx);
    setEditingStep(null);
  };

  const startEdit = (idx: number, currentInput: string) => {
    setEditingStep(idx);
    setEditInput(currentInput);
  };

  const cancelEdit = () => {
    setEditingStep(null);
    setEditInput("");
  };

  const runEdit = async (idx: number) => {
    try {
      const step = await invoke<TestStep>("test_component_rerun", {
        id: componentId,
        componentType: componentType,
        stepIndex: idx,
        newInput: editInput,
      });
      if (result) {
        const newSteps = [...result.steps];
        newSteps[idx] = step;
        setResult({ ...result, steps: newSteps });
      }
      setEditingStep(null);
    } catch (e: any) {
      setError(e.toString());
    }
  };

  const selectedType = types.find((t) => t.type === componentType);

  return (
    <div className="test-panel">
      <h2>Test Runner</h2>
      <div className="test-input">
        <label>Component ID:</label>
        <input
          type="text"
          value={componentId}
          onChange={(e) => setComponentId(e.target.value)}
          placeholder="comp-xxxxx"
        />
        <label>Component Type:</label>
        <select
          value={componentType}
          onChange={(e) => setComponentType(e.target.value)}
        >
          {types.length === 0 && (
            <>
              <option value="agent">Agent</option>
              <option value="tool">Tool</option>
              <option value="workflow">Workflow</option>
              <option value="knowledge">Knowledge</option>
              <option value="persona">Persona</option>
            </>
          )}
          {types.map((t) => (
            <option key={t.type} value={t.type}>
              {t.icon} {t.label}
            </option>
          ))}
        </select>
        <textarea
          value={input}
          onChange={(e) => setInput(e.target.value)}
          placeholder="Enter test input..."
          rows={3}
        />
        <button onClick={runTest} disabled={running || !input.trim() || !componentId.trim()}>
          {running ? "Running..." : `Run Test${selectedType ? ` (${selectedType.icon} ${selectedType.label})` : ""}`}
        </button>
      </div>

      {error && <div className="error-indicator">{error}</div>}

      {result && (
        <div className="test-results">
          <h3>Execution Trace</h3>
          <div className="trace-steps">
            {result.steps.map((step, i) => (
              <div key={i}>
                <div
                  className={`trace-step ${step.success ? "success" : "failure"} ${expandedStep === i ? "expanded" : ""}`}
                  onClick={() => toggleExpand(i)}
                  style={{ cursor: "pointer" }}
                >
                  <span className="step-num">[{i + 1}]</span>
                  <span className="step-name">{step.name}</span>
                  <span className="step-desc">{step.description}</span>
                  <span className="step-time">
                    ({(step.duration_ms / 1000).toFixed(2)}s)
                    {step.tokens_used != null && ` | ${step.tokens_used} tok`}
                    {step.cost != null && ` | ¥${step.cost.toFixed(4)}`}
                  </span>
                  <span className="step-expand-icon">{expandedStep === i ? "▲" : "▼"}</span>
                </div>
                {expandedStep === i && (
                  <div className="trace-step-detail">
                    <div className="trace-step-io">
                      <strong>Input:</strong>
                      {editingStep === i ? (
                        <textarea
                          className="step-edit-textarea"
                          value={editInput}
                          onChange={(e) => setEditInput(e.target.value)}
                          rows={4}
                        />
                      ) : (
                        <pre>{step.input_preview || "(no input)"}</pre>
                      )}
                    </div>
                    <div className="trace-step-io">
                      <strong>Output:</strong>
                      <pre>{step.output_preview || "(no output)"}</pre>
                    </div>
                    {step.tokens_used != null && (
                      <div className="trace-step-meta">Tokens: {step.tokens_used} | Cost: ¥{step.cost?.toFixed(4) ?? "0"}</div>
                    )}
                    <div className="trace-step-actions">
                      {editingStep === i ? (
                        <>
                          <button className="step-rerun-btn" onClick={() => runEdit(i)}>Re-run</button>
                          <button className="step-cancel-btn" onClick={cancelEdit}>Cancel</button>
                        </>
                      ) : (
                        <button className="step-rerun-btn" onClick={() => startEdit(i, step.input_preview || "")}>Edit & Re-run</button>
                      )}
                    </div>
                  </div>
                )}
              </div>
            ))}
          </div>
          <div className="trace-summary">
            <p>Total: {(result.total_duration_ms / 1000).toFixed(2)}s | Tokens: {result.total_tokens} | Cost: ¥{result.total_cost.toFixed(2)}</p>
            <p className="trace-output">Output: {result.output}</p>
          </div>
        </div>
      )}
    </div>
  );
}