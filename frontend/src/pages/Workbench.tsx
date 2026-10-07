import { useCallback, useEffect, useState } from "react";
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
  V115Status,
  V115ControlPlaneData,
} from "../api";
import { Card, EmptyState, ErrorBox, KeyValue, Loading, StatusPill } from "../components/ui";

/** Domain-gated UI extension point: BioLab reference UI is only rendered when
 *  the backend advertises the biolab-reference domain pack (zero-domain builds
 *  advertise none, so the Core surfaces stay domain-free). */
export function biolabEnabled(domainPacks: string[]): boolean {
  return domainPacks.includes("biolab-reference");
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
    apiGet<V115ControlPlaneData>("/v115/control-plane")
      .then(setV115Control)
      .catch(() => undefined);
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
          {biolab && <button onClick={runE2e}>Run BioLab E2E</button>}
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

        {v115Control && (
          <Card title="v11.5 Durable Work Truth">
            <KeyValue k="Work resources" v={v115Control.work.length} />
            <KeyValue k="Bindings" v={v115Control.execution_bindings.length} />
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
            <KeyValue k="Production write" v={v115.factory_profile.production_write ? "enabled" : "not entered"} />
            <KeyValue k="Capability lifecycle" v={v115.capability_supply_chain.stages.join(" → ")} />
          </Card>
        )}
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
    </div>
  );
}
