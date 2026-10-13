import { useCallback, useEffect, useState } from "react";

import {
  apiGet,
  apiPostJson,
  V115ControlPlaneData,
  V115Status,
} from "../api";
import { Card, Loading, StatusPill } from "./ui";

type E0HarnessCapability = {
  manifest: {
    id: string;
    name: string;
    provider_ref: string;
    authority: { maximum_effect: string };
  };
  stage: string;
};

type AcpLifecycleSession = {
  session_id: string;
  workspace_id: string;
  work_package_id: string;
  work_generation: number;
  execution_binding_ref: string;
  execution_environment_ref: string;
  scope_ref: string;
  route_ref: string;
  model_provider: string;
  model: string;
  reasoning_effort: string | null;
  runtime_version: string;
  runtime_digest: string;
  wire_agent_version: string;
  state: "active" | "faulted" | "closed" | "transport-closed";
  prompt_count: number;
  last_stop_reason: string | null;
};

type AcpLifecycleCatalog = {
  configured: boolean;
  execution_environment_ref?: string | null;
  profile_configuration_ref?: string | null;
  runtime_version?: string | null;
  runtime_digest?: string | null;
  route_ref?: string | null;
  sessions: Array<{
    session: AcpLifecycleSession;
    cancel_handle_available: boolean;
    work_bound_control: boolean;
  }>;
};

function textField(record: Record<string, unknown>, field: string): string | null {
  return typeof record[field] === "string" ? (record[field] as string) : null;
}

function fieldRefs(record: Record<string, unknown>, field: string): string[] {
  const refs = record[field];
  return Array.isArray(refs) ? refs.filter((value): value is string => typeof value === "string") : [];
}

function executionAllowed(phase: string | null | undefined): boolean {
  return ["Ready", "Running", "Waiting"].includes(phase ?? "");
}

export function DshAcpLifecyclePanel({
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
  const [catalog, setCatalog] = useState<AcpLifecycleCatalog | null>(null);
  const [catalogError, setCatalogError] = useState<string | null>(null);
  const [workId, setWorkId] = useState("");
  const [capabilityId, setCapabilityId] = useState("");
  const [bindingId, setBindingId] = useState("");
  const [environmentRef, setEnvironmentRef] = useState("");
  const [sessionId, setSessionId] = useState("");
  const [prompt, setPrompt] = useState(
    "Inspect the bound Work context. Return lifecycle-control evidence only; do not claim business outcome or acceptance.",
  );
  const [promptBusy, setPromptBusy] = useState(false);
  const [controlBusy, setControlBusy] = useState(false);
  const [message, setMessage] = useState<string | null>(null);

  const refreshSessions = useCallback(() => {
    apiGet<AcpLifecycleCatalog>("/v115/work/acp-lifecycle/sessions")
      .then((response) => {
        setCatalog(response);
        setCatalogError(null);
      })
      .catch((error: Error) => {
        setCatalog(null);
        setCatalogError(error.message);
      });
  }, []);

  useEffect(() => {
    refreshSessions();
  }, [refreshSessions]);

  const works = control.work.filter(
    (work) => !["Accepted", "Rejected", "Cancelled"].includes(work.status.phase),
  );
  const effectiveWorkId = workId || works[0]?.id || "";
  const selectedWork = works.find((work) => work.id === effectiveWorkId);
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
      capability.manifest.provider_ref === "deepseek-harness" &&
      capability.manifest.authority.maximum_effect === "E0LifecycleReversible" &&
      ["Qualified", "Admitted"].includes(capability.stage) &&
      resolvedCapabilityRefs.has(capability.manifest.id),
  );
  const effectiveCapabilityId = capabilityId || eligibleCapabilities[0]?.manifest.id || "";

  const acpBindings = control.execution_bindings.filter(
    (binding) =>
      textField(binding, "work_id") === effectiveWorkId &&
      binding.work_generation === selectedWork?.generation &&
      textField(binding, "provider_ref") === "deepseek-harness" &&
      textField(binding, "runtime_ref")?.startsWith("dsh-acp-route:") === true &&
      (!catalog?.route_ref || textField(binding, "runtime_ref") === catalog.route_ref),
  );
  const effectiveBindingId =
    bindingId || (acpBindings.length === 1 ? textField(acpBindings[0], "id") ?? "" : "");

  const expectedRuntimeIdentity =
    catalog?.runtime_version && catalog.runtime_digest
      ? `deepseek-harness@${catalog.runtime_version}#${catalog.runtime_digest}`
      : null;
  const eligibleEnvironments = (status?.execution_environment_attestations ?? []).filter(
    (environment) =>
      environment.active &&
      environment.environment_ref === catalog?.execution_environment_ref &&
      expectedRuntimeIdentity !== null &&
      environment.runtime_identities.includes(expectedRuntimeIdentity) &&
      (!catalog.profile_configuration_ref ||
        environment.runtime_identities.includes(catalog.profile_configuration_ref)),
  );
  const effectiveEnvironmentRef =
    environmentRef ||
    (eligibleEnvironments.length === 1 ? eligibleEnvironments[0].environment_ref : "");

  const governedSessions = (catalog?.sessions ?? []).filter((entry) => entry.work_bound_control);
  const effectiveSessionId =
    sessionId ||
    (governedSessions.length === 1 ? governedSessions[0].session.session_id : "");
  const selectedSession = governedSessions.find(
    (entry) => entry.session.session_id === effectiveSessionId,
  );
  const currentSessionMatchesWork =
    selectedSession !== undefined &&
    control.work.some(
      (work) =>
        work.id === selectedSession.session.work_package_id &&
        work.generation === selectedSession.session.work_generation &&
        !["Accepted", "Rejected", "Cancelled"].includes(work.status.phase),
    );

  const bindLifecycle = async () => {
    if (
      !effectiveWorkId ||
      !effectiveCapabilityId ||
      !effectiveEnvironmentRef ||
      selectedWork?.status.phase !== "Ready"
    ) {
      return;
    }
    setControlBusy(true);
    setMessage(null);
    try {
      const response = await apiPostJson<{ binding: { id: string }; transport: string }>(
        "/v115/work/bind-attested-e0",
        {
          work_id: effectiveWorkId,
          capability_manifest_id: effectiveCapabilityId,
          execution_environment_ref: effectiveEnvironmentRef,
          transport: "acp-lifecycle",
        },
      );
      setBindingId(response.binding.id);
      setMessage("Immutable ACP lifecycle binding persisted. No session or executor success exists yet.");
      reload();
    } catch (error) {
      setMessage((error as Error).message);
    } finally {
      setControlBusy(false);
    }
  };

  const openSession = async () => {
    if (!effectiveWorkId || !effectiveBindingId) return;
    setControlBusy(true);
    setMessage(null);
    try {
      const response = await apiPostJson<{ session: AcpLifecycleSession }>(
        "/v115/work/acp-lifecycle/open",
        { work_id: effectiveWorkId, binding_id: effectiveBindingId },
      );
      setSessionId(response.session.session_id);
      setMessage("ACP lifecycle session opened under the exact Work/binding/scope route.");
      refreshSessions();
    } catch (error) {
      setMessage((error as Error).message);
    } finally {
      setControlBusy(false);
    }
  };

  const promptSession = async () => {
    if (!selectedSession || !prompt.trim() || !currentSessionMatchesWork) return;
    setPromptBusy(true);
    setMessage(null);
    try {
      const response = await apiPostJson<{
        prompt: { stop_reason: string; assistant_text: string };
        session: AcpLifecycleSession;
      }>("/v115/work/acp-lifecycle/prompt", {
        session_id: selectedSession.session.session_id,
        work_id: selectedSession.session.work_package_id,
        binding_id: selectedSession.session.execution_binding_ref,
        input: prompt.trim(),
      });
      setMessage(
        `ACP settled: ${response.prompt.stop_reason}. Visible assistant output: ${response.prompt.assistant_text || "(none)"}. No business outcome was inferred.`,
      );
      refreshSessions();
    } catch (error) {
      setMessage((error as Error).message);
      refreshSessions();
    } finally {
      setPromptBusy(false);
    }
  };

  const sessionControl = async (action: "cancel" | "close" | "resume" | "release") => {
    if (!selectedSession) return;
    setControlBusy(true);
    setMessage(null);
    try {
      await apiPostJson<Record<string, unknown>>(`/v115/work/acp-lifecycle/${action}`, {
        session_id: selectedSession.session.session_id,
        work_id: selectedSession.session.work_package_id,
        binding_id: selectedSession.session.execution_binding_ref,
      });
      if (action === "cancel") {
        setMessage(
          "Cancellation requested out-of-band. This is not rollback proof; prompt settlement remains authoritative for lifecycle state.",
        );
      } else if (action === "release") {
        setMessage("Lifecycle scope released permanently; this session can no longer resume.");
        setSessionId("");
      } else {
        setMessage(`ACP lifecycle ${action} completed without creating Work Outcome or Acceptance.`);
      }
      refreshSessions();
    } catch (error) {
      setMessage((error as Error).message);
      refreshSessions();
    } finally {
      setControlBusy(false);
    }
  };

  return (
    <Card title="DeepSeek ACP lifecycle control">
      <p>
        ACP is the out-of-band session-control plane for DSH. It is bound to an immutable Work
        generation, E0 capability, execution environment and provider route. ACP prompt text and
        lifecycle settlement are never promoted into business Outcome or Acceptance.
      </p>
      {catalogError && <p role="alert">{catalogError}</p>}
      {!catalog ? (
        <Loading />
      ) : !catalog.configured ? (
        <p className="work-focus-empty">
          ACP lifecycle is disabled. Enable it explicitly in deployment configuration; SDK E0
          execution remains independent.
        </p>
      ) : (
        <>
          <div className="acp-identity-strip">
            <span>Environment <b>{catalog.execution_environment_ref ?? "not pinned"}</b></span>
            <span>Runtime <b>{catalog.runtime_version ?? "?"}</b></span>
            <span>Route <b>{catalog.route_ref ? "pinned" : "missing"}</b></span>
          </div>

          <div className="governed-execution-grid">
            <label>
              Current Work
              <select
                value={effectiveWorkId}
                onChange={(event) => {
                  setWorkId(event.target.value);
                  setCapabilityId("");
                  setBindingId("");
                  setEnvironmentRef("");
                }}
              >
                {works.map((work) => (
                  <option key={work.id} value={work.id}>
                    {work.spec.goal} · {work.status.phase}
                  </option>
                ))}
              </select>
            </label>
            <label>
              DSH E0 capability
              <select
                value={effectiveCapabilityId}
                onChange={(event) => setCapabilityId(event.target.value)}
                disabled={eligibleCapabilities.length === 0}
              >
                {eligibleCapabilities.length === 0 ? (
                  <option value="">No resolved qualified DSH E0 capability</option>
                ) : (
                  eligibleCapabilities.map((capability) => (
                    <option key={capability.manifest.id} value={capability.manifest.id}>
                      {capability.manifest.name}
                    </option>
                  ))
                )}
              </select>
            </label>
          </div>

          <label className="governed-execution-environment">
            ACP execution environment
            <select
              value={effectiveEnvironmentRef}
              onChange={(event) => setEnvironmentRef(event.target.value)}
              disabled={eligibleEnvironments.length === 0}
            >
              {eligibleEnvironments.length === 0 ? (
                <option value="">No active attestation matching runtime/profile identity</option>
              ) : eligibleEnvironments.length > 1 && !environmentRef ? (
                <>
                  <option value="">Select exact environment</option>
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
          </label>

          <div className="page-actions">
            <button
              onClick={bindLifecycle}
              disabled={
                controlBusy ||
                selectedWork?.status.phase !== "Ready" ||
                !effectiveCapabilityId ||
                !effectiveEnvironmentRef
              }
            >
              Persist ACP lifecycle binding
            </button>
            <span className="muted">{acpBindings.length} lifecycle binding(s) for this generation</span>
          </div>

          <label className="governed-execution-binding">
            ACP lifecycle binding
            <select
              value={effectiveBindingId}
              onChange={(event) => setBindingId(event.target.value)}
              disabled={acpBindings.length === 0}
            >
              {acpBindings.length === 0 ? (
                <option value="">Persist an ACP lifecycle binding first</option>
              ) : acpBindings.length > 1 && !bindingId ? (
                <>
                  <option value="">Select exact ACP binding</option>
                  {acpBindings.map((binding, index) => {
                    const id = textField(binding, "id") ?? "";
                    return <option key={id || index} value={id}>{id || "Unknown binding"}</option>;
                  })}
                </>
              ) : (
                acpBindings.map((binding, index) => {
                  const id = textField(binding, "id") ?? "";
                  return <option key={id || index} value={id}>{id || "Unknown binding"}</option>;
                })
              )}
            </select>
          </label>

          <div className="page-actions">
            <button
              onClick={openSession}
              disabled={
                controlBusy ||
                !effectiveBindingId ||
                !executionAllowed(selectedWork?.status.phase)
              }
            >
              Open Work-bound ACP session
            </button>
          </div>

          <label className="governed-execution-binding">
            Governed ACP session
            <select
              value={effectiveSessionId}
              onChange={(event) => setSessionId(event.target.value)}
              disabled={governedSessions.length === 0}
            >
              {governedSessions.length === 0 ? (
                <option value="">No Work-bound ACP session</option>
              ) : governedSessions.length > 1 && !sessionId ? (
                <>
                  <option value="">Select exact session</option>
                  {governedSessions.map((entry) => (
                    <option key={entry.session.session_id} value={entry.session.session_id}>
                      {entry.session.work_package_id} · g{entry.session.work_generation} · {entry.session.state}
                    </option>
                  ))}
                </>
              ) : (
                governedSessions.map((entry) => (
                  <option key={entry.session.session_id} value={entry.session.session_id}>
                    {entry.session.work_package_id} · g{entry.session.work_generation} · {entry.session.state}
                  </option>
                ))
              )}
            </select>
          </label>

          {selectedSession && (
            <div className="acp-session-state">
              <StatusPill value={selectedSession.session.state} />
              <span>Prompts: {selectedSession.session.prompt_count}</span>
              <span>Last stop: {selectedSession.session.last_stop_reason ?? "none"}</span>
              <span>
                Cancel handle: {selectedSession.cancel_handle_available ? "reachable" : "not available"}
              </span>
            </div>
          )}

          <label className="governed-execution-prompt">
            ACP prompt
            <textarea value={prompt} onChange={(event) => setPrompt(event.target.value)} />
          </label>

          <div className="page-actions acp-control-actions">
            <button
              onClick={promptSession}
              disabled={
                promptBusy ||
                !selectedSession ||
                selectedSession.session.state !== "active" ||
                !currentSessionMatchesWork ||
                !prompt.trim()
              }
            >
              {promptBusy ? "Prompt running…" : "Run ACP prompt"}
            </button>
            <button
              onClick={() => sessionControl("cancel")}
              disabled={
                controlBusy ||
                !selectedSession?.cancel_handle_available ||
                !selectedSession ||
                !["active", "faulted"].includes(selectedSession.session.state)
              }
            >
              Request cancel
            </button>
            <button
              onClick={() => sessionControl("close")}
              disabled={
                controlBusy ||
                !selectedSession ||
                !["active", "faulted"].includes(selectedSession.session.state)
              }
            >
              Close
            </button>
            <button
              onClick={() => sessionControl("resume")}
              disabled={
                controlBusy ||
                !selectedSession ||
                selectedSession.session.state !== "closed" ||
                !currentSessionMatchesWork
              }
            >
              Resume exact session
            </button>
            <button onClick={() => sessionControl("release")} disabled={controlBusy || !selectedSession}>
              Release scope
            </button>
          </div>

          {!currentSessionMatchesWork && selectedSession && (
            <p className="work-focus-alert" role="status">
              This session belongs to a historical Work generation. New prompts/resume are blocked,
              but cancel/close/release remain available for safe cleanup.
            </p>
          )}
          {message && <p role="status" className="work-focus-empty">{message}</p>}
        </>
      )}
    </Card>
  );
}
