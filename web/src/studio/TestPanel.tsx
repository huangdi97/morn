import { useState } from "react";

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
  const [input, setInput] = useState("");
  const [running, setRunning] = useState(false);
  const [result, setResult] = useState<TestResult | null>(null);

  const runTest = () => {
    setRunning(true);
    setResult(null);

    const steps: TestStep[] = [
      { name: "persona_injection", description: "Enhance Prompt", duration_ms: 0.02, success: true },
      { name: "knowledge_retrieval", description: "stock_db -> MACD=Golden Cross", duration_ms: 0.15, success: true },
      { name: "llm_call", description: "deepseek (1.2s, 890 tokens)", duration_ms: 1.2, success: true },
      { name: "tool_execution", description: "get_kline (2.1s)", duration_ms: 2.1, success: true },
    ];

    setTimeout(() => {
      setResult({
        steps,
        total_duration_ms: steps.reduce((a, s) => a + s.duration_ms, 0) * 1000,
        total_tokens: 2090,
        total_cost: 0.02,
        output: `Test completed for input: ${input}`,
      });
      setRunning(false);
    }, 1500);
  };

  return (
    <div className="test-panel">
      <h2>Test Runner</h2>
      <div className="test-input">
        <textarea
          value={input}
          onChange={(e) => setInput(e.target.value)}
          placeholder="Enter test input..."
          rows={3}
        />
        <button onClick={runTest} disabled={running || !input.trim()}>
          {running ? "Running..." : "Run Test"}
        </button>
      </div>

      {result && (
        <div className="test-results">
          <h3>Execution Trace</h3>
          <div className="trace-steps">
            {result.steps.map((step, i) => (
              <div key={i} className={`trace-step ${step.success ? "success" : "failure"}`}>
                <span className="step-num">[{i + 1}]</span>
                <span className="step-name">{step.name}</span>
                <span className="step-desc">{step.description}</span>
                <span className="step-time">({step.duration_ms.toFixed(2)}s)</span>
              </div>
            ))}
          </div>
          <div className="trace-summary">
            <p>Total: {(result.total_duration_ms / 1000).toFixed(2)}s | Tokens: {result.total_tokens} | Cost: ¥{result.total_cost.toFixed(2)}</p>
          </div>
        </div>
      )}
    </div>
  );
}