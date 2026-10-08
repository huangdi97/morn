import { useEffect, useState } from "react";
import { Link } from "react-router-dom";
import {
  apiGet,
  apiPostJson,
  CompilerRun,
  CreatorDraftOutcome,
  OpenApiCompileOutcome,
  SolutionInstantiationOutcome,
  V115Status,
} from "../api";
import { Card, EmptyState, ErrorBox, KeyValue, StatusPill } from "../components/ui";

interface ManifestOutcome {
  manifest: Record<string, unknown> | null;
  detail?: string;
}
interface StoredSolutionPackage {
  id: string;
  name: string;
  approved_solution_id: string | null;
  version: { major: number; minor: number; patch: number };
}

export default function Studio() {
  const [goal, setGoal] = useState("Deliver a reviewed report");
  const [creatorName, setCreatorName] = useState("My Morn worker");
  const [creatorAcceptance, setCreatorAcceptance] = useState("reviewed outcome exists");
  const [creatorAutonomy, setCreatorAutonomy] = useState("governed");
  const [creatorDraft, setCreatorDraft] = useState<CreatorDraftOutcome | null>(null);
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
  const [profileRef, setProfileRef] = useState("morn.lite@1.0.0");
  const [profileOptions, setProfileOptions] = useState<string[]>([
    "morn.lite@1.0.0",
    "morn.enterprise@1.0.0",
    "morn.factory.readonly@1.0.0",
    "morn.research@1.0.0",
  ]);
  const [siteRef, setSiteRef] = useState("");
  const [instantiated, setInstantiated] = useState<SolutionInstantiationOutcome | null>(null);
  const [solutionPackages, setSolutionPackages] = useState<StoredSolutionPackage[]>([]);
  const [selectedSolutionId, setSelectedSolutionId] = useState("");
  const [solutionListError, setSolutionListError] = useState<string | null>(null);

  const refreshSolutionPackages = async (preferredId?: string) => {
    try {
      const response = await apiGet<{ solution_packages: StoredSolutionPackage[] }>("/v115/solutions");
      const approved = response.solution_packages.filter((item) => item.approved_solution_id);
      setSolutionPackages(approved);
      setSelectedSolutionId((previous) =>
        preferredId ?? (approved.some((item) => item.id === previous) ? previous : approved[0]?.id ?? ""),
      );
      setSolutionListError(null);
    } catch (e) {
      setSolutionListError((e as Error).message);
    }
  };

  useEffect(() => {
    void refreshSolutionPackages();
  }, []);

  useEffect(() => {
    apiGet<V115Status>("/v115/status")
      .then((status) => {
        const refs = status.profiles.map(
          (profile) =>
            `${profile.id}@${profile.version.major}.${profile.version.minor}.${profile.version.patch}`,
        );
        if (refs.length) {
          setProfileOptions(refs);
          if (!refs.includes(profileRef)) setProfileRef(refs[0]);
        }
      })
      .catch(() => undefined);
  }, [profileRef]);

  const runCreatorDraft = async () => {
    setError(null);
    try {
      const caps = capabilities.split(",").map((item) => item.trim()).filter(Boolean);
      const acceptance = creatorAcceptance
        .split("\n")
        .map((item) => item.trim())
        .filter(Boolean);
      const result = await apiPostJson<CreatorDraftOutcome>("/v115/creator/draft", {
        name: creatorName,
        goal,
        profile_ref: profileRef,
        site_ref: siteRef || undefined,
        required_capabilities: caps.length ? caps : ["*"],
        constraints: [],
        acceptance,
        autonomy: creatorAutonomy,
      });
      setCreatorDraft(result);
    } catch (e) {
      setError((e as Error).message);
    }
  };

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

  const instantiateSolution = async () => {
    setError(null);
    try {
      const result = await apiPostJson<SolutionInstantiationOutcome>("/v115/solution/instantiate", {
        solution_package_id: selectedSolutionId || undefined,
        goal,
        profile_ref: profileRef,
        site_ref: siteRef || undefined,
      });
      setInstantiated(result);
    } catch (e) {
      setError((e as Error).message);
    }
  };

  const approveAndCompile = async () => {
    setError(null);
    try {
      await apiPostJson("/compiler/approve", { approver: "pi" });
      const created = await apiPostJson<{ package: { id: string } }>("/compiler/compile", {});
      const m = await apiGet<ManifestOutcome>("/compiler/manifest");
      setManifest(m);
      setApproved(true);
      setInstantiated(null);
      await refreshSolutionPackages(created.package.id);
    } catch (e) {
      setError((e as Error).message);
    }
  };

  return (
    <div className="page">
      <header className="page-header">
        <h1>Studio — Solution Compiler</h1>
      </header>

      <Card title="1 · Describe goal and acceptance">
        <p>
          Start from a goal, guarantee profile and explicit acceptance. Creator only drafts an
          existing Solution pipeline; it does not create a parallel Agent/Blueprint truth model.
        </p>
        <div className="builder-item">
          <label>Name: </label>
          <input
            value={creatorName}
            onChange={(e) => setCreatorName(e.target.value)}
            style={{ width: "45%", padding: 6 }}
          />
        </div>
        <div className="builder-item">
          <label>Goal: </label>
          <input
            value={goal}
            onChange={(e) => setGoal(e.target.value)}
            style={{ width: "70%", padding: 6 }}
          />
        </div>
        <div className="builder-item">
          <label>Guarantee profile: </label>
          <select
            value={profileRef}
            onChange={(e) => setProfileRef(e.target.value)}
            style={{ padding: 6 }}
          >
            {profileOptions.map((profile) => (
              <option key={profile} value={profile}>
                {profile}
              </option>
            ))}
          </select>
        </div>
        <div className="builder-item">
          <label>Site (when applicable): </label>
          <input
            value={siteRef}
            onChange={(e) => setSiteRef(e.target.value)}
            placeholder="plant-a"
            style={{ width: "40%", padding: 6 }}
          />
        </div>
        <div className="builder-item">
          <label>Autonomy: </label>
          <select
            value={creatorAutonomy}
            onChange={(e) => setCreatorAutonomy(e.target.value)}
            style={{ padding: 6 }}
          >
            <option value="assist">Assist</option>
            <option value="governed">Governed</option>
            <option value="autonomous-within-policy">Autonomous within policy</option>
          </select>
        </div>
        <div className="builder-item">
          <label>Acceptance (one criterion per line): </label>
          <textarea
            value={creatorAcceptance}
            onChange={(e) => setCreatorAcceptance(e.target.value)}
            style={{ width: "70%", minHeight: 70, padding: 8 }}
          />
        </div>
        <div className="page-actions">
          <button onClick={runCreatorDraft}>Draft composition</button>
        </div>
        {creatorDraft && (
          <>
            <KeyValue k="Canonical write" v={creatorDraft.canonical_write ? "yes" : "no"} />
            <KeyValue k="Profile" v={creatorDraft.draft.profile_ref} />
            <KeyValue k="Work nodes" v={creatorDraft.draft.work_graph.nodes.length} />
            <KeyValue
              k="Readiness gates"
              v={creatorDraft.draft.readiness_gates.join(" → ")}
            />
            <KeyValue
              k="Unresolved"
              v={
                creatorDraft.draft.unresolved.length
                  ? creatorDraft.draft.unresolved.join(", ")
                  : "none"
              }
            />
            <KeyValue k="Canonicalization" v={creatorDraft.draft.canonicalization} />
            <KeyValue k="Next" v={creatorDraft.next.join(" → ")} />
          </>
        )}
      </Card>

      <p className="studio-workflow-hint">
        1 · Describe a goal and acceptance criteria → 2 · Validate and approve a solution →
        3 · Instantiate canonical Work. Artifact imports are optional and create candidates only.
      </p>

      <Card title="2 · Validate and compile solution">
        <p className="studio-compiler-goal">
          <strong>Desired goal:</strong> {goal.trim() || "Set a goal in step 1."}
        </p>
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
            <>
              <Card title="6. SolutionPackage Manifest">
                <pre>{JSON.stringify(manifest.manifest ?? manifest.detail, null, 2)}</pre>
              </Card>

            </>
          )}
        </>
      )}
      <Card title="3 · Instantiate canonical Work">
        <p>
          Instantiate an approved persisted SolutionPackage as canonical Work. This records
          a desired goal; no capability, authority, executor, or accepted outcome is implied.
        </p>
        {solutionListError && <p role="alert">Cannot read saved solutions: {solutionListError}</p>}
        <div className="builder-item">
          <label htmlFor="approved-solution-package">Approved solution package: </label>
          <select
            id="approved-solution-package"
            value={selectedSolutionId}
            onChange={(event) => setSelectedSolutionId(event.target.value)}
          >
            <option value="">Use current compiled package</option>
            {solutionPackages.map((item) => (
              <option key={item.id} value={item.id}>
                {item.name} @ {item.version.major}.{item.version.minor}.{item.version.patch} · {item.id}
              </option>
            ))}
          </select>
        </div>
        {solutionPackages.length === 0 && !manifest && (
          <p>No approved package yet. Compile and approve a SolutionPackage in step 2 above.</p>
        )}
        <div className="builder-item">
          <label htmlFor="work-profile">Guarantee profile: </label>
          <select id="work-profile" value={profileRef} onChange={(e) => setProfileRef(e.target.value)}>
            {profileOptions.map((profile) => (
              <option key={profile} value={profile}>{profile}</option>
            ))}
          </select>
        </div>
        <div className="builder-item">
          <label htmlFor="work-site">Site (optional): </label>
          <input id="work-site" value={siteRef} onChange={(e) => setSiteRef(e.target.value)} placeholder="plant-a" />
        </div>
        <div className="page-actions">
          <button onClick={instantiateSolution} disabled={!selectedSolutionId && !manifest}>Instantiate Work</button>
        </div>
        {instantiated && (
          <div role="status">
            <KeyValue k="Work" v={instantiated.plan.work.id} />
            <KeyValue k="Phase" v={<StatusPill value={instantiated.plan.work.status.phase} />} />
            <KeyValue k="Profile" v={instantiated.plan.work.spec.profile_ref} />
            <KeyValue k="Source solution" v={instantiated.plan.solution_package_ref} />
            <KeyValue k="Readiness gates" v={instantiated.plan.unresolved_gates.join(" → ")} />
            <KeyValue k="Execution started" v={instantiated.execution_started ? "yes" : "no — explicit gates remain"} />
            <Link className="action-link" to="/workbench">Review Work in Workbench →</Link>
          </div>
        )}
      </Card>

      <details className="studio-capability-details">
        <summary>Advanced · Import assets as candidate capabilities</summary>
        <p className="studio-capability-warning">
          OpenAPI, procedures, repositories and reviewed papers create <strong>candidate</strong>
          capabilities. Qualification, release, site admission and execution authority are separate gates.
        </p>
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

      </details>

    </div>
  );
}
