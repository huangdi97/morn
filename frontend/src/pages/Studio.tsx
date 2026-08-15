import { useState } from "react";
import { apiGet, apiPostJson, CompilerRun } from "../api";
import { Card, EmptyState, ErrorBox, KeyValue, StatusPill } from "../components/ui";

interface ManifestOutcome {
  manifest: Record<string, unknown> | null;
  detail?: string;
}

export default function Studio() {
  const [goal, setGoal] = useState("Dataset to Reviewed Scientific Claim");
  const [capabilities, setCapabilities] = useState("*");
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [compiled, setCompiled] = useState<CompilerRun | null>(null);
  const [approved, setApproved] = useState(false);
  const [manifest, setManifest] = useState<ManifestOutcome | null>(null);

  const runCompiler = async () => {
    setError(null);
    setLoading(true);
    try {
      const caps = capabilities.split(",").map((c) => c.trim()).filter(Boolean);
      const r = await apiPostJson<CompilerRun>("/compiler/run", {
        goal,
        domain: "biolab",
        capabilities: caps.length ? caps : ["*"],
        harnesses: ["morn-native"],
      });
      setCompiled(r);
      setApproved(false);
      setManifest(null);
    } catch (e) {
      setError((e as Error).message);
    } finally {
      setLoading(false);
    }
  };

  const approveAndCompile = async () => {
    setError(null);
    try {
      await apiPostJson("/compiler/approve", { approver: "pi" });
      await apiPostJson("/compiler/compile", {});
      const m = await apiGet<ManifestOutcome>("/compiler/manifest");
      setManifest(m);
      setApproved(true);
    } catch (e) {
      setError((e as Error).message);
    }
  };

  return (
    <div className="page">
      <header className="page-header">
        <h1>Studio — Solution Compiler</h1>
      </header>

      <Card title="1. Describe Goal">
        <div className="builder-item">
          <label>Goal: </label>
          <input
            style={{ width: "70%", padding: 6 }}
            value={goal}
            onChange={(e) => setGoal(e.target.value)}
          />
        </div>
        <div className="builder-item">
          <label>Capabilities (comma separated, * = all): </label>
          <input
            style={{ width: "40%", padding: 6 }}
            value={capabilities}
            onChange={(e) => setCapabilities(e.target.value)}
          />
        </div>
        <div className="page-actions">
          <button onClick={runCompiler} disabled={loading}>
            {loading ? "Compiling…" : "Run Compiler"}
          </button>
        </div>
      </Card>

      {error && <ErrorBox message={error} />}

      {compiled && (
        <>
          <div className="grid">
            <Card title="2. ProblemSpec">
              <KeyValue k="Objective" v={compiled.problem.objective} />
              <KeyValue k="Domain" v={compiled.problem.domain} />
              <KeyValue k="Assumptions" v={compiled.problem.assumptions.length} />
            </Card>
            <Card title="3. WorkGraph">
              {compiled.work_graph.nodes.length === 0 ? (
                <EmptyState label="No nodes" />
              ) : (
                <ul>
                  {compiled.work_graph.nodes.map((n) => (
                    <li key={n.id}>
                      {n.name} — <StatusPill value={n.nature} />
                    </li>
                  ))}
                </ul>
              )}
            </Card>
            <Card title="4. WorkPackages & Capability">
              <KeyValue k="Work packages" v={compiled.proposed.work_packages.length} />
              {compiled.proposed.capability_gaps.length === 0 ? (
                <EmptyState label="No capability gaps" />
              ) : (
                <ul>
                  {compiled.proposed.capability_gaps.map((g) => (
                    <li key={g.requirement}>{g.detail}</li>
                  ))}
                </ul>
              )}
              <KeyValue k="Risk summary" v={compiled.proposed.risk_summary} />
            </Card>
            <Card title="5. Validation">
              <KeyValue k="Passed" v={compiled.validation.passed ? "yes" : "no"} />
              <ul>
                {compiled.validation.issues.map((i, idx) => (
                  <li key={idx}>
                    {i.severity}: {i.message}
                  </li>
                ))}
              </ul>
              {compiled.validation.passed && (
                <div className="page-actions" style={{ marginTop: 8 }}>
                  <button onClick={approveAndCompile} disabled={approved}>
                    {approved ? "Approved ✓" : "Approve & Compile"}
                  </button>
                </div>
              )}
            </Card>
          </div>

          {manifest && (
            <Card title="6. SolutionPackage Manifest">
              <pre>{JSON.stringify(manifest.manifest ?? manifest.detail, null, 2)}</pre>
            </Card>
          )}
        </>
      )}
    </div>
  );
}
