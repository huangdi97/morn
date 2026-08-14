import { useEffect, useState } from "react";
import { apiGet } from "../api";
import { Card, EmptyState, ErrorBox, KeyValue, Loading } from "../components/ui";

interface Trace {
  seq: number;
  event_type: string;
  subject: string;
  summary: string;
  principal: string;
  payload_hash: string | null;
}

interface ConsoleData {
  identity: { workspace: string; owner: string };
  world_state: number;
  work: number;
  harness_health: { native: string; dsh: string };
  approvals_satisfied: string[];
  attention: number;
  policy: string;
  traces: Trace[];
  outcomes: Array<{ id: string; objective: string; acceptance_met: boolean; snapshots: string[] }>;
  evolution_promotions: Array<{ id: string; outcome: string; previous_version: string; new_version: string | null; rollback_ref: string | null }>;
  version_rollback: number;
}

export default function ConsolePage() {
  const [data, setData] = useState<ConsoleData | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    apiGet<ConsoleData>("/console")
      .then(setData)
      .catch((e: Error) => setError(e.message));
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
          <KeyValue k="DeepSeek Harness" v={data.harness_health.dsh} />
        </Card>
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
