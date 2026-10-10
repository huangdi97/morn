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
  SourceObservationCatalog,
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

export function uiExtensionEnabledForProfiles(
  requiredProfile: string | null,
  activeProfiles: string[],
): boolean {
  return requiredProfile === null || activeProfiles.includes(requiredProfile);
}

type UiExtension = UiExtensionRegistry["extensions"][number];

function safeExtensionPreview(payload: unknown): string {
  const serialized = JSON.stringify(payload, null, 2);
  if (!serialized) return "No serializable data";
  return serialized.length > 8000
    ? `${serialized.slice(0, 8000)}\n… payload truncated by safe renderer`
    : serialized;
}

function UiExtensionData({
  extension,
  enabled,
}: {
  extension: UiExtension;
  enabled: boolean;
}) {
  const [payload, setPayload] = useState<unknown>(null);
  const [loadError, setLoadError] = useState<string | null>(null);

  useEffect(() => {
    if (!enabled || !extension.data_endpoint) {
      setPayload(null);
      setLoadError(null);
      return;
    }
    if (!extension.data_endpoint.startsWith("/api/")) {
      setPayload(null);
      setLoadError("Unsafe extension data endpoint rejected.");
      return;
    }
    const path = extension.data_endpoint.slice(4);
    apiGet<unknown>(path)
      .then((value) => {
        setPayload(value);
        setLoadError(null);
      })
      .catch((error: Error) => {
        setPayload(null);
        setLoadError(error.message);
      });
  }, [enabled, extension.data_endpoint]);

  if (!extension.data_endpoint) return null;
  if (!enabled) return <p className="work-focus-empty">Profile gate blocks extension data access.</p>;
  if (loadError) return <p role="alert">Extension data unavailable: {loadError}</p>;
  if (payload === null) return <Loading />;

  if (
    extension.renderer === "key-value" &&
    typeof payload === "object" &&
    payload !== null &&
    !Array.isArray(payload)
  ) {
    const fields = Object.entries(payload as Record<string, unknown>)
      .filter(([, value]) => ["string", "number", "boolean"].includes(typeof value) || value === null)
      .slice(0, 12);
    if (fields.length > 0) {
      return (
        <div className="ui-extension-safe-data">
          {fields.map(([key, value]) => (
            <KeyValue key={key} k={key} v={String(value ?? "null")} />
          ))}
        </div>
      );
    }
  }

  if (extension.renderer === "status" && typeof payload === "object" && payload !== null) {
    const status =
      Object.entries(payload as Record<string, unknown>).find(
        ([key, value]) => /status|state|phase/i.test(key) && typeof value === "string",
      )?.[1] ?? "available";
    return <StatusPill value={String(status)} />;
  }

  return (
    <pre className="ui-extension-safe-json" aria-label={`${extension.title} data`}>
      {safeExtensionPreview(payload)}
    </pre>
  );
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
  const interopBindings = control.interop_bindings.filter((row) => {
    const ref = textField(row, "execution_binding_ref");
    return ref !== null && bindingIds.has(ref) && textField(row, "work_ref") === workId;
  });
  const interopBindingIds = new Set(
    interopBindings
      .map((row) => textField(row, "execution_binding_ref"))
      .filter((id): id is string => !!id),
  );
  const externalTasks = control.external_task_observations.filter((row) => {
    const bindingRef = textField(row, "execution_binding_ref");
    return (
      textField(row, "work_id") === workId &&
      row.work_generation === generation &&
      bindingRef !== null &&
      bindingIds.has(bindingRef) &&
      interopBindingIds.has(bindingRef)
    );
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
  const outcomes = control.outcomes.filter(
    (row) =>
      textField(row, "work_package_id") === workId &&
      row.work_generation === generation,
  );
  const outcomeIds = new Set(outcomes.map((row) => textField(row, "id")).filter((id): id is string => !!id));
  const acceptances = control.acceptance_decisions.filter(
    (row) =>
      textField(row, "work_package_id") === workId &&
      row.work_generation === generation &&
      fieldRefs(row, "outcome_refs").some((id) => outcomeIds.has(id)),
  );
  const acceptanceIds = new Set(
    acceptances.map((row) => textField(row, "id")).filter((id): id is string => !!id),
  );
  const values = control.value_assessments.filter((row) => {
    const outcomeRef = textField(row, "outcome_ref");
    const acceptanceRef = textField(row, "acceptance_ref");
    return (
      textField(row, "work_package_id") === workId &&
      row.work_generation === generation &&
      outcomeRef !== null &&
      outcomeIds.has(outcomeRef) &&
      (acceptanceRef === null || acceptanceIds.has(acceptanceRef))
    );
  });
  const valueIds = new Set(
    values.map((row) => textField(row, "id")).filter((id): id is string => !!id),
  );
  const valueSupport = control.value_assessment_support.filter((support) =>
    valueIds.has(support.assessment_id),
  );
  const evidence = control.condition_evidence.filter((row) =>
    textField(row, "work_ref") === workId && row.work_generation === generation,
  );
  return {
    resolutions,
    bindings,
    receipts,
    interopBindings,
    externalTasks,
    attempts,
    reconciliations,
    outcomes,
    acceptances,
    values,
    valueSupport,
    evidence,
  };
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
  const summary = `${trace.resolutions.length} resolution decisions · ${trace.bindings.length} execution bindings · ${trace.interopBindings.length} interop bindings · ${trace.receipts.length} harness receipts · ${trace.externalTasks.length} external tasks · ${trace.outcomes.length} outcomes · ${trace.acceptances.length} linked decisions · ${trace.values.length} value assessments`;
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
                  <small>Runtime version: {textField(receipt, "runtime_version") ?? "Not pinned"}</small>
                  <small>Runtime digest: {textField(receipt, "runtime_digest") ?? "Fixture / not pinned"}</small>
                  <small>Environment: {textField(receipt, "execution_environment_ref") ?? "Fixture / not pinned"}</small>
                </li>
              ))}
            </ul>
          )}
          <p>Harness receipts are executor evidence only; they never establish a business outcome or acceptance.</p>
        </section>
        <section>
          <strong>Governed interoperability bindings</strong>
          {trace.interopBindings.length === 0 ? (
            <p>No persisted MCP/A2A endpoint is pinned to this Work execution binding.</p>
          ) : (
            <ul>
              {trace.interopBindings.map((interop, index) => {
                const endpoint = interop.endpoint as Record<string, unknown> | undefined;
                return (
                  <li key={textField(interop, "execution_binding_ref") ?? index}>
                    <b>{String(endpoint?.protocol ?? "external").toUpperCase()} endpoint</b>
                    <small>Endpoint: {String(endpoint?.endpoint_ref ?? "Not pinned")}</small>
                    <small>Capability: {textField(interop, "capability_ref") ?? "Not pinned"}</small>
                  </li>
                );
              })}
            </ul>
          )}
        </section>
        <section>
          <strong>External protocol task evidence</strong>
          {trace.externalTasks.length === 0 ? (
            <p>No durable MCP/A2A task observation for this Work generation.</p>
          ) : (
            <ul>
              {trace.externalTasks.map((task, index) => {
                const snapshot = task.snapshot as Record<string, unknown> | undefined;
                const protocol = snapshot ? Object.keys(snapshot)[0] ?? "external" : "external";
                return (
                  <li key={textField(task, "id") ?? index}>
                    <b>{protocol.toUpperCase()} task observation</b>
                    <small>Binding: {textField(task, "execution_binding_ref") ?? "Not pinned"}</small>
                    <small>Executor state only — never Morn Outcome or Acceptance</small>
                  </li>
                );
              })}
            </ul>
          )}
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
          <strong>Outcome-linked value evidence</strong>
          {trace.values.length === 0 ? (
            <p>No value assessment is pinned to this Work generation and its observed outcomes.</p>
          ) : (
            <ul>
              {trace.values.map((assessment, index) => {
                const assessmentId = textField(assessment, "id");
                const evidenceClass = textField(assessment, "evidence_class") ?? "Unclassified";
                const acceptanceRef = textField(assessment, "acceptance_ref");
                const support = trace.valueSupport.find(
                  (candidate) => candidate.assessment_id === assessmentId,
                );
                const customerValidated = evidenceClass === "CustomerValidated";
                const currentTrust = support?.currently_supported
                  ? "proven"
                  : support?.current_real_site_state ?? "missing";
                return (
                  <li key={assessmentId ?? index}>
                    <b>{evidenceClass}</b>
                    <small>Outcome: {textField(assessment, "outcome_ref") ?? "Missing"}</small>
                    <small>
                      Acceptance: {acceptanceRef ?? "Not linked — cannot be CustomerValidated"}
                    </small>
                    <small>Value evidence refs: {fieldRefs(assessment, "evidence_refs").length}</small>
                    {customerValidated && (
                      <small>
                        Current RealSite support: {currentTrust}
                        {support?.current_real_site_claim_id
                          ? ` · ${support.current_real_site_claim_id}`
                          : ""}
                      </small>
                    )}
                  </li>
                );
              })}
            </ul>
          )}
          <p>
            CustomerValidated is historical evidence, not a permanent entitlement. Current RealSite
            support is re-evaluated from the latest evidence claim; revoked or externally blocked
            support remains visible without rewriting the original assessment.
          </p>
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
              <article className="work-focus-item" key={work.id} data-work-id={work.id}>
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

export function governedRealHarnessEnvironment(
  providerRef: string | undefined,
  status: V115Status | null,
): string | null {
  if (providerRef === "deepseek-harness" && status?.harness_runtime.dsh.mode === "real") {
    return status.harness_runtime.dsh.configured_execution_environment_ref;
  }
  if (providerRef === "pi" && status?.harness_runtime.pi.mode === "real") {
    return status.harness_runtime.pi.configured_execution_environment_ref;
  }
  return null;
}

export function governedRealHarnessRuntimeIdentity(
  providerRef: string | undefined,
  status: V115Status | null,
): string | null {
  if (providerRef === "deepseek-harness" && status?.harness_runtime.dsh.mode === "real") {
    const runtime = status.harness_runtime.dsh;
    return runtime.runtime_version && runtime.runtime_digest
      ? `deepseek-harness@${runtime.runtime_version}#${runtime.runtime_digest}`
      : null;
  }
  if (providerRef === "pi" && status?.harness_runtime.pi.mode === "real") {
    const runtime = status.harness_runtime.pi;
    return runtime.runtime_version && runtime.runtime_digest
      ? `pi@${runtime.runtime_version}#${runtime.runtime_digest}`
      : null;
  }
  return null;
}

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
  const realHarness = realDsh || realPi;
  const configuredHarnessEnvironment =
    governedRealHarnessEnvironment(selectedCapability?.manifest.provider_ref, status) ?? "";
  const configuredRuntimeIdentity =
    governedRealHarnessRuntimeIdentity(selectedCapability?.manifest.provider_ref, status);
  const eligibleEnvironments = (status?.execution_environment_attestations ?? []).filter(
    (environment) =>
      environment.active &&
      (!realHarness ||
        (environment.environment_ref === configuredHarnessEnvironment &&
          configuredRuntimeIdentity !== null &&
          environment.runtime_identities.includes(configuredRuntimeIdentity))),
  );
  const effectiveEnvironmentRef =
    environmentRef ||
    (realHarness && eligibleEnvironments.length === 1
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
    if (!effectiveWorkId || !effectiveCapabilityId) return;
    if (realHarness && !effectiveEnvironmentRef) return;
    setBusy(true);
    setMessage(null);
    try {
      const path = realHarness ? "/v115/work/bind-attested-e0" : "/v115/work/bind-e0";
      await apiPostJson(path, {
        work_id: effectiveWorkId,
        capability_manifest_id: effectiveCapabilityId,
        ...(realHarness ? { execution_environment_ref: effectiveEnvironmentRef } : {}),
      });
      setMessage(
        realHarness
          ? `Trusted environment and exact ${selectedCapability?.manifest.provider_ref ?? "provider"} runtime identity pinned. Execution has not started.`
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
      {realHarness && (
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
            Real {selectedCapability?.manifest.provider_ref ?? "Harness"} binding is allowed only
            when this fresh deployment attestation exactly matches the environment pinned by that
            provider's launch configuration and runtime distribution identity.
          </small>
        </label>
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
            (realHarness && !effectiveEnvironmentRef)
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
  observations,
  reload,
}: {
  control: V115ControlPlaneData;
  catalog: SourceOfTruthCatalog | null;
  observations: SourceObservationCatalog | null;
  reload: () => void;
}) {
  const [workId, setWorkId] = useState("");
  const [catalogId, setCatalogId] = useState("");
  const [sourceBindingId, setSourceBindingId] = useState("");
  const [observationAttestationId, setObservationAttestationId] = useState("");
  const [busy, setBusy] = useState(false);
  const [message, setMessage] = useState<string | null>(null);

  const works = control.work.filter(
    (work) => !["Accepted", "Rejected", "Cancelled"].includes(work.status.phase),
  );
  const effectiveWorkId = workId || works[0]?.id || "";
  const work = works.find((item) => item.id === effectiveWorkId);
  const catalogBindings = (catalog?.bindings ?? []).filter(
    (binding) => (binding.site_ref ?? null) === (work?.spec.site_ref ?? null),
  );
  const effectiveCatalogId =
    catalogId || (catalogBindings.length === 1 ? catalogBindings[0].id : "");

  const boundIds = new Set(
    control.condition_evidence
      .filter(
        (entry) =>
          textField(entry, "work_ref") === effectiveWorkId &&
          entry.work_generation === work?.generation &&
          textField(entry, "condition_type") === "SourceOfTruthBound" &&
          entry.satisfied === true,
      )
      .flatMap((entry) => fieldRefs(entry, "evidence_refs")),
  );
  const workBindings = control.source_of_truth_bindings.filter((binding) => {
    const id = textField(binding, "id");
    return id !== null && boundIds.has(id);
  });
  const effectiveSourceBindingId =
    sourceBindingId ||
    (workBindings.length === 1 ? textField(workBindings[0], "id") ?? "" : "");

  const trustedObservations = (observations?.observations ?? []).filter(
    (observation) =>
      observation.work_package_id === effectiveWorkId &&
      observation.work_generation === work?.generation &&
      observation.source_binding_id === effectiveSourceBindingId,
  );
  const effectiveObservationAttestationId =
    observationAttestationId ||
    (trustedObservations.length === 1 ? trustedObservations[0].attestation_id : "");
  const selectedObservation = trustedObservations.find(
    (observation) => observation.attestation_id === effectiveObservationAttestationId,
  );

  const attach = async () => {
    if (!effectiveWorkId || !effectiveCatalogId) return;
    setBusy(true);
    setMessage(null);
    try {
      await apiPostJson("/v115/work/bind-source-of-truth", {
        work_id: effectiveWorkId,
        catalog_binding_id: effectiveCatalogId,
      });
      setSourceBindingId("");
      setObservationAttestationId("");
      setMessage("Deployment-owned source authority attached to this Work generation.");
      reload();
    } catch (e) {
      setMessage((e as Error).message);
    } finally {
      setBusy(false);
    }
  };

  const observe = async () => {
    if (!effectiveWorkId || !effectiveObservationAttestationId) return;
    setBusy(true);
    setMessage(null);
    try {
      await apiPostJson("/v115/work/observe-outcome", {
        work_id: effectiveWorkId,
        observation_attestation_id: effectiveObservationAttestationId,
      });
      setObservationAttestationId("");
      setMessage(
        "Deployment-attested authoritative Outcome persisted. Independent review is still required.",
      );
      reload();
    } catch (e) {
      setMessage((e as Error).message);
    } finally {
      setBusy(false);
    }
  };

  if (works.length === 0) {
    return (
      <Card title="Authoritative outcome observation">
        <p>
          No non-terminal canonical Work is available. Executor output is never promoted
          implicitly into a business outcome.
        </p>
      </Card>
    );
  }

  return (
    <Card title="Authoritative outcome observation">
      <p>
        Attach a deployment-reviewed source of truth, then consume an observation attestation
        produced by that authoritative source. This surface cannot submit or edit world facts.
      </p>
      <div className="governed-outcome-grid">
        <label>
          Work
          <select
            value={effectiveWorkId}
            onChange={(event) => {
              setWorkId(event.target.value);
              setCatalogId("");
              setSourceBindingId("");
              setObservationAttestationId("");
            }}
          >
            {works.map((item) => (
              <option key={item.id} value={item.id}>
                {item.spec.goal} · {item.status.phase}
              </option>
            ))}
          </select>
        </label>
        <label>
          Deployment authority
          <select
            value={effectiveCatalogId}
            onChange={(event) => setCatalogId(event.target.value)}
            disabled={catalogBindings.length === 0}
          >
            {catalogBindings.length === 0 && (
              <option value="">No reviewed source for this site</option>
            )}
            {catalogBindings.length > 1 && !catalogId && (
              <option value="">Select reviewed authority</option>
            )}
            {catalogBindings.map((binding) => (
              <option key={binding.id} value={binding.id}>
                {binding.source_ref} · {binding.authority_kind}
              </option>
            ))}
          </select>
        </label>
      </div>
      <div className="page-actions">
        <button disabled={busy || !effectiveCatalogId} onClick={attach}>
          Attach authoritative source
        </button>
        <span className="muted">{workBindings.length} Work-scoped source binding(s)</span>
      </div>
      <div className="governed-outcome-grid">
        <label>
          Work-scoped source
          <select
            value={effectiveSourceBindingId}
            onChange={(event) => {
              setSourceBindingId(event.target.value);
              setObservationAttestationId("");
            }}
            disabled={workBindings.length === 0}
          >
            {workBindings.length === 0 && <option value="">Attach a source first</option>}
            {workBindings.length > 1 && !sourceBindingId && (
              <option value="">Select exact source binding</option>
            )}
            {workBindings.map((binding, index) => {
              const id = textField(binding, "id") ?? "";
              return (
                <option key={id || index} value={id}>
                  {textField(binding, "source_ref") ?? id}
                </option>
              );
            })}
          </select>
        </label>
        <label>
          Deployment-attested observation
          <select
            value={effectiveObservationAttestationId}
            onChange={(event) => setObservationAttestationId(event.target.value)}
            disabled={!effectiveSourceBindingId || trustedObservations.length === 0}
          >
            {trustedObservations.length === 0 && (
              <option value="">No unconsumed trusted observation</option>
            )}
            {trustedObservations.length > 1 && !observationAttestationId && (
              <option value="">Select trusted observation</option>
            )}
            {trustedObservations.map((observation) => (
              <option key={observation.attestation_id} value={observation.attestation_id}>
                {observation.objective} · {observation.fact_type}
              </option>
            ))}
          </select>
        </label>
      </div>
      {selectedObservation && (
        <div className="work-focus-empty">
          <strong>{selectedObservation.objective}</strong>
          <br />
          Source: {selectedObservation.source_ref} · fact: {selectedObservation.fact_type} ·
          witness refs: {selectedObservation.evidence_refs.length}
        </div>
      )}
      <div className="page-actions">
        <button
          disabled={busy || !effectiveObservationAttestationId}
          onClick={observe}
        >
          Persist attested authoritative Outcome
        </button>
      </div>
      <small className="muted">
        HTTP callers cannot provide observed_facts, source_ref or evidence_refs. Those fields come
        only from the deployment/connector attestation.
      </small>
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
  const [reviewAuthorizationId, setReviewAuthorizationId] = useState("");
  const [busy, setBusy] = useState(false);
  const [message, setMessage] = useState<string | null>(null);

  const candidates = control.work
    .filter((work) => ["Delivered", "Waiting"].includes(work.status.phase))
    .map((work) => ({
      work,
      outcomes: control.outcomes.filter(
        (outcome) =>
          textField(outcome, "work_package_id") === work.id &&
          outcome.work_generation === work.generation &&
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
      !reviewAuthorizationId.trim() ||
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
        review_authorization_id: reviewAuthorizationId.trim(),
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
        Out-of-band review authorization ID
        <input
          value={reviewAuthorizationId}
          onChange={(event) => setReviewAuthorizationId(event.target.value)}
          placeholder="deployment-issued exact Work/Outcome authorization"
        />
        <small className="muted">
          This ID is not listed by the API. It must come from the deployment's authenticated review
          workflow and is consumed exactly once.
        </small>
      </label>
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
            !reviewAuthorizationId.trim() ||
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

export function ValueAssessmentPanel({
  control,
  reload,
}: {
  control: V115ControlPlaneData;
  reload: () => void;
}) {
  const [workId, setWorkId] = useState("");
  const [outcomeId, setOutcomeId] = useState("");
  const [acceptanceId, setAcceptanceId] = useState("");
  const [evidenceClass, setEvidenceClass] = useState("observed-operational");
  const [evidenceRef, setEvidenceRef] = useState("");
  const [baselineRef, setBaselineRef] = useState("");
  const [kpisText, setKpisText] = useState("{}");
  const [busy, setBusy] = useState(false);
  const [message, setMessage] = useState<string | null>(null);

  const candidates = control.work
    .filter((work) => work.status.phase === "Accepted")
    .map((work) => {
      const outcomes = control.outcomes.filter(
        (outcome) =>
          textField(outcome, "work_package_id") === work.id &&
          outcome.work_generation === work.generation &&
          !!textField(outcome, "source_ref") &&
          fieldRefs(outcome, "evidence_refs").length > 0,
      );
      const outcomeIds = new Set(
        outcomes.map((outcome) => textField(outcome, "id")).filter((id): id is string => !!id),
      );
      const acceptances = control.acceptance_decisions.filter(
        (decision) =>
          textField(decision, "work_package_id") === work.id &&
          decision.work_generation === work.generation &&
          textField(decision, "disposition") === "Accept" &&
          fieldRefs(decision, "outcome_refs").some((id) => outcomeIds.has(id)),
      );
      return { work, outcomes, acceptances };
    })
    .filter((entry) => entry.outcomes.length > 0 && entry.acceptances.length > 0);

  const effectiveWorkId = workId || candidates[0]?.work.id || "";
  const selected = candidates.find((entry) => entry.work.id === effectiveWorkId);
  const effectiveOutcomeId = outcomeId || textField(selected?.outcomes[0] ?? {}, "id") || "";
  const compatibleAcceptances =
    selected?.acceptances.filter((decision) =>
      fieldRefs(decision, "outcome_refs").includes(effectiveOutcomeId),
    ) ?? [];
  const effectiveAcceptanceId =
    acceptanceId || textField(compatibleAcceptances[0] ?? {}, "id") || "";

  const assess = async () => {
    if (!effectiveWorkId || !effectiveOutcomeId || !effectiveAcceptanceId || !evidenceRef.trim()) {
      return;
    }
    let kpis: Record<string, number>;
    try {
      const parsed = JSON.parse(kpisText) as unknown;
      if (!parsed || Array.isArray(parsed) || typeof parsed !== "object") {
        throw new Error("KPI JSON must be an object");
      }
      kpis = Object.fromEntries(
        Object.entries(parsed as Record<string, unknown>).map(([key, value]) => {
          if (typeof value !== "number" || !Number.isFinite(value)) {
            throw new Error(`KPI ${key} must be a finite number`);
          }
          return [key, value];
        }),
      );
    } catch (e) {
      setMessage((e as Error).message);
      return;
    }

    setBusy(true);
    setMessage(null);
    try {
      const response = await apiPostJson<{
        value_assessment: { id: string; evidence_class: string };
        value_subject: string;
        customer_validated: boolean;
      }>("/v115/work/assess-value", {
        work_id: effectiveWorkId,
        outcome_id: effectiveOutcomeId,
        acceptance_id: effectiveAcceptanceId,
        evidence_class: evidenceClass,
        evidence_refs: [evidenceRef.trim()],
        baseline_ref: baselineRef.trim() || undefined,
        kpis,
      });
      setMessage(
        `Value assessment persisted: ${response.value_assessment.evidence_class}. Subject: ${response.value_subject}.`,
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
      <Card title="Accepted outcome value">
        <p>
          No independently Accepted, source-grounded Outcome is ready for value assessment.
          Executor completion, Delivered Work or a conditional review is insufficient.
        </p>
      </Card>
    );
  }

  return (
    <Card title="Accepted outcome value">
      <p>
        Measure value only against an exact accepted Outcome. CustomerValidated additionally
        requires a deployment-owned RealSite evidence claim; this form cannot self-assert one.
      </p>
      <div className="governed-execution-grid">
        <label>
          Accepted Work
          <select
            value={effectiveWorkId}
            onChange={(event) => {
              setWorkId(event.target.value);
              setOutcomeId("");
              setAcceptanceId("");
            }}
          >
            {candidates.map(({ work }) => (
              <option key={work.id} value={work.id}>
                {work.spec.goal} · generation {work.generation}
              </option>
            ))}
          </select>
        </label>
        <label>
          Source-grounded Outcome
          <select
            value={effectiveOutcomeId}
            onChange={(event) => {
              setOutcomeId(event.target.value);
              setAcceptanceId("");
            }}
          >
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
          Independent Acceptance
          <select
            value={effectiveAcceptanceId}
            onChange={(event) => setAcceptanceId(event.target.value)}
          >
            {compatibleAcceptances.map((decision, index) => {
              const id = textField(decision, "id") ?? `acceptance-${index}`;
              return <option key={id} value={textField(decision, "id") ?? ""}>{id}</option>;
            })}
          </select>
        </label>
        <label>
          Value evidence class
          <select value={evidenceClass} onChange={(event) => setEvidenceClass(event.target.value)}>
            <option value="fixture">Fixture</option>
            <option value="simulation">Simulation</option>
            <option value="shadow">Shadow</option>
            <option value="observed-operational">Observed operational</option>
            <option value="customer-validated">Customer validated (requires RealSite)</option>
          </select>
        </label>
      </div>
      <label className="governed-execution-prompt">
        Value evidence reference
        <input
          value={evidenceRef}
          onChange={(event) => setEvidenceRef(event.target.value)}
          placeholder="metric://report-or-signed-analysis"
        />
      </label>
      <label className="governed-execution-prompt">
        Baseline reference (optional)
        <input
          value={baselineRef}
          onChange={(event) => setBaselineRef(event.target.value)}
          placeholder="baseline://approved-reference"
        />
      </label>
      <label className="governed-execution-prompt">
        KPI JSON (optional)
        <textarea
          value={kpisText}
          onChange={(event) => setKpisText(event.target.value)}
          placeholder='{"human_minutes_saved": 12.5}'
        />
      </label>
      {evidenceClass === "customer-validated" && (
        <p className="work-focus-empty">
          CustomerValidated will be rejected unless deployment evidence contains a Proven real-site
          claim for this exact Work generation and Outcome.
        </p>
      )}
      <div className="page-actions">
        <button
          disabled={
            busy ||
            !effectiveWorkId ||
            !effectiveOutcomeId ||
            !effectiveAcceptanceId ||
            !evidenceRef.trim()
          }
          onClick={assess}
        >
          Persist value assessment
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
  const [sourceObservationCatalog, setSourceObservationCatalog] =
    useState<SourceObservationCatalog | null>(null);
  const [reviewerCatalog, setReviewerCatalog] = useState<AcceptanceReviewerCatalog | null>(null);

  const loadCanonical = useCallback(() => {
    setV115ControlError(null);
    apiGet<V115ControlPlaneData>("/v115/control-plane")
      .then(setV115Control)
      .catch((e: Error) => setV115ControlError(e.message))
      .finally(() => setV115ControlLoading(false));
  }, []);

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
    loadCanonical();
    apiGet<UiExtensionRegistry>("/v115/ui/extensions")
      .then(setUiExtensions)
      .catch(() => undefined);
    apiGet<{ capabilities: E0HarnessCapability[] }>("/v115/capabilities")
      .then((response) => setV115Capabilities(response.capabilities))
      .catch(() => setV115Capabilities([]));
    apiGet<SourceOfTruthCatalog>("/v115/source-of-truth/catalog")
      .then(setSourceTruthCatalog)
      .catch(() => setSourceTruthCatalog(null));
    apiGet<SourceObservationCatalog>("/v115/source-of-truth/observations")
      .then(setSourceObservationCatalog)
      .catch(() => setSourceObservationCatalog(null));
    apiGet<AcceptanceReviewerCatalog>("/v115/acceptance/reviewers")
      .then(setReviewerCatalog)
      .catch(() => setReviewerCatalog(null));
  }, [loadCanonical]);

  useEffect(() => {
    load();
    const refreshCanonical = () => {
      if (document.visibilityState === "visible") loadCanonical();
    };
    const interval = window.setInterval(loadCanonical, 2000);
    window.addEventListener("focus", loadCanonical);
    document.addEventListener("visibilitychange", refreshCanonical);
    return () => {
      window.clearInterval(interval);
      window.removeEventListener("focus", loadCanonical);
      document.removeEventListener("visibilitychange", refreshCanonical);
    };
  }, [load, loadCanonical]);

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
            <AuthoritativeOutcomePanel
              control={v115Control}
              catalog={sourceTruthCatalog}
              observations={sourceObservationCatalog}
              reload={load}
            />
            <OutcomeReviewPanel control={v115Control} reviewers={reviewerCatalog} reload={load} />
            <ValueAssessmentPanel control={v115Control} reload={load} />
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
          <AuthoritativeOutcomePanel
              control={v115Control}
              catalog={sourceTruthCatalog}
              observations={sourceObservationCatalog}
              reload={load}
            />
            <OutcomeReviewPanel control={v115Control} reviewers={reviewerCatalog} reload={load} />
            <ValueAssessmentPanel control={v115Control} reload={load} />
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
          .map((extension) => {
            const activeProfiles = v115Control?.work.map((work) => work.spec.profile_ref) ?? [];
            const enabled = uiExtensionEnabledForProfiles(
              extension.required_profile,
              activeProfiles,
            );
            return (
              <Card key={extension.id} title={`Extension · ${extension.title}`}>
                <KeyValue k="Domain" v={extension.domain} />
                <KeyValue k="Slot" v={extension.slot} />
                <KeyValue k="Renderer" v={extension.renderer} />
                <KeyValue
                  k="Required profile"
                  v={extension.required_profile ?? "Any active profile"}
                />
                <KeyValue
                  k="Profile gate"
                  v={<StatusPill value={enabled ? "Eligible" : "Blocked"} />}
                />
                <KeyValue
                  k="Safety model"
                  v={uiExtensions.arbitrary_remote_js ? "remote JS enabled" : "declarative / no arbitrary remote JS"}
                />
                <UiExtensionData extension={extension} enabled={enabled} />
                {extension.actions.length > 0 && (
                  <div className="page-actions" style={{ marginTop: 8 }}>
                    {extension.actions.map((action) => (
                      <button
                        key={action.id}
                        disabled={!enabled}
                        title={
                          action.authority_semantic
                            ? `Backend authority semantic: ${action.authority_semantic}`
                            : "Presentation action; backend enforcement remains authoritative"
                        }
                        onClick={() => runUiExtensionAction(action.method, action.endpoint)}
                      >
                        {action.label}
                      </button>
                    ))}
                  </div>
                )}
              </Card>
            );
          })}

        {v115Control && (
          <Card title="v11.5 Durable Work Truth">
            <KeyValue k="Work resources" v={v115Control.work.length} />
            <KeyValue k="Condition evidence" v={v115Control.condition_evidence.length} />
            <KeyValue k="Profile conformance attestations" v={v115Control.profile_conformance_attestations.length} />
            <KeyValue k="Bindings" v={v115Control.execution_bindings.length} />
            <KeyValue k="Execution manifests" v={v115Control.execution_manifests.length} />
            <KeyValue k="External task observations" v={v115Control.external_task_observations.length} />
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
