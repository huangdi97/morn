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
}

interface ConsoleData {
  identity: { workspace: string; owner: string };
  world_state: number;
  work: number;
  harness_health: { native: string; dsh: HarnessHealth };
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
        </Card>
        {v115Control && (
          <Card title="v11.5 Persisted Control Records">
            <KeyValue k="Work" v={v115Control.work.length} />
            <KeyValue k="Source-of-truth bindings" v={v115Control.source_of_truth_bindings.length} />
            <KeyValue k="Execution bindings" v={v115Control.execution_bindings.length} />
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
              <KeyValue k="Execution environments" v={v115.providers.execution_environment.join(", ")} />
              <KeyValue k="Factory guarantees" v={v115.factory_profile.required_guarantees.length} />
              <KeyValue k="Real DSH" v={v115.claims.real_dsh} />
              <KeyValue k="Real Pi" v={v115.claims.real_pi} />
              <KeyValue k="Real factory" v={v115.claims.real_factory} />
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
