import { useEffect, useState } from "react";
import { apiGet, OpintRegistry, V115ControlPlaneData, V115Status } from "../api";
import { Card, EmptyState, ErrorBox, KeyValue, Loading, StatusPill } from "../components/ui";

interface Trace {
  seq: number;
  event_type: string;
  subject: string;
  summary: string;
  principal: string;
  payload_hash: string | null;
}

interface HarnessHealth {
  provider: string;
  mode: string;
  features: {
    interrupt: boolean;
    resume: boolean;
    session_close: boolean;
    durable_events: boolean;
    multi_session: boolean;
  };
  effect_ceiling: string;
  credential_boundary: string;
  runtime_health: {
    state: string;
    settled_turns: number;
    reason: string;
    evidence_refs: string[];
  };
  runtime_version: string | null;
  runtime_digest: string | null;
  profile_configuration_ref?: string | null;
  route_ref: string | null;
  wire_server_version?: string | null;
  execution_environment_ref: string | null;
  tool_mediation_required: boolean;
}

interface ConsoleData {
  identity: { workspace: string; owner: string };
  world_state: number;
  work: number;
  harness_health: { native: string; dsh: HarnessHealth; pi: HarnessHealth };
  approvals_satisfied: string[];
  attention: number;
  policy: string;
  traces: Trace[];
  outcomes: Array<{ id: string; objective: string; acceptance_met: boolean; snapshots: string[] }>;
  evolution_promotions: Array<{ id: string; outcome: string; previous_version: string; new_version: string | null; rollback_ref: string | null }>;
  version_rollback: number;
}

function fmtVersion(v: unknown): string {
  if (typeof v === "string") return v;
  if (v && typeof v === "object") {
    const o = v as { major?: number; minor?: number; patch?: number };
    return `${o.major ?? 0}.${o.minor ?? 0}.${o.patch ?? 0}`;
  }
  return String(v);
}

function readinessTone(value: string): "ready" | "blocked" | "neutral" {
  const normalized = value.toLowerCase();
  if (normalized === "proven" || normalized.includes("pass")) return "ready";
  if (
    normalized.includes("blocked") ||
    normalized.includes("revoked") ||
    normalized.includes("missing") ||
    normalized.includes("unhealthy") ||
    normalized.includes("not authorized")
  ) {
    return "blocked";
  }
  return "neutral";
}

function readinessLabel(id: string): string {
  switch (id) {
    case "architecture-baseline":
      return "Architecture baseline";
    case "local-reference-slice":
      return "Local reference slice";
    case "ci-conformance":
      return "CI conformance";
    case "deepseek-live-runtime":
      return "DeepSeek live runtime";
    case "pi-live-runtime":
      return "Pi live runtime";
    case "customer-real-site":
      return "Customer / factory site";
    case "production-write":
      return "Production write";
    default:
      return id;
  }
}

export default function ConsolePage() {
  const [data, setData] = useState<ConsoleData | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [durableRuns, setDurableRuns] = useState<Array<{ id: string; status: string }>>([]);
  const [attention, setAttention] = useState<Array<{ kind: string; subject: string }>>([]);
  const [certified, setCertified] = useState<Array<{ name: string; version: string; status: string }>>([]);
  const [managedRuns, setManagedRuns] = useState<Array<{ id: string; status: string }>>([]);
  const [replacementRecords, setReplacementRecords] = useState<Array<{ work: string; decision: string }>>([]);
  const [opint, setOpint] = useState<OpintRegistry | null>(null);
  const [v115, setV115] = useState<V115Status | null>(null);
  const [v115Control, setV115Control] = useState<V115ControlPlaneData | null>(null);

  useEffect(() => {
    apiGet<ConsoleData>("/console")
      .then(setData)
      .catch((e: Error) => setError(e.message));
    apiGet<{ runs: Array<{ id: string; status: string }> }>("/durable/runs")
      .then((r) => setDurableRuns(r.runs))
      .catch(() => undefined);
    apiGet<{ attention: Array<{ kind: string; subject: string }> }>("/durable/attention")
      .then((r) => setAttention(r.attention))
      .catch(() => undefined);
    apiGet<{ capabilities: Array<{ name: string; version: string; status: string }> }>("/certify/list")
      .then((r) => setCertified(r.capabilities))
      .catch(() => undefined);
    apiGet<{ runs: Array<{ id: string; status: string }> }>("/managed/runs")
      .then((r) => setManagedRuns(r.runs))
      .catch(() => undefined);
    apiGet<{ records: Array<{ work: string; decision: string }> }>("/replacement/records")
      .then((r) => setReplacementRecords(r.records))
      .catch(() => undefined);
    apiGet<OpintRegistry>("/opint/registry")
      .then(setOpint)
      .catch(() => undefined);
    apiGet<V115Status>("/v115/status")
      .then(setV115)
      .catch(() => undefined);
    apiGet<V115ControlPlaneData>("/v115/control-plane")
      .then(setV115Control)
      .catch(() => undefined);
  }, []);

  if (error) return <ErrorBox message={error} />;
  if (!data) return <Loading />;

  return (
    <div className="page">
      <header className="page-header">
        <h1>Console</h1>
      </header>
      {v115 && (
        <section className="release-readiness" data-testid="release-readiness" aria-label="Release readiness">
          <div className="release-readiness-heading">
            <div>
              <span className="eyebrow">Morn v11.5 · evidence-aware release posture</span>
              <h2>Release readiness</h2>
            </div>
            <p>
              Local engineering can be green while live providers, site evidence, customer acceptance
              or production write remain independently blocked.
            </p>
          </div>
          <div className="release-readiness-grid">
            {v115.release_readiness.axes.map((axis) => (
              <article
                className={`readiness-item readiness-${readinessTone(axis.state)}`}
                key={axis.id}
              >
                <span>{readinessLabel(axis.id)}</span>
                <strong>{axis.state}</strong>
                <small className="readiness-reason">{axis.reason}</small>
                <small className="readiness-evidence">
                  {axis.evidence_refs.length
                    ? `Evidence: ${axis.evidence_refs.join(" · ")}`
                    : "No active evidence references"}
                </small>
              </article>
            ))}
            <article className="readiness-item readiness-neutral">
              <span>Attested environments</span>
              <strong>{v115.execution_environment_attestations.filter((item) => item.active).length}</strong>
              <small className="readiness-reason">
                Environment attestation does not itself prove a live provider, customer outcome or production authorization.
              </small>
            </article>
            <article className="readiness-item readiness-neutral">
              <span>Verified supply-chain digests</span>
              <strong>{v115.capability_supply_chain.verification_evidence.length}</strong>
              <small className="readiness-reason">
                Supply-chain verification proves artifact provenance/signature only; it does not grant execution authority.
              </small>
            </article>
          </div>
          <p className="release-readiness-footnote">{v115.release_readiness.semantics.reason}</p>
        </section>
      )}
      <div className="grid">
        <Card title="Identity / Registry">
          <KeyValue k="Workspace" v={data.identity.workspace} />
          <KeyValue k="Owner" v={data.identity.owner} />
          <KeyValue k="Policy" v={data.policy} />
        </Card>
        <Card title="World & Work">
          <KeyValue k="World objects" v={data.world_state} />
          <KeyValue k="Work packages" v={data.work} />
          <KeyValue k="Approvals satisfied" v={data.approvals_satisfied.join(", ") || "none"} />
          <KeyValue k="Open attention" v={data.attention} />
        </Card>
        <Card title="Harness / Runtime Health">
          <KeyValue k="Native" v={data.harness_health.native} />
          <KeyValue k="DeepSeek Harness" v={data.harness_health.dsh.provider} />
          <KeyValue k="DSH mode" v={data.harness_health.dsh.mode} />
          <KeyValue k="DSH effect ceiling" v={data.harness_health.dsh.effect_ceiling} />
          <KeyValue k="DSH credentials" v={data.harness_health.dsh.credential_boundary} />
          <KeyValue k="DSH runtime health" v={data.harness_health.dsh.runtime_health.state} />
          <KeyValue k="DSH distribution version" v={data.harness_health.dsh.runtime_version ?? "not live/pinned"} />
          <KeyValue k="DSH distribution digest" v={data.harness_health.dsh.runtime_digest ?? "not pinned"} />
          <KeyValue
            k="DSH profile configuration"
            v={data.harness_health.dsh.profile_configuration_ref ?? "fixture / not pinned"}
          />
          <KeyValue k="DSH bound route" v={data.harness_health.dsh.route_ref ?? "fixture / not pinned"} />
          <KeyValue k="DSH SDK wire version" v={data.harness_health.dsh.wire_server_version ?? "not initialized"} />
          <KeyValue k="DSH execution environment" v={data.harness_health.dsh.execution_environment_ref ?? "fixture / not configured"} />
          <KeyValue
            k="DSH tool mediation"
            v={data.harness_health.dsh.tool_mediation_required ? "attested precondition required" : "fixture/reference"}
          />
          <KeyValue k="DSH settled live turns" v={data.harness_health.dsh.runtime_health.settled_turns} />
          <KeyValue k="DSH health reason" v={data.harness_health.dsh.runtime_health.reason} />
          <KeyValue
            k="DSH health evidence"
            v={data.harness_health.dsh.runtime_health.evidence_refs.join(", ") || "none"}
          />
          <KeyValue
            k="DSH lifecycle"
            v={[
              data.harness_health.dsh.features.interrupt && "interrupt",
              data.harness_health.dsh.features.resume && "resume",
              data.harness_health.dsh.features.session_close && "session-close",
              data.harness_health.dsh.features.durable_events && "durable-events",
              data.harness_health.dsh.features.multi_session && "multi-session",
            ]
              .filter(Boolean)
              .join(", ") || "no optional lifecycle features"}
          />
          <KeyValue k="Pi Harness" v={data.harness_health.pi.provider} />
          <KeyValue k="Pi mode" v={data.harness_health.pi.mode} />
          <KeyValue k="Pi effect ceiling" v={data.harness_health.pi.effect_ceiling} />
          <KeyValue k="Pi credentials" v={data.harness_health.pi.credential_boundary} />
          <KeyValue k="Pi runtime health" v={data.harness_health.pi.runtime_health.state} />
          <KeyValue k="Pi distribution version" v={data.harness_health.pi.runtime_version ?? "not live/pinned"} />
          <KeyValue k="Pi distribution digest" v={data.harness_health.pi.runtime_digest ?? "not pinned"} />
          <KeyValue k="Pi bound route" v={data.harness_health.pi.route_ref ?? "fixture / not pinned"} />
          <KeyValue k="Pi execution environment" v={data.harness_health.pi.execution_environment_ref ?? "fixture / not configured"} />
          <KeyValue
            k="Pi tool mediation"
            v={data.harness_health.pi.tool_mediation_required ? "attested precondition required" : "fixture/reference"}
          />
          <KeyValue k="Pi settled live turns" v={data.harness_health.pi.runtime_health.settled_turns} />
          <KeyValue k="Pi health reason" v={data.harness_health.pi.runtime_health.reason} />
          <KeyValue
            k="Pi health evidence"
            v={data.harness_health.pi.runtime_health.evidence_refs.join(", ") || "none"}
          />
          <KeyValue
            k="Pi lifecycle"
            v={[
              data.harness_health.pi.features.interrupt && "interrupt",
              data.harness_health.pi.features.resume && "resume",
              data.harness_health.pi.features.session_close && "session-close",
              data.harness_health.pi.features.durable_events && "durable-events",
              data.harness_health.pi.features.multi_session && "multi-session",
            ]
              .filter(Boolean)
              .join(", ") || "no optional lifecycle features"}
          />
        </Card>
        {v115Control && (
          <Card title="v11.5 Persisted Control Records">
            <KeyValue k="Work" v={v115Control.work.length} />
            <KeyValue k="Source-of-truth bindings" v={v115Control.source_of_truth_bindings.length} />
            <KeyValue k="Execution bindings" v={v115Control.execution_bindings.length} />
            <KeyValue k="Execution receipts" v={v115Control.execution_receipts.length} />
            <KeyValue k="Interop endpoint bindings" v={v115Control.interop_bindings.length} />
            <KeyValue k="External task observations" v={v115Control.external_task_observations.length} />
            <KeyValue k="Durable workflow bindings" v={v115Control.durable_workflow_bindings.length} />
            <KeyValue k="Attempts" v={v115Control.attempts.length} />
            <KeyValue k="Reconciliations" v={v115Control.reconciliations.length} />
            <KeyValue k="Outcomes" v={v115Control.outcomes.length} />
            <KeyValue k="Acceptance decisions" v={v115Control.acceptance_decisions.length} />
            <KeyValue k="Value assessments" v={v115Control.value_assessments.length} />
          </Card>
        )}
        {v115 && (
          <>
            <Card title="v11.5 Control / Trust / Composition">
              <KeyValue k="Semantic slots" v={v115.architecture.semantic_slots.join(", ")} />
              <KeyValue k="Authority" v={v115.providers.authority} />
              <KeyValue k="Execution class vocabulary" v={v115.providers.execution_environment.join(", ")} />
              <KeyValue
                k="Active attested environments"
                v={v115.execution_environment_attestations.filter((item) => item.active).length}
              />
              <KeyValue k="Factory guarantees" v={v115.factory_profile.required_guarantees.length} />
              <KeyValue k="Real DSH" v={v115.claims.real_dsh} />
              <KeyValue k="Real Pi" v={v115.claims.real_pi} />
              <KeyValue k="Real factory" v={v115.claims.real_factory} />
            </Card>
            <Card title="Capability Supply-Chain Trust">
              <KeyValue
                k="Site admission policy"
                v={
                  v115.capability_supply_chain.site_admission_requires_verified_signature_and_provenance
                    ? "verified signature + provenance required"
                    : "not enforced"
                }
              />
              <KeyValue
                k="Caller self-assertion"
                v={v115.capability_supply_chain.caller_can_self_assert_verification ? "allowed" : "forbidden"}
              />
              <KeyValue
                k="Deployment-verified digests"
                v={v115.capability_supply_chain.verification_evidence.length}
              />
              {v115.capability_supply_chain.verification_evidence.map((evidence) => (
                <div key={evidence.subject_digest} className="kv">
                  <span className="kv-key">{evidence.subject_digest}</span>
                  <span className="kv-value">
                    signature={evidence.signature_verified ? "verified" : "missing"} ·
                    provenance={evidence.provenance_verified ? "verified" : "missing"} ·
                    {evidence.verifier_ref}
                  </span>
                </div>
              ))}
            </Card>
            <Card title="Evidence Class / Non-Claim Discipline">
              <KeyValue k="Evidence classes (categorical)" v={v115.evidence_policy.classes.join(", ")} />
              <KeyValue
                k="Implicit promotion"
                v={v115.evidence_policy.no_implicit_promotion ? "forbidden" : "allowed"}
              />
              {v115.evidence_policy.claims.map((claim) => (
                <div key={claim.id} className="kv">
                  <span className="kv-key">{claim.subject}</span>
                  <span className="kv-value">
                    {claim.class} / {claim.state} — {claim.reason}
                  </span>
                </div>
              ))}
            </Card>
          </>
        )}
        <details className="console-reference-details">
          <summary>Reference analytics &amp; legacy assurance views</summary>
          <p className="console-reference-note">
            These v0.2–v0.4 projections remain available for compatibility and diagnostics.
            Canonical v11.5 Work, Provider health, evidence and control records above remain authoritative.
          </p>
          <div className="grid">
        <Card title="Outcomes / Metrics">
          {data.outcomes.length === 0 ? (
            <EmptyState label="No outcomes yet" />
          ) : (
            data.outcomes.map((o) => (
              <div key={o.id} className="kv">
                <span className="kv-key">{o.objective}</span>
                <span className="kv-value">
                  {o.acceptance_met ? "accepted" : "pending"} (snapshots: {o.snapshots.length})
                </span>
              </div>
            ))
          )}
        </Card>
        <Card title="Evolution Promotion Decisions">
          {data.evolution_promotions.length === 0 ? (
            <EmptyState label="No promotions yet" />
          ) : (
            data.evolution_promotions.map((p) => (
              <div key={p.id} className="kv">
                <span className="kv-key">{p.outcome}</span>
                <span className="kv-value">
                  {p.previous_version} → {p.new_version ?? "—"} (rollback: {p.rollback_ref ?? "none"})
                </span>
              </div>
            ))
          )}
          <KeyValue k="Version rollbacks recorded" v={data.version_rollback} />
        </Card>
        <Card title="Durable Runs (v0.2)">
          {durableRuns.length === 0 ? (
            <EmptyState label="No durable runs" />
          ) : (
            <ul>
              {durableRuns.map((r) => (
                <li key={r.id}>
                  {r.id} — <StatusPill value={r.status} />
                </li>
              ))}
            </ul>
          )}
          <KeyValue k="Attention items" v={attention.length} />
          {attention.map((a) => (
            <div key={a.subject} className="kv">
              <span className="kv-key">{a.kind}</span>
              <span className="kv-value">{a.subject}</span>
            </div>
          ))}
        </Card>
        <Card title="Delegation & Representation">
          <KeyValue k="Delegation model" v="scope + expiry + retained accountability" />
          <KeyValue k="Representation" v="allow / deny / revoke enforced" />
          <KeyValue k="Accountability" v="judged -> delegated -> verified -> approved -> executed -> retained" />
        </Card>
        <Card title="Certification (v0.3)">
          {certified.length === 0 ? (
            <EmptyState label="No certified capabilities yet" />
          ) : (
            <ul>
              {certified.map((cap) => (
                <li key={`${cap.name}-${fmtVersion(cap.version)}`}>
                  {cap.name}@{fmtVersion(cap.version)} — <StatusPill value={cap.status} />
                </li>
              ))}
            </ul>
          )}
        </Card>
        <Card title="Managed Deliveries (v0.3)">
          {managedRuns.length === 0 ? (
            <EmptyState label="No managed deliveries" />
          ) : (
            <ul>
              {managedRuns.map((r) => (
                <li key={r.id}>
                  {r.id} — <StatusPill value={r.status} />
                </li>
              ))}
            </ul>
          )}
        </Card>
        <Card title="Replacement Records (v0.3)">
          {replacementRecords.length === 0 ? (
            <EmptyState label="No replacement records" />
          ) : (
            <ul>
              {replacementRecords.map((r, idx) => (
                <li key={`${r.work}-${idx}`}>
                  {r.work} — {r.decision}
                </li>
              ))}
            </ul>
          )}
        </Card>
        <Card title="Predictor Registry & Persistence (v0.4)">
          {opint ? (
            <>
              <KeyValue k="Episodes" v={opint.episodes} />
              <KeyValue k="Snapshots" v={opint.snapshots} />
              <KeyValue k="Rollback receipts" v={opint.rollback_receipts} />
              <ul>
                {opint.predictors.map((p) => (
                  <li key={p.id}>
                    {p.target} — n={p.n} <StatusPill value={p.insufficient_data ? "insufficient-data" : p.status} />
                  </li>
                ))}
              </ul>
            </>
          ) : (
            <EmptyState label="No predictor data yet" />
          )}
        </Card>
          </div>
        </details>

        <Card title="Trace / Errors (ledger)">
          {data.traces.length === 0 ? (
            <EmptyState label="No ledger entries" />
          ) : (
            <table>
              <thead>
                <tr>
                  <th>seq</th>
                  <th>event</th>
                  <th>subject</th>
                  <th>summary</th>
                </tr>
              </thead>
              <tbody>
                {data.traces.map((t) => (
                  <tr key={t.seq}>
                    <td>{t.seq}</td>
                    <td>{t.event_type}</td>
                    <td>{t.subject}</td>
                    <td>{t.summary}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          )}
        </Card>
      </div>
    </div>
  );
}
