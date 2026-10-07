import { useState } from "react";
import { apiGet, apiPostJson, CompilerRun, OpenApiCompileOutcome } from "../api";
import { Card, EmptyState, ErrorBox, KeyValue, StatusPill } from "../components/ui";

interface ManifestOutcome {
  manifest: Record<string, unknown> | null;
  detail?: string;
}

export default function Studio() {
  const [goal, setGoal] = useState("Deliver a reviewed report");
  const [capabilities, setCapabilities] = useState("*");
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [compiled, setCompiled] = useState<CompilerRun | null>(null);
  const [approved, setApproved] = useState(false);
  const [manifest, setManifest] = useState<ManifestOutcome | null>(null);
  const [openApiText, setOpenApiText] = useState(
    '{"openapi":"3.1.0","paths":{"/orders":{"get":{},"post":{}}}}',
  );
  const [artifactCandidate, setArtifactCandidate] = useState<OpenApiCompileOutcome | null>(null);
  const [procedureText, setProcedureText] = useState(
    '{"provides":["factory.outage.review"],"steps":[{"id":"inspect","capability":"historian.read","effect":"E0"},{"id":"review","capability":"human.approve","effect":"E0"}]}',
  );
  const [procedureCandidate, setProcedureCandidate] = useState<OpenApiCompileOutcome | null>(null);
  const [repositoryText, setRepositoryText] = useState(
    '{"kind":"solver","provides":["capacity.optimize"],"entrypoints":["bin/solve"],"maximum_effect":"E0"}',
  );
  const [repositoryCandidate, setRepositoryCandidate] = useState<OpenApiCompileOutcome | null>(null);
  const [paperText, setPaperText] = useState(
    '{"reviewed":true,"executable":true,"provides":["method.execute"],"code_bindings":["repo://paper-code#run"],"evidence_refs":["doi://example#method"]}',
  );
  const [paperCandidate, setPaperCandidate] = useState<OpenApiCompileOutcome | null>(null);

  const runCompiler = async () => {
    setError(null);
    setLoading(true);
    try {
      const caps = capabilities.split(",").map((c) => c.trim()).filter(Boolean);
      const r = await apiPostJson<CompilerRun>("/compiler/run", {
        goal,
        domain: "generic",
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

  const compileOpenApi = async () => {
    setError(null);
    try {
      const result = await apiPostJson<OpenApiCompileOutcome>("/v115/artifact/openapi/compile", {
        name: "studio-openapi-capability",
        source_ref: "studio://inline-openapi",
        content: openApiText,
      });
      setArtifactCandidate(result);
    } catch (e) {
      setError((e as Error).message);
    }
  };

  const compileProcedure = async () => {
    setError(null);
    try {
      const result = await apiPostJson<OpenApiCompileOutcome>("/v115/artifact/procedure/compile", {
        name: "studio-procedure-capability",
        source_ref: "studio://inline-procedure",
        content: procedureText,
      });
      setProcedureCandidate(result);
    } catch (e) {
      setError((e as Error).message);
    }
  };

  const compileRepository = async () => {
    setError(null);
    try {
      const result = await apiPostJson<OpenApiCompileOutcome>("/v115/artifact/repository/compile", {
        name: "studio-repository-capability",
        source_ref: "repo://studio/declared-manifest",
        content: repositoryText,
      });
      setRepositoryCandidate(result);
    } catch (e) {
      setError((e as Error).message);
    }
  };

  const compilePaper = async () => {
    setError(null);
    try {
      const result = await apiPostJson<OpenApiCompileOutcome>("/v115/artifact/paper/compile", {
        name: "studio-paper-derived-capability",
        source_ref: "paper://reviewed-manifest",
        content: paperText,
      });
      setPaperCandidate(result);
    } catch (e) {
      setError((e as Error).message);
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

      <Card title="Artifact → Capability Candidate">
        <p>
          Compile an existing asset into a candidate capability. Compilation never means qualified
          or site-admitted.
        </p>
        <textarea
          style={{ width: "100%", minHeight: 110, padding: 8 }}
          value={openApiText}
          onChange={(e) => setOpenApiText(e.target.value)}
        />
        <div className="page-actions" style={{ marginTop: 8 }}>
          <button onClick={compileOpenApi}>Compile OpenAPI</button>
        </div>
        {artifactCandidate && (
          <>
            <KeyValue k="Stage" v={artifactCandidate.candidate.record.stage} />
            <KeyValue
              k="Operations"
              v={artifactCandidate.candidate.report.discovered_operations.join(", ")}
            />
            <KeyValue k="Admission" v={artifactCandidate.admission} />
            <KeyValue k="Next gates" v={artifactCandidate.next.join(" → ")} />
          </>
        )}
        <hr style={{ margin: "20px 0" }} />
        <p>
          SOP / Procedure uses the same supply-chain rule: generation creates a candidate only.
        </p>
        <textarea
          style={{ width: "100%", minHeight: 110, padding: 8 }}
          value={procedureText}
          onChange={(e) => setProcedureText(e.target.value)}
        />
        <div className="page-actions" style={{ marginTop: 8 }}>
          <button onClick={compileProcedure}>Compile SOP / Procedure</button>
        </div>
        {procedureCandidate && (
          <>
            <KeyValue k="Procedure stage" v={procedureCandidate.candidate.record.stage} />
            <KeyValue
              k="Discovered steps"
              v={procedureCandidate.candidate.report.discovered_operations.join(", ")}
            />
            <KeyValue k="Procedure admission" v={procedureCandidate.admission} />
            <KeyValue k="Procedure next gates" v={procedureCandidate.next.join(" → ")} />
          </>
        )}

        <hr style={{ margin: "20px 0" }} />
        <p>
          Repository conversion requires an explicit capability manifest. Morn does not scan a
          repository and invent hidden entrypoints.
        </p>
        <textarea
          style={{ width: "100%", minHeight: 110, padding: 8 }}
          value={repositoryText}
          onChange={(e) => setRepositoryText(e.target.value)}
        />
        <div className="page-actions" style={{ marginTop: 8 }}>
          <button onClick={compileRepository}>Compile Repository Manifest</button>
        </div>
        {repositoryCandidate && (
          <>
            <KeyValue k="Repository stage" v={repositoryCandidate.candidate.record.stage} />
            <KeyValue
              k="Entrypoints"
              v={repositoryCandidate.candidate.report.discovered_operations.join(", ")}
            />
            <KeyValue k="Repository admission" v={repositoryCandidate.admission} />
          </>
        )}

        <hr style={{ margin: "20px 0" }} />
        <p>
          Paper-derived capabilities require reviewed extraction. Executable candidates must bind
          to real code; free-form paper text is never promoted directly.
        </p>
        <textarea
          style={{ width: "100%", minHeight: 110, padding: 8 }}
          value={paperText}
          onChange={(e) => setPaperText(e.target.value)}
        />
        <div className="page-actions" style={{ marginTop: 8 }}>
          <button onClick={compilePaper}>Compile Reviewed Paper Manifest</button>
        </div>
        {paperCandidate && (
          <>
            <KeyValue k="Paper stage" v={paperCandidate.candidate.record.stage} />
            <KeyValue
              k="Code / evidence bindings"
              v={paperCandidate.candidate.report.discovered_operations.join(", ")}
            />
            <KeyValue k="Paper admission" v={paperCandidate.admission} />
            <KeyValue k="Paper next gates" v={paperCandidate.next.join(" → ")} />
          </>
        )}
      </Card>

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
