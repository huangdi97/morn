import { useCallback, useEffect, useState } from "react";
import {
  apiGet,
  apiPost,
  apiPostJson,
  DurableRun,
  EvaluationOutcome,
  LoopAOutcome,
  ReplayOutcome,
  ShadowOutcome,
  WorkbenchData,
} from "../api";
import { Card, EmptyState, ErrorBox, KeyValue, Loading, StatusPill } from "../components/ui";

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

  const load = useCallback(() => {
    setLoading(true);
    setError(null);
    apiGet<WorkbenchData>("/workbench")
      .then(setData)
      .catch((e: Error) => setError(e.message))
      .finally(() => setLoading(false));
  }, []);

  useEffect(load, [load]);

  const runE2e = async () => {
    setError(null);
    try {
      await apiPost("/biolab/run");
      load();
    } catch (e) {
      setError((e as Error).message);
    }
  };

  if (loading) return <Loading />;
  if (error) return <ErrorBox message={error} />;
  if (!data) return <EmptyState label="No workbench data" />;

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
          <button onClick={runE2e}>Run BioLab E2E</button>
        </div>
      </header>

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
      </div>

      <Card title="Durable Work Runtime (v0.2)">
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
    </div>
  );
}
