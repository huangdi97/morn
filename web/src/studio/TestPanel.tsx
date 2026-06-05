import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";

interface TestStep {
  name: string;
  description: string;
  duration_ms: number;
  success: boolean;
}

interface TestResult {
  steps: TestStep[];
  total_duration_ms: number;
  total_tokens: number;
  total_cost: number;
  output: string;
}

export function TestPanel() {
  const [componentId, setComponentId] = useState("");
  const [input, setInput] = useState("");
  const [running, setRunning] = useState(false);
  const [result, setResult] = useState<TestResult | null>(null);
  const [error, setError] = useState<string | null>(null);

  const runTest = async () => {
    if (!componentId.trim() || !input.trim()) return;
    setRunning(true);
    setResult(null);
    setError(null);

    try {
      const res = await invoke<TestResult>("test_component", {
        id: componentId,
        input: input,
      });
      setResult(res);
    } catch (e: any) {
      setError(e.toString());
    } finally {
      setRunning(false);
    }
  };

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
        <textarea
          value={input}
          onChange={(e) => setInput(e.target.value)}
          placeholder="Enter test input..."
          rows={3}
        />
        <button onClick={runTest} disabled={running || !input.trim() || !componentId.trim()}>
          {running ? "Running..." : "Run Test"}
        </button>
      </div>

      {error && <div className="error-indicator">{error}</div>}

      {result && (
        <div className="test-results">
          <h3>Execution Trace</h3>
          <div className="trace-steps">
            {result.steps.map((step, i) => (
              <div key={i} className={`trace-step ${step.success ? "success" : "failure"}`}>
                <span className="step-num">[{i + 1}]</span>
                <span className="step-name">{step.name}</span>
                <span className="step-desc">{step.description}</span>
                <span className="step-time">({(step.duration_ms / 1000).toFixed(2)}s)</span>
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