import { useCallback, useEffect, useState } from "react";
import { Link } from "react-router-dom";
import {
  apiGet,
  apiPost,
  apiPostJson,
  CertifyOutcome,
  DistillOutcome,
  DurableRun,
  EvaluationOutcome,
  FlywheelOutcome,
  LoopAOutcome,
  ManagedOutcome,
  ReplacementOutcome,
  ReplayOutcome,
  ShadowOutcome,
  WorkbenchData,
  OpintPredictOutcome,
  AcceptanceReviewerCatalog,
  SourceOfTruthCatalog,
  V115Status,
  V115ControlPlaneData,
  UiExtensionRegistry,
} from "../api";
import { Card, EmptyState, KeyValue, Loading, StatusPill } from "../components/ui";

/** Domain-gated UI extension point: BioLab reference UI is only rendered when
 *  the backend advertises the biolab-reference domain pack (zero-domain builds
 *  advertise none, so the Core surfaces stay domain-free). */
export function biolabEnabled(domainPacks: string[]): boolean {
  return domainPacks.includes("biolab-reference");
}


function textField(record: Record<string, unknown>, field: string): string | null {
  return typeof record[field] === "string" ? (record[field] as string) : null;
}

function fieldRefs(record: Record<string, unknown>, field: string): string[] {
  const refs = record[field];
  return Array.isArray(refs) ? refs.filter((value): value is string => typeof value === "string") : [];
}

/** Correlate only persisted records explicitly linked to the same Work.
 *  A provider result, unlinked receipt or fixture must never count as accepted outcome. */
export function workEvidenceTrace(control: V115ControlPlaneData, workId: string, generation: number) {
  const resolutions = control.capability_resolutions.filter(
    (row) => textField(row, "work_ref") === workId && row.work_generation === generation,
  );
  const bindings = control.execution_bindings.filter((row) => textField(row, "work_id") === workId);
  const bindingIds = new Set(bindings.map((row) => textField(row, "id")).filter((id): id is string => !!id));
  const receipts = control.execution_receipts.filter((row) => {
    const id = textField(row, "execution_binding_ref");
    return id !== null && bindingIds.has(id) && row.work_generation === generation;
  });
  const attempts = control.attempts.filter((row) => {
    const id = textField(row, "binding_id");
    return id !== null && bindingIds.has(id);
  });
  const attemptIds = new Set(attempts.map((row) => textField(row, "id")).filter((id): id is string => !!id));
  const reconciliations = control.reconciliations.filter((row) => {
    const id = textField(row, "attempt_id");
    return id !== null && attemptIds.has(id);
  });
  const outcomes = control.outcomes.filter((row) => textField(row, "work_package_id") === workId);
  const outcomeIds = new Set(outcomes.map((row) => textField(row, "id")).filter((id): id is string => !!id));
  const acceptances = control.acceptance_decisions.filter(
    (row) => textField(row, "work_package_id") === workId &&
      fieldRefs(row, "outcome_refs").some((id) => outcomeIds.has(id)),
  );
  const evidence = control.condition_evidence.filter((row) =>
    textField(row, "work_ref") === workId && row.work_generation === generation,
  );
  return { resolutions, bindings, receipts, attempts, reconciliations, outcomes, acceptances, evidence };
}

function WorkEvidenceTrace({
  control,
  workId,
  generation,
}: {
  control: V115ControlPlaneData;
  workId: string;
  generation: number;
}) {
  const trace = workEvidenceTrace(control, workId, generation);
  const summary = `${trace.resolutions.length} resolution decisions · ${trace.bindings.length} bindings · ${trace.receipts.length} harness receipts · ${trace.outcomes.length} outcomes · ${trace.acceptances.length} linked decisions`;
  return (
    <details className="work-truth-trace">
      <summary>Execution, reality &amp; independent acceptance — {summary}</summary>
      <div className="work-truth-trace-grid">
        <section>
          <strong>Capability resolution</strong>
          {trace.resolutions.length === 0 ? (
            <p>No durable Workcell resolution decision for this generation.</p>
          ) : (
            <ul>
              {trace.resolutions.map((decision, index) => (
                <li key={textField(decision, "id") ?? index}>
                  <b>{textField(decision, "id") ?? "Resolution decision"}</b>
                  <small>Generation: {String(decision.work_generation ?? "unknown")}</small>
                  <small>Source solution: {textField(decision, "source_solution_ref") ?? "Not linked"}</small>
                </li>
              ))}
            </ul>
          )}
        </section>
        <section>
          <strong>Harness execution evidence</strong>
          {trace.receipts.length === 0 ? (
            <p>No pinned harness execution receipt; provider completion is not implied.</p>
          ) : (
            <ul>
              {trace.receipts.map((receipt, index) => (
                <li key={textField(receipt, "id") ?? index}>
                  <b>{textField(receipt, "provider_ref") ?? "Unknown provider"}</b> — {textField(receipt, "outcome") ?? "Unsettled"}
                  <small>Session: {textField(receipt, "session_id") ?? "Unknown"}</small>
                  <small>Environment: {textField(receipt, "execution_environment_ref") ?? "Fixture / not pinned"}</small>
                </li>
              ))}
            </ul>
          )}
          <p>Harness receipts are executor evidence only; they never establish a business outcome or acceptance.</p>
        </section>
        <section>
          <strong>External action &amp; side-effect truth</strong>
          {trace.attempts.length === 0 ? (
            <p>No recorded external action attempts; no side effect may be inferred.</p>
          ) : (
            <ul>
              {trace.attempts.map((attempt, index) => (
                <li key={textField(attempt, "id") ?? index}>
                  <b>{textField(attempt, "action") ?? "Unknown action"}</b> — {textField(attempt, "state") ?? "Unclassified"}
                  <small>Business key: {textField(attempt, "business_key") ?? "Missing"}</small>
                  <small>External reference: {textField(attempt, "external_ref") ?? "Not observed"}</small>
                </li>
              ))}
            </ul>
          )}
          <p>Reconciliations: {trace.reconciliations.length}. Unknown external effects are not automatically retried.</p>
        </section>
        <section>
          <strong>Source-grounded outcomes</strong>
          {trace.outcomes.length === 0 ? (
            <p>No authoritative outcome observation for this Work.</p>
          ) : (
            <ul>
              {trace.outcomes.map((outcome, index) => (
                <li key={textField(outcome, "id") ?? index}>
                  {textField(outcome, "objective") ?? "Observed outcome"}
                  <small>Source: {textField(outcome, "source_ref") ?? "Not bound"}</small>
                  <small>Witness references: {fieldRefs(outcome, "evidence_refs").length}</small>
                </li>
              ))}
            </ul>
          )}
        </section>
        <section>
          <strong>Independent acceptance</strong>
          {trace.acceptances.length === 0 ? (
            <p>No linked independent acceptance decision; executor completion does not establish acceptance.</p>
          ) : (
            <ul>
              {trace.acceptances.map((decision, index) => (
                <li key={textField(decision, "id") ?? index}>
                  <b>{textField(decision, "disposition") ?? "Undecided"}</b>
                  <small>Role: {textField(decision, "acting_role") ?? "Unspecified"}</small>
                  <small>Reason: {textField(decision, "reason") ?? "No reason provided"}</small>
                </li>
              ))}
            </ul>
          )}
        </section>
        <section>
          <strong>Generation-scoped readiness witnesses</strong>
          {trace.evidence.length === 0 ? (
            <p>No generation-scoped witnesses stored for this Work.</p>
          ) : (
            <ul>
              {trace.evidence.map((entry, index) => (
                <li key={textField(entry, "id") ?? index}>
                  {textField(entry, "condition_type") ?? "Condition"} — {entry.satisfied === true ? "witnessed" : "not satisfied"}
                  <small>Producer: {textField(entry, "producer_ref") ?? "Unknown"}</small>
                </li>
              ))}
            </ul>
          )}
        </section>
      </div>
    </details>
  );
}

/** The canonical Work surface is independent of legacy/demo diagnostic health. */
export function CanonicalWorkOverview({
  control,
  error,
}: {
  control: V115ControlPlaneData | null;
  error: string | null;
}) {
  return (
      <section className="work-focus" aria-label="Canonical Work overview">
        <div className="work-focus-intro">
          <div>
            <span className="work-focus-eyebrow">MORN v11.5 · CANONICAL WORK</span>
            <h2>Work is the unit of coordination</h2>
            <p>
              Follow the goal, actual observations, required conditions, pinned execution and
              independent acceptance. Provider sessions and demo runs are not Work truth.
            </p>
          </div>
          <Link className="action-link" to="/studio">Draft Work in Studio →</Link>
        </div>
        {error ? (
          <p role="alert" className="work-focus-alert">
            Canonical Work data is unavailable: {error}. Legacy diagnostics cannot
            establish canonical Work truth.
          </p>
        ) : !control ? (
          <p className="work-focus-empty" role="status">Loading persisted Work state…</p>
        ) : control.work.length === 0 ? (
          <div className="work-focus-empty">
            <strong>No canonical Work has been persisted yet.</strong>
            <p>
              Start with a goal and acceptance criteria in Studio. Compilation does not grant
              execution authority, and a completed harness run does not establish accepted outcome.
            </p>
          </div>
        ) : (
          <div className="work-focus-grid">
            {control.work.map((work) => (
              <article className="work-focus-item" key={work.id}>
                <div className="work-focus-item-header">
                  <h3>{work.spec.goal}</h3>
                  <StatusPill value={work.status.phase} />
                </div>
                <dl>
                  <div><dt>Observed generation</dt><dd>{work.status.observed_generation}/{work.generation}</dd></div>
                  <div><dt>Profile</dt><dd>{work.spec.profile_ref}</dd></div>
                  <div><dt>Execution binding</dt><dd>{work.status.active_binding ?? "Not bound"}</dd></div>
                </dl>
                <div className="work-focus-conditions">
                  <strong>Readiness &amp; evidence</strong>
                  {work.status.conditions.length === 0 ? (
                    <p>Conditions not yet observed — do not infer readiness.</p>
                  ) : (
                    <ul>
                      {work.status.conditions.map((condition, index) => (
                        <li key={`${condition.condition_type}-${index}`}>
                          <span>{condition.condition_type}</span>
                          <StatusPill value={condition.status} />
                          <small>{condition.reason}</small>
                        </li>
                      ))}
                    </ul>
                  )}
                </div>
                <WorkEvidenceTrace control={control} workId={work.id} generation={work.generation} />
              </article>
            ))}
          </div>
        )}
      </section>
  );
}

type E0HarnessCapability = {
  manifest: {
    id: string;
    name: string;
    provider_ref: string;
    authority: { maximum_effect: string };
  };
  stage: string;
};

function GovernedE0Executor({
  control,
  capabilities,
  status,
  reload,
}: {
  control: V115ControlPlaneData;
  capabilities: E0HarnessCapability[];
  status: V115Status | null;
  reload: () => void;
}) {
  const [workId, setWorkId] = useState("");
  const [capabilityId, setCapabilityId] = useState("");
  const [bindingId, setBindingId] = useState("");
  const [environmentRef, setEnvironmentRef] = useState("");
  const [prompt, setPrompt] = useState("Execute the bound E0 capability and return executor evidence only.");
  const [busy, setBusy] = useState(false);
  const [message, setMessage] = useState<string | null>(null);

  const candidateWorks = control.work.filter(
    (work) => !["Accepted", "Rejected", "Cancelled"].includes(work.status.phase),
  );
  const effectiveWorkId = workId || candidateWorks[0]?.id || "";
  const selectedWork = candidateWorks.find((work) => work.id === effectiveWorkId);
  const resolvedCapabilityRefs = new Set(
    control.condition_evidence
      .filter(
        (evidence) =>
          textField(evidence, "work_ref") === effectiveWorkId &&
          evidence.work_generation === selectedWork?.generation &&
          textField(evidence, "condition_type") === "CapabilityResolved" &&
          evidence.satisfied === true,
      )
      .flatMap((evidence) => fieldRefs(evidence, "evidence_refs")),
  );
  const eligibleCapabilities = capabilities.filter(
    (capability) =>
      ["Qualified", "Admitted"].includes(capability.stage) &&
      ["morn-native", "deepseek-harness", "pi"].includes(capability.manifest.provider_ref) &&
      capability.manifest.authority.maximum_effect === "E0LifecycleReversible" &&
      resolvedCapabilityRefs.has(capability.manifest.id),
  );
  const workBindings = control.execution_bindings.filter(
    (binding) =>
      textField(binding, "work_id") === effectiveWorkId &&
      binding.work_generation ===
        candidateWorks.find((work) => work.id === effectiveWorkId)?.generation,
  );
  const effectiveCapabilityId = capabilityId || eligibleCapabilities[0]?.manifest.id || "";
  const selectedCapability = eligibleCapabilities.find(
    (capability) => capability.manifest.id === effectiveCapabilityId,
  );
  const realDsh =
    selectedCapability?.manifest.provider_ref === "deepseek-harness" &&
    status?.harness_runtime.dsh.mode === "real";
  const realPi =
    selectedCapability?.manifest.provider_ref === "pi" &&
    status?.harness_runtime.pi.mode === "real";
  const configuredDshEnvironment =
    status?.harness_runtime.dsh.configured_execution_environment_ref ?? "";
  const eligibleEnvironments = (status?.execution_environment_attestations ?? []).filter(
    (environment) =>
      environment.active &&
      (!realDsh || environment.environment_ref === configuredDshEnvironment),
  );
  const effectiveEnvironmentRef =
    environmentRef ||
    (realDsh && eligibleEnvironments.length === 1
      ? eligibleEnvironments[0].environment_ref
      : "");
  const activeBindingId = selectedWork?.status.active_binding ?? "";
  const activeBindingMatches = workBindings.some(
    (binding) => textField(binding, "id") === activeBindingId,
  );
  const effectiveBindingId =
    bindingId ||
    (activeBindingMatches ? activeBindingId : workBindings.length === 1 ? textField(workBindings[0], "id") ?? "" : "");

  const resolve = async () => {
    if (!effectiveWorkId) return;
    setBusy(true);
    setMessage(null);
    try {
      const response = await apiPostJson<{
        resolved: boolean;
        evidence_blockers: string[];
        work: { status: { phase: string } };
      }>("/v115/work/resolve", { work_id: effectiveWorkId });
      setMessage(
        response.resolved
          ? `Resolution recorded. Work phase: ${response.work.status.phase}.${response.evidence_blockers.length ? ` Remaining gates: ${response.evidence_blockers.join("; ")}` : ""}`
          : `Resolution incomplete: ${response.evidence_blockers.join("; ")}`,
      );
      reload();
    } catch (e) {
      setMessage((e as Error).message);
    } finally {
      setBusy(false);
    }
  };

  const bind = async () => {
    if (!effectiveWorkId || !effectiveCapabilityId || realPi) return;
    if (realDsh && !effectiveEnvironmentRef) return;
    setBusy(true);
    setMessage(null);
    try {
      const path = realDsh ? "/v115/work/bind-attested-e0" : "/v115/work/bind-e0";
      await apiPostJson(path, {
        work_id: effectiveWorkId,
        capability_manifest_id: effectiveCapabilityId,
        ...(realDsh ? { execution_environment_ref: effectiveEnvironmentRef } : {}),
      });
      setMessage(
        realDsh
          ? "Trusted environment and exact DSH runtime identity pinned. Execution has not started."
          : "Binding persisted. Execution has not started.",
      );
      reload();
    } catch (e) {
      setMessage((e as Error).message);
    } finally {
      setBusy(false);
    }
  };

  const execute = async () => {
    if (!effectiveWorkId || !effectiveBindingId || !prompt.trim()) return;
    setBusy(true);
    setMessage(null);
    try {
      const response = await apiPostJson<{
        executor_status: string;
        business_outcome_observed: boolean;
        independent_acceptance: boolean;
      }>("/v115/work/execute-e0", {
        work_id: effectiveWorkId,
        binding_id: effectiveBindingId,
        input: prompt,
      });
      setMessage(
        `Executor: ${response.executor_status}. Business outcome: ${response.business_outcome_observed ? "observed" : "not observed"}. Independent acceptance: ${response.independent_acceptance ? "yes" : "no"}.`,
      );
      reload();
    } catch (e) {
      setMessage((e as Error).message);
    } finally {
      setBusy(false);
    }
  };

  if (candidateWorks.length === 0) {
    return null;
  }

  return (
    <Card title="Governed E0 execution">
      <p>
        Resolve requirements from the approved SolutionPackage, persist an immutable E0 binding, then
        execute it as executor evidence. Real DSH/Pi bindings fail closed without trusted environment
        attestation. Harness completion never creates a business outcome or acceptance.
      </p>
      <div className="governed-execution-steps" aria-label="Governed execution stages">
        <span>1 · Resolve Workcell</span>
        <span>2 · Persist binding</span>
        <span>3 · Execute E0</span>
      </div>
      <div className="governed-execution-grid">
        <label>
          Work
          <select
            value={effectiveWorkId}
            onChange={(event) => {
              setWorkId(event.target.value);
              setCapabilityId("");
              setBindingId("");
              setEnvironmentRef("");
            }}
          >
            {candidateWorks.map((work) => (
              <option key={work.id} value={work.id}>
                {work.spec.goal} · {work.status.phase}
              </option>
            ))}
          </select>
        </label>
        <label>
          E0 capability
          <select
            value={effectiveCapabilityId}
            onChange={(event) => {
              setCapabilityId(event.target.value);
              setEnvironmentRef("");
            }}
            disabled={eligibleCapabilities.length === 0}
          >
            {eligibleCapabilities.length === 0 ? (
              <option value="">No qualified/admitted Harness E0 capability</option>
            ) : (
              eligibleCapabilities.map((capability) => (
                <option key={capability.manifest.id} value={capability.manifest.id}>
                  {capability.manifest.name} · {capability.manifest.provider_ref}
                </option>
              ))
            )}
          </select>
        </label>
      </div>
      {realDsh && (
        <label className="governed-execution-environment">
          Trusted execution environment
          <select
            value={effectiveEnvironmentRef}
            onChange={(event) => setEnvironmentRef(event.target.value)}
            disabled={eligibleEnvironments.length === 0}
          >
            {eligibleEnvironments.length === 0 ? (
              <option value="">No fresh matching deployment attestation</option>
            ) : eligibleEnvironments.length > 1 && !environmentRef ? (
              <>
                <option value="">Select an attested environment</option>
                {eligibleEnvironments.map((environment) => (
                  <option key={environment.environment_ref} value={environment.environment_ref}>
                    {environment.environment_ref} · {environment.isolation}
                  </option>
                ))}
              </>
            ) : (
              eligibleEnvironments.map((environment) => (
                <option key={environment.environment_ref} value={environment.environment_ref}>
                  {environment.environment_ref} · {environment.isolation}
                </option>
              ))
            )}
          </select>
          <small>
            Real DSH binding is allowed only when this fresh deployment attestation exactly matches
            the environment pinned by the DSH launch configuration.
          </small>
        </label>
      )}
      {realPi && (
        <p className="work-focus-alert" role="status">
          Real Pi binding remains fail-closed because the current RPC boundary does not expose a
          verifiable runtime version. Fixture Pi remains available for conformance only.
        </p>
      )}
      <div className="page-actions">
        <button
          disabled={busy || !selectedWork?.spec.source_solution_ref}
          onClick={resolve}
        >
          Resolve approved Solution requirements
        </button>
        <button
          disabled={
            busy ||
            selectedWork?.status.phase !== "Ready" ||
            !effectiveCapabilityId ||
            realPi ||
            (realDsh && !effectiveEnvironmentRef)
          }
          onClick={bind}
        >
          Persist binding
        </button>
        <span className="muted">
          {workBindings.length > 0
            ? `${workBindings.length} binding(s) persisted for this generation`
            : "No binding persisted for this generation"}
        </span>
      </div>
      <label className="governed-execution-binding">
        Execution binding
        <select
          value={effectiveBindingId}
          onChange={(event) => setBindingId(event.target.value)}
          disabled={workBindings.length === 0}
        >
          {workBindings.length === 0 ? (
            <option value="">No binding persisted for this generation</option>
          ) : workBindings.length > 1 && !activeBindingMatches && !bindingId ? (
            <>
              <option value="">Select an exact binding</option>
              {workBindings.map((binding, index) => {
                const id = textField(binding, "id") ?? "";
                return (
                  <option key={id || index} value={id}>
                    {textField(binding, "provider_ref") ?? "Unknown provider"} · {id || "Unknown binding"}
                  </option>
                );
              })}
            </>
          ) : (
            workBindings.map((binding, index) => {
              const id = textField(binding, "id") ?? "";
              return (
                <option key={id || index} value={id}>
                  {textField(binding, "provider_ref") ?? "Unknown provider"} · {id || "Unknown binding"}
                </option>
              );
            })
          )}
        </select>
        <small>
          {activeBindingMatches
            ? "Canonical active binding selected by default."
            : workBindings.length > 1 && !effectiveBindingId
              ? "Multiple bindings exist; execution is blocked until one is selected explicitly."
              : "Execution uses this exact immutable binding."}
        </small>
      </label>
      <label className="governed-execution-prompt">
        Executor input
        <textarea value={prompt} onChange={(event) => setPrompt(event.target.value)} />
      </label>
      <div className="page-actions">
        <button disabled={busy || !effectiveBindingId || !prompt.trim()} onClick={execute}>
          Execute bound E0 capability
        </button>
      </div>
      {message && <p role="status" className="work-focus-empty">{message}</p>}
    </Card>
  );
}


function AuthoritativeOutcomePanel({
  control,
  catalog,
  reload,
}: {
  control: V115ControlPlaneData;
  catalog: SourceOfTruthCatalog | null;
  reload: () => void;
}) {
  const [workId, setWorkId] = useState("");
  const [catalogId, setCatalogId] = useState("");
  const [sourceBindingId, setSourceBindingId] = useState("");
  const [factType, setFactType] = useState("");
  const [objective, setObjective] = useState("Observe authoritative business outcome");
  const [sourceRef, setSourceRef] = useState("");
  const [evidenceRef, setEvidenceRef] = useState("");
  const [facts, setFacts] = useState('{"status":"complete"}');
  const [busy, setBusy] = useState(false);
  const [message, setMessage] = useState<string | null>(null);

  const works = control.work.filter((work) => !["Accepted", "Rejected", "Cancelled"].includes(work.status.phase));
  const effectiveWorkId = workId || works[0]?.id || "";
  const work = works.find((item) => item.id === effectiveWorkId);
  const catalogBindings = (catalog?.bindings ?? []).filter(
    (binding) => (binding.site_ref ?? null) === (work?.spec.site_ref ?? null),
  );
  const effectiveCatalogId = catalogId || (catalogBindings.length === 1 ? catalogBindings[0].id : "");

  const boundIds = new Set(
    control.condition_evidence
      .filter((entry) =>
        textField(entry, "work_ref") === effectiveWorkId &&
        entry.work_generation === work?.generation &&
        textField(entry, "condition_type") === "SourceOfTruthBound" &&
        entry.satisfied === true)
      .flatMap((entry) => fieldRefs(entry, "evidence_refs")),
  );
  const workBindings = control.source_of_truth_bindings.filter((binding) => {
    const id = textField(binding, "id");
    return id !== null && boundIds.has(id);
  });
  const effectiveSourceBindingId =
    sourceBindingId || (workBindings.length === 1 ? textField(workBindings[0], "id") ?? "" : "");
  const sourceBinding = workBindings.find((binding) => textField(binding, "id") === effectiveSourceBindingId);
  const factTypes = fieldRefs(sourceBinding ?? {}, "authoritative_fact_types");
  const effectiveFactType = factType || (factTypes.length === 1 ? factTypes[0] : "");

  const attach = async () => {
    if (!effectiveWorkId || !effectiveCatalogId) return;
    setBusy(true); setMessage(null);
    try {
      await apiPostJson("/v115/work/bind-source-of-truth", {
        work_id: effectiveWorkId,
        catalog_binding_id: effectiveCatalogId,
      });
      setSourceBindingId("");
      setMessage("Deployment-owned source authority attached to this Work generation.");
      reload();
    } catch (e) { setMessage((e as Error).message); }
    finally { setBusy(false); }
  };

  const observe = async () => {
    if (!effectiveWorkId || !effectiveSourceBindingId || !effectiveFactType ||
        !sourceRef.trim() || !evidenceRef.trim() || !objective.trim()) return;
    let observedFacts: Record<string, unknown>;
    try {
      const parsed = JSON.parse(facts) as unknown;
      if (!parsed || Array.isArray(parsed) || typeof parsed !== "object") throw new Error("Observed facts must be a JSON object.");
      observedFacts = parsed as Record<string, unknown>;
    } catch (e) { setMessage((e as Error).message); return; }
    setBusy(true); setMessage(null);
    try {
      await apiPostJson("/v115/work/observe-outcome", {
        work_id: effectiveWorkId,
        source_binding_id: effectiveSourceBindingId,
        fact_type: effectiveFactType,
        objective: objective.trim(),
        source_ref: sourceRef.trim(),
        observed_facts: observedFacts,
        evidence_refs: [evidenceRef.trim()],
      });
      setMessage("Source-grounded Outcome persisted. Independent review is still required.");
      reload();
    } catch (e) { setMessage((e as Error).message); }
    finally { setBusy(false); }
  };

  if (works.length === 0) {
    return (
      <Card title="Authoritative outcome observation">
        <p>
          No non-terminal canonical Work is available. Deployment source authority cannot be
          attached until a Work exists; executor output is never promoted implicitly.
        </p>
      </Card>
    );
  }
  return (
    <Card title="Authoritative outcome observation">
      <p>
        Attach a deployment-reviewed source of truth, then persist an authoritative world observation.
        Executor/model output cannot be promoted here.
      </p>
      <div className="governed-outcome-grid">
        <label>Work
          <select value={effectiveWorkId} onChange={(e) => {
            setWorkId(e.target.value); setCatalogId(""); setSourceBindingId(""); setFactType("");
          }}>
            {works.map((item) => <option key={item.id} value={item.id}>{item.spec.goal} · {item.status.phase}</option>)}
          </select>
        </label>
        <label>Deployment authority
          <select value={effectiveCatalogId} onChange={(e) => setCatalogId(e.target.value)} disabled={catalogBindings.length === 0}>
            {catalogBindings.length === 0 && <option value="">No reviewed source for this site</option>}
            {catalogBindings.length > 1 && !catalogId && <option value="">Select reviewed authority</option>}
            {catalogBindings.map((binding) =>
              <option key={binding.id} value={binding.id}>{binding.source_ref} · {binding.authority_kind}</option>)}
          </select>
        </label>
      </div>
      <div className="page-actions">
        <button disabled={busy || !effectiveCatalogId} onClick={attach}>Attach authoritative source</button>
        <span className="muted">{workBindings.length} Work-scoped source binding(s)</span>
      </div>
      <div className="governed-outcome-grid">
        <label>Work-scoped source
          <select value={effectiveSourceBindingId} onChange={(e) => { setSourceBindingId(e.target.value); setFactType(""); }} disabled={workBindings.length === 0}>
            {workBindings.length === 0 && <option value="">Attach a source first</option>}
            {workBindings.length > 1 && !sourceBindingId && <option value="">Select exact source binding</option>}
            {workBindings.map((binding, index) => {
              const id = textField(binding, "id") ?? "";
              return <option key={id || index} value={id}>{textField(binding, "source_ref") ?? id}</option>;
            })}
          </select>
        </label>
        <label>Authoritative fact type
          <select value={effectiveFactType} onChange={(e) => setFactType(e.target.value)} disabled={factTypes.length === 0}>
            {factTypes.length === 0 && <option value="">No authoritative fact type</option>}
            {factTypes.length > 1 && !factType && <option value="">Select fact type</option>}
            {factTypes.map((value) => <option key={value} value={value}>{value}</option>)}
          </select>
        </label>
        <label>Observation objective<input value={objective} onChange={(e) => setObjective(e.target.value)} /></label>
        <label>Source record URI<input value={sourceRef} onChange={(e) => setSourceRef(e.target.value)}
          placeholder={textField(sourceBinding ?? {}, "source_ref") ?? "system://authoritative/record"} /></label>
        <label>Evidence reference<input value={evidenceRef} onChange={(e) => setEvidenceRef(e.target.value)}
          placeholder="system://authoritative/record/receipt" /></label>
        <label className="governed-outcome-facts">Observed facts (JSON)
          <textarea value={facts} onChange={(e) => setFacts(e.target.value)} />
        </label>
      </div>
      <div className="page-actions">
        <button disabled={busy || !effectiveSourceBindingId || !effectiveFactType || !sourceRef.trim() || !evidenceRef.trim()} onClick={observe}>
          Persist source-grounded Outcome
        </button>
      </div>
      {message && <p role="status" className="work-focus-empty">{message}</p>}
    </Card>
  );
}

function OutcomeReviewPanel({
  control,
  reviewers,
  reload,
}: {
  control: V115ControlPlaneData;
  reviewers: AcceptanceReviewerCatalog | null;
  reload: () => void;
}) {
  const [workId, setWorkId] = useState("");
  const [outcomeId, setOutcomeId] = useState("");
  const [disposition, setDisposition] = useState("accept");
  const [reviewerPrincipalId, setReviewerPrincipalId] = useState("");
  const [actingRole, setActingRole] = useState("");
  const [reason, setReason] = useState("");
  const [evidenceRef, setEvidenceRef] = useState("");
  const [busy, setBusy] = useState(false);
  const [message, setMessage] = useState<string | null>(null);

  const candidates = control.work
    .filter((work) => ["Delivered", "Waiting"].includes(work.status.phase))
    .map((work) => ({
      work,
      outcomes: control.outcomes.filter(
        (outcome) =>
          textField(outcome, "work_package_id") === work.id &&
          !!textField(outcome, "source_ref") &&
          fieldRefs(outcome, "evidence_refs").length > 0,
      ),
    }))
    .filter((entry) => entry.outcomes.length > 0);

  const effectiveWorkId = workId || candidates[0]?.work.id || "";
  const selected = candidates.find((entry) => entry.work.id === effectiveWorkId);
  const effectiveOutcomeId = outcomeId || textField(selected?.outcomes[0] ?? {}, "id") || "";
  const selectedOutcome = selected?.outcomes.find(
    (outcome) => textField(outcome, "id") === effectiveOutcomeId,
  );
  const availableReviewers = reviewers?.reviewers ?? [];
  const effectiveReviewerPrincipalId =
    reviewerPrincipalId || (availableReviewers.length === 1 ? availableReviewers[0].principal_id : "");
  const selectedReviewer = availableReviewers.find(
    (reviewer) => reviewer.principal_id === effectiveReviewerPrincipalId,
  );
  const effectiveActingRole =
    actingRole || (selectedReviewer?.acting_roles.length === 1 ? selectedReviewer.acting_roles[0] : "");


  const review = async () => {
    if (
      !effectiveWorkId ||
      !effectiveOutcomeId ||
      !effectiveReviewerPrincipalId ||
      !effectiveActingRole ||
      !reason.trim() ||
      !evidenceRef.trim()
    ) {
      return;
    }
    setBusy(true);
    setMessage(null);
    try {
      const response = await apiPostJson<{
        work: { status: { phase: string } };
        decision: { disposition: string };
      }>("/v115/work/review-outcome", {
        work_id: effectiveWorkId,
        outcome_id: effectiveOutcomeId,
        disposition,
        reviewer_principal_id: effectiveReviewerPrincipalId,
        acting_role: effectiveActingRole,
        reason,
        evidence_refs: [evidenceRef.trim()],
      });
      setMessage(
        `Review persisted: ${response.decision.disposition}. Work phase: ${response.work.status.phase}.`,
      );
      reload();
    } catch (e) {
      setMessage((e as Error).message);
    } finally {
      setBusy(false);
    }
  };

  if (candidates.length === 0) {
    return (
      <Card title="Independent outcome review">
        <p>
          No source-grounded Outcome is ready for review. Harness completion alone cannot create one;
          an authoritative observation must be persisted first.
        </p>
      </Card>
    );
  }

  return (
    <Card title="Independent outcome review">
      <p>
        Review only an already-persisted source-grounded Outcome. This action cannot convert model
        output or a Harness receipt into business truth.
      </p>
      <div className="governed-execution-grid">
        <label>
          Work
          <select
            value={effectiveWorkId}
            onChange={(event) => {
              setWorkId(event.target.value);
              setOutcomeId("");
            }}
          >
            {candidates.map(({ work }) => (
              <option key={work.id} value={work.id}>
                {work.spec.goal} · {work.status.phase}
              </option>
            ))}
          </select>
        </label>
        <label>
          Source-grounded Outcome
          <select value={effectiveOutcomeId} onChange={(event) => setOutcomeId(event.target.value)}>
            {selected?.outcomes.map((outcome, index) => {
              const id = textField(outcome, "id") ?? `outcome-${index}`;
              return (
                <option key={id} value={textField(outcome, "id") ?? ""}>
                  {textField(outcome, "objective") ?? id}
                </option>
              );
            })}
          </select>
        </label>
        <label>
          Decision
          <select value={disposition} onChange={(event) => setDisposition(event.target.value)}>
            <option value="accept">Accept</option>
            <option value="reject">Reject</option>
            <option value="conditional">Conditional</option>
            <option value="request-more-evidence">Request more evidence</option>
          </select>
        </label>
        <label>
          Deployment-attested reviewer
          <select
            value={effectiveReviewerPrincipalId}
            onChange={(event) => {
              setReviewerPrincipalId(event.target.value);
              setActingRole("");
            }}
            disabled={availableReviewers.length === 0}
          >
            {availableReviewers.length === 0 && <option value="">No independent reviewer attested</option>}
            {availableReviewers.length > 1 && !reviewerPrincipalId && <option value="">Select reviewer</option>}
            {availableReviewers.map((reviewer) => (
              <option key={reviewer.principal_id} value={reviewer.principal_id}>
                {reviewer.principal_id}
              </option>
            ))}
          </select>
        </label>
        <label>
          Attested reviewer role
          <select
            value={effectiveActingRole}
            onChange={(event) => setActingRole(event.target.value)}
            disabled={!selectedReviewer}
          >
            {!selectedReviewer && <option value="">Select reviewer first</option>}
            {selectedReviewer && selectedReviewer.acting_roles.length > 1 && !actingRole && (
              <option value="">Select attested role</option>
            )}
            {selectedReviewer?.acting_roles.map((role) => (
              <option key={role} value={role}>{role}</option>
            ))}
          </select>
        </label>
      </div>
      {selectedOutcome && (
        <p className="work-focus-empty">
          Source: {textField(selectedOutcome, "source_ref")} · witness refs:{" "}
          {fieldRefs(selectedOutcome, "evidence_refs").length}
        </p>
      )}
      <label className="governed-execution-prompt">
        Review reason
        <textarea
          value={reason}
          onChange={(event) => setReason(event.target.value)}
          placeholder="Explain how the authoritative evidence satisfies or fails the acceptance criteria."
        />
      </label>
      <label className="governed-execution-prompt">
        Independent review evidence reference
        <input
          value={evidenceRef}
          onChange={(event) => setEvidenceRef(event.target.value)}
          placeholder="review://ticket-or-signed-record"
        />
      </label>
      <div className="page-actions">
        <button
          disabled={
            busy ||
            !effectiveOutcomeId ||
            !effectiveReviewerPrincipalId ||
            !effectiveActingRole ||
            !reason.trim() ||
            !evidenceRef.trim()
          }
          onClick={review}
        >
          Persist independent review
        </button>
      </div>
      {message && <p role="status" className="work-focus-empty">{message}</p>}
    </Card>
  );
}

export default function Workbench() {
  const [data, setData] = useState<WorkbenchData | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);
  const [durableRun, setDurableRun] = useState<DurableRun | null>(null);
  const [durableRuns, setDurableRuns] = useState<Array<{ id: string; status: string }>>([]);
  const [evaluation, setEvaluation] = useState<EvaluationOutcome | null>(null);
  const [shadow, setShadow] = useState<ShadowOutcome | null>(null);
  const [replay, setReplay] = useState<ReplayOutcome | null>(null);
  const [loopA, setLoopA] = useState<LoopAOutcome | null>(null);
  const [loopC, setLoopC] = useState<string | null>(null);
  const [flywheel, setFlywheel] = useState<FlywheelOutcome | null>(null);
  const [distill, setDistill] = useState<DistillOutcome | null>(null);
  const [managedRun, setManagedRun] = useState<ManagedOutcome | null>(null);
  const [replacement, setReplacement] = useState<ReplacementOutcome | null>(null);
  const [opintPrediction, setOpintPrediction] = useState<OpintPredictOutcome | null>(null);
  const [v115, setV115] = useState<V115Status | null>(null);
  const [v115Control, setV115Control] = useState<V115ControlPlaneData | null>(null);
  const [v115ControlError, setV115ControlError] = useState<string | null>(null);
  const [v115ControlLoading, setV115ControlLoading] = useState(true);
  const [uiExtensions, setUiExtensions] = useState<UiExtensionRegistry | null>(null);
  const [v115Capabilities, setV115Capabilities] = useState<E0HarnessCapability[]>([]);
  const [sourceTruthCatalog, setSourceTruthCatalog] = useState<SourceOfTruthCatalog | null>(null);
  const [reviewerCatalog, setReviewerCatalog] = useState<AcceptanceReviewerCatalog | null>(null);

  const load = useCallback(() => {
    setLoading(true);
    setError(null);
    apiGet<WorkbenchData>("/workbench")
      .then(setData)
      .catch((e: Error) => setError(e.message))
      .finally(() => setLoading(false));
    apiGet<V115Status>("/v115/status")
      .then(setV115)
      .catch(() => undefined);
    setV115Control(null);
    setV115ControlError(null);
    setV115ControlLoading(true);
    apiGet<V115ControlPlaneData>("/v115/control-plane")
      .then(setV115Control)
      .catch((e: Error) => setV115ControlError(e.message))
      .finally(() => setV115ControlLoading(false));
    apiGet<UiExtensionRegistry>("/v115/ui/extensions")
      .then(setUiExtensions)
      .catch(() => undefined);
    apiGet<{ capabilities: E0HarnessCapability[] }>("/v115/capabilities")
      .then((response) => setV115Capabilities(response.capabilities))
      .catch(() => setV115Capabilities([]));
    apiGet<SourceOfTruthCatalog>("/v115/source-of-truth/catalog")
      .then(setSourceTruthCatalog)
      .catch(() => setSourceTruthCatalog(null));
    apiGet<AcceptanceReviewerCatalog>("/v115/acceptance/reviewers")
      .then(setReviewerCatalog)
      .catch(() => setReviewerCatalog(null));
  }, []);

  useEffect(load, [load]);

  const runUiExtensionAction = async (method: "GET" | "POST", endpoint: string) => {
    setError(null);
    try {
      const path = endpoint.startsWith("/api/") ? endpoint.slice(4) : endpoint;
      if (method === "POST") {
        await apiPost(path);
      } else {
        await apiGet(path);
      }
      load();
    } catch (e) {
      setError((e as Error).message);
    }
  };

  const runE2e = async () => {
    setError(null);
    try {
      await apiPost("/biolab/run");
      load();
    } catch (e) {
      setError((e as Error).message);
    }
  };

  if (loading && v115ControlLoading && !data) return <Loading />;
  if (error || !data) {
    return (
      <div className="page">
        <header className="page-header">
          <h1>Workbench</h1>
        </header>
        <CanonicalWorkOverview control={v115Control} error={v115ControlError} />
        {v115Control && (
          <>
            <GovernedE0Executor
              control={v115Control}
              capabilities={v115Capabilities}
              status={v115}
              reload={load}
            />
            <AuthoritativeOutcomePanel control={v115Control} catalog={sourceTruthCatalog} reload={load} />
            <OutcomeReviewPanel control={v115Control} reviewers={reviewerCatalog} reload={load} />
          </>
        )}
        <p role="status" className="work-focus-empty">
          {error
            ? `Legacy diagnostic data is unavailable: ${error}. Canonical Work remains authoritative.`
            : loading
              ? "Loading legacy diagnostics; canonical Work remains available independently."
              : "No legacy diagnostic data is available. Canonical Work remains authoritative."}
        </p>
      </div>
    );
  }
  const biolab = biolabEnabled(data.domain_packs);

  const startDurable = async () => {
    try {
      const r = await apiPost<DurableRun>("/durable/start");
      setDurableRun(r);
      const all = await apiGet<{ runs: Array<{ id: string; status: string }> }>("/durable/runs");
      setDurableRuns(all.runs);
    } catch (e) {
      setError((e as Error).message);
    }
  };

  const signalRun = async () => {
    if (!durableRun) return;
    try {
      await apiPostJson("/durable/signal", { run_id: durableRun.run.id, kind: "HumanApproval", identity: "pi-1", authority: "pi" });
      const all = await apiGet<{ runs: Array<{ id: string; status: string }> }>("/durable/runs");
      setDurableRuns(all.runs);
    } catch (e) {
      setError((e as Error).message);
    }
  };

  const runEvaluation = async () => {
    try {
      const r = await apiPostJson<EvaluationOutcome>("/evaluation/run", {
        scenario: "bio",
        faults: [{ kind: "ApprovalMissing", target_step: "release" }],
      });
      setEvaluation(r);
    } catch (e) {
      setError((e as Error).message);
    }
  };

  const runShadow = async () => {
    try {
      const r = await apiPostJson<ShadowOutcome>("/shadow/compare", {
        faults: [{ kind: "PermissionDenied", target_step: "release" }],
      });
      setShadow(r);
    } catch (e) {
      setError((e as Error).message);
    }
  };

  const runReplay = async () => {
    try {
      const r = await apiPostJson<ReplayOutcome>("/replay/run", { mutate_step: "analyze" });
      setReplay(r);
    } catch (e) {
      setError((e as Error).message);
    }
  };

  const runLoopA = async () => {
    try {
      const r = await apiPostJson<LoopAOutcome>("/biolab/loop-a", {
        question: "Is mechanism X reproducible?",
        sources: [{ name: "S1", source_ref: "doi:1", evidence_type: "single_cell", conclusion: "present" }],
      });
      setLoopA(r);
    } catch (e) {
      setError((e as Error).message);
    }
  };

  const runEvolutionAnalyze = async () => {
    try {
      const r = await apiPost<FlywheelOutcome>("/evolution/analyze");
      setFlywheel(r);
    } catch (e) {
      setError((e as Error).message);
    }
  };

  const runDistill = async () => {
    try {
      const r = await apiPost<DistillOutcome>("/distill/run");
      setDistill(r);
    } catch (e) {
      setError((e as Error).message);
    }
  };

  const startManaged = async () => {
    try {
      await apiPost<CertifyOutcome>("/certify/run");
      const r = await apiPost<ManagedOutcome>("/managed/start");
      setManagedRun(r);
    } catch (e) {
      setError((e as Error).message);
    }
  };

  const deliverAndAcceptManaged = async () => {
    if (!managedRun) return;
    try {
      await apiPostJson("/managed/deliver", { run_id: managedRun.run.id });
      await apiPostJson("/managed/accept", { run_id: managedRun.run.id, decided_by: "pi" });
      const r = await apiPost<ManagedOutcome>("/managed/start");
      setManagedRun(r);
    } catch (e) {
      setError((e as Error).message);
    }
  };

  const runOpint = async () => {
    try {
      await apiPostJson("/opint/predictor/train", { target: "outcome_acceptance" });
      const p = await apiPostJson<OpintPredictOutcome>("/opint/predict", { target: "outcome_acceptance", context: "biolab" });
      setOpintPrediction(p);
    } catch (e) {
      setError((e as Error).message);
    }
  };

  const runReplacementShadow = async () => {
    try {
      const r = await apiPost<ReplacementOutcome>("/replacement/shadow");
      setReplacement(r);
    } catch (e) {
      setError((e as Error).message);
    }
  };

  const runLoopC = async () => {
    try {
      await apiPost("/biolab/loop-c");
      setLoopC("done");
    } catch (e) {
      setError((e as Error).message);
    }
  };

  return (
    <div className="page">
      <header className="page-header">
        <h1>Workbench</h1>
        <div className="page-actions">
          {biolab && !uiExtensions && <button onClick={runE2e}>Run BioLab E2E</button>}
        </div>
      </header>

      <CanonicalWorkOverview control={v115Control} error={v115ControlError} />
      {v115Control && (
        <>
          <GovernedE0Executor
            control={v115Control}
            capabilities={v115Capabilities}
            status={v115}
            reload={load}
          />
          <AuthoritativeOutcomePanel control={v115Control} catalog={sourceTruthCatalog} reload={load} />
            <OutcomeReviewPanel control={v115Control} reviewers={reviewerCatalog} reload={load} />
        </>
      )}

      <details className="workbench-reference" data-testid="reference-tools">
        <summary>Reference runs &amp; engineering diagnostics</summary>
        <p className="workbench-reference-note">
          These are optional legacy/fixture execution tools. Their status, model output and
          demos are not a substitute for canonical Work, observed outcomes or independent acceptance.
        </p>
        <div className="grid">
        <Card title="Mission">
          <KeyValue k="Name" v={data.mission.name} />
          <KeyValue k="Kind" v={data.mission.kind} />
          <KeyValue k="Status" v={<StatusPill value={data.mission.status} />} />
        </Card>

        <Card title="Operational World Objects">
          {data.world_objects.length === 0 ? (
            <EmptyState label="No objects" />
          ) : (
            <table>
              <thead>
                <tr>
                  <th>id</th>
                  <th>type</th>
                  <th>version</th>
                </tr>
              </thead>
              <tbody>
                {data.world_objects.map((o) => (
                  <tr key={o.id}>
                    <td>{o.id}</td>
                    <td>{o.type}</td>
                    <td>{o.version}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          )}
        </Card>

        <Card title="Work Packages">
          {data.work_packages.length === 0 ? (
            <EmptyState label="No work packages" />
          ) : (
            <ul>
              {data.work_packages.map((wp) => (
                <li key={wp.id}>
                  {wp.objective} — <StatusPill value={wp.status} />
                </li>
              ))}
            </ul>
          )}
        </Card>

        <Card title="Artifacts & Outcomes">
          <KeyValue k="Artifact versions" v={data.artifacts.versions} />
          <KeyValue k="Outcomes" v={data.outcomes.length} />
          {data.outcomes.map((o) => (
            <div key={o.id} className="kv">
              <span className="kv-key">{o.objective}</span>
              <span className="kv-value">{o.acceptance_met ? "accepted" : "pending"}</span>
            </div>
          ))}
        </Card>

        <Card title="Attention Queue">
          {data.attention.length === 0 ? (
            <EmptyState label="No open attention items" />
          ) : (
            <ul>
              {data.attention.map((a) => (
                <li key={a.id}>
                  {a.kind} — {a.subject} <StatusPill value={a.priority} />
                </li>
              ))}
            </ul>
          )}
        </Card>

        <Card title="Harness / Runtime Health">
          <KeyValue k="Native" v={`${data.harness.native.provider} (${data.harness.native.status})`} />
          <KeyValue k="DeepSeek Harness" v={`${data.harness.dsh.provider} (${data.harness.dsh.status})`} />
          <KeyValue k="Evolution candidates" v={data.evolution_candidates} />
        </Card>

        {uiExtensions?.extensions
          .filter((extension) => extension.surface === "workbench")
          .map((extension) => (
            <Card key={extension.id} title={`Extension · ${extension.title}`}>
              <KeyValue k="Domain" v={extension.domain} />
              <KeyValue k="Slot" v={extension.slot} />
              <KeyValue k="Renderer" v={extension.renderer} />
              <KeyValue
                k="Safety model"
                v={uiExtensions.arbitrary_remote_js ? "remote JS enabled" : "declarative / no arbitrary remote JS"}
              />
              {extension.actions.length > 0 && (
                <div className="page-actions" style={{ marginTop: 8 }}>
                  {extension.actions.map((action) => (
                    <button
                      key={action.id}
                      onClick={() => runUiExtensionAction(action.method, action.endpoint)}
                    >
                      {action.label}
                    </button>
                  ))}
                </div>
              )}
            </Card>
          ))}

        {v115Control && (
          <Card title="v11.5 Durable Work Truth">
            <KeyValue k="Work resources" v={v115Control.work.length} />
            <KeyValue k="Condition evidence" v={v115Control.condition_evidence.length} />
            <KeyValue k="Profile conformance attestations" v={v115Control.profile_conformance_attestations.length} />
            <KeyValue k="Bindings" v={v115Control.execution_bindings.length} />
            <KeyValue k="Execution manifests" v={v115Control.execution_manifests.length} />
            <KeyValue k="Binding migrations" v={v115Control.binding_migrations.length} />
            <KeyValue k="Attempts" v={v115Control.attempts.length} />
            <KeyValue k="Reconciliations" v={v115Control.reconciliations.length} />
            <KeyValue k="Observed outcomes" v={v115Control.outcomes.length} />
            <KeyValue k="Acceptance decisions" v={v115Control.acceptance_decisions.length} />
            {v115Control.work.length === 0 ? (
              <EmptyState label="No persisted v11.5 Work yet" />
            ) : (
              <ul>
                {v115Control.work.map((work) => (
                  <li key={work.id}>
                    {work.spec.goal} — <StatusPill value={work.status.phase} /> — generation{" "}
                    {work.status.observed_generation}/{work.generation}
                  </li>
                ))}
              </ul>
            )}
          </Card>
        )}

        {v115 && (
          <Card title="Morn v11.5 Control Plane">
            <KeyValue
              k="Protocol"
              v={`${v115.architecture.protocol_version.major}.${v115.architecture.protocol_version.minor}.${v115.architecture.protocol_version.patch}`}
            />
            <KeyValue k="Control model" v={v115.architecture.control_model} />
            <KeyValue
              k="Composition"
              v={`${v115.architecture.composition_runtime.name} ${v115.architecture.composition_runtime.reference_version} — ${v115.architecture.composition_runtime.role}`}
            />
            <KeyValue k="Factory profile" v={v115.factory_profile.id} />
            <KeyValue k="Isolation floor" v={v115.factory_profile.minimum_isolation} />
            <KeyValue
              k="Execution guarantees"
              v={v115.factory_profile.required_execution_guarantees.join(", ")}
            />
            <KeyValue k="Production write" v={v115.factory_profile.production_write ? "enabled" : "not entered"} />
            <KeyValue k="Capability lifecycle" v={v115.capability_supply_chain.stages.join(" → ")} />
          </Card>
        )}
      </div>

      <Card title="Durable Workflow Provider (legacy-compatible execution state)">
        <div className="page-actions" style={{ marginBottom: 8 }}>
          <button onClick={startDurable}>Start Durable Run</button>
          <button onClick={signalRun} disabled={!durableRun}>
            Signal Approval
          </button>
        </div>
        {durableRuns.length === 0 ? (
          <EmptyState label="No durable runs yet" />
        ) : (
          <ul>
            {durableRuns.map((r) => (
              <li key={r.id}>
                {r.id} — <StatusPill value={r.status} />
              </li>
            ))}
          </ul>
        )}
      </Card>

      <Card title="Replay / Shadow / Evaluation">
        <div className="page-actions" style={{ marginBottom: 8 }}>
          <button onClick={runReplay}>Run Replay (drift)</button>
          <button onClick={runShadow}>Shadow Compare</button>
          <button onClick={runEvaluation}>Evaluate (approval missing)</button>
        </div>
        {replay && <KeyValue k="Replay reproduced" v={replay.replay_report.reproduced ? "yes" : "no (drift detected)"} />}
        {shadow && <KeyValue k="Shadow readiness" v={shadow.shadow_run.comparison.readiness} />}
        {evaluation && <KeyValue k="Evaluation decision" v={evaluation.result.decision} />}
      </Card>

      {biolab && (
      <Card title="BioLab Dream Factory — Loops A & C">
        <div className="page-actions" style={{ marginBottom: 8 }}>
          <button onClick={runLoopA}>Run Loop A</button>
          <button onClick={runLoopC} disabled={!data.e2e_result}>
            Run Loop C
          </button>
        </div>
        {loopA && <KeyValue k="Loop A" v={`approved=${loopA.loop_a.pi_approved} ok=${loopA.loop_a.all_ok}`} />}
        {loopC && <KeyValue k="Loop C" v="manuscript release candidate created" />}
      </Card>
      )}

      <Card title="Evolution Center (v0.3)">
        <div className="page-actions" style={{ marginBottom: 8 }}>
          <button onClick={runEvolutionAnalyze}>Detect Patterns & Candidates</button>
          <button onClick={runDistill}>Distill QC Step</button>
        </div>
        {flywheel && (
          <>
            <KeyValue k="Patterns" v={flywheel.patterns.length} />
            <ul>
              {flywheel.candidates.map((c) => (
                <li key={c.id}>
                  {c.candidate_type}: {c.proposed_change}
                </li>
              ))}
            </ul>
          </>
        )}
        {distill && (
          <KeyValue
            k="Distillation regression"
            v={`passed=${distill.regression.passed} matches=${distill.regression.program_matches_actor} fallback=${distill.regression.long_tail_fallback_count}`}
          />
        )}
      </Card>

      <Card title="Managed Work / Outcome Delivery (v0.3)">
        <div className="page-actions" style={{ marginBottom: 8 }}>
          <button onClick={startManaged}>Certify & Start Managed Work</button>
          <button onClick={deliverAndAcceptManaged} disabled={!managedRun}>
            Deliver & Accept
          </button>
        </div>
        {managedRun && <KeyValue k="Run" v={`${managedRun.run.id} — ${managedRun.run.status}`} />}
      </Card>

      <Card title="Operational Intelligence (v0.4)">
        <div className="page-actions" style={{ marginBottom: 8 }}>
          <button onClick={runOpint}>Train & Predict Outcome Acceptance</button>
        </div>
        {opintPrediction && (
          <>
            <KeyValue k="Prediction" v={opintPrediction.prediction.value.toFixed(3)} />
            <KeyValue k="Interval" v={`[${opintPrediction.prediction.interval_lo.toFixed(3)}, ${opintPrediction.prediction.interval_hi.toFixed(3)}]`} />
            <KeyValue k="Confidence" v={opintPrediction.prediction.confidence.toFixed(3)} />
            <KeyValue k="Context match" v={opintPrediction.prediction.context_match ? "yes" : "no"} />
          </>
        )}
      </Card>

      <Card title="Replacement Compare (v0.3)">
        <div className="page-actions" style={{ marginBottom: 8 }}>
          <button onClick={runReplacementShadow}>Shadow Compare Baseline vs Candidate</button>
        </div>
        {replacement && (
          <>
            <KeyValue k="Meets critical" v={replacement.comparison.candidate_meets_critical ? "yes" : "no"} />
            <KeyValue k="Quality" v={`manual=${replacement.comparison.baseline.quality} → native=${replacement.comparison.candidate.quality}`} />
            <KeyValue k="Human minutes" v={`manual=${replacement.comparison.baseline.human_minutes} → native=${replacement.comparison.candidate.human_minutes}`} />
            <KeyValue k="Cost" v={`manual=${replacement.comparison.baseline.cost_estimate} → native=${replacement.comparison.candidate.cost_estimate}`} />
          </>
        )}
      </Card>

      {data.e2e_result && (
        <Card title="BioLab E2E — Dataset → Reviewed Claim">
          <KeyValue k="Claim" v={data.e2e_result.claim_id} />
          <KeyValue k="All steps" v={data.e2e_result.all_ok ? "passed" : "failed"} />
          <ol>
            {data.e2e_result.steps.map((s) => (
              <li key={s.step}>
                {s.step}: {s.detail} {s.ok ? "✓" : "✗"}
              </li>
            ))}
          </ol>
        </Card>
      )}
      </details>
    </div>
  );
}
