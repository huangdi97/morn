// API client for the Morn shared backend.

const BASE = "/api";

export async function apiGet<T = unknown>(path: string): Promise<T> {
  const res = await fetch(`${BASE}${path}`);
  if (!res.ok) {
    const body = await res.json().catch(() => ({}));
    throw new Error((body as { error?: string }).error ?? `HTTP ${res.status}`);
  }
  return (await res.json()) as T;
}

export async function apiPost<T = unknown>(path: string): Promise<T> {
  const res = await fetch(`${BASE}${path}`, { method: "POST" });
  if (!res.ok) {
    const body = await res.json().catch(() => ({}));
    throw new Error((body as { error?: string }).error ?? `HTTP ${res.status}`);
  }
  return (await res.json()) as T;
}

export interface WorkbenchData {
  mission: { id: string; name: string; kind: string; status: string };
  world_objects: Array<{ id: string; type: string; state: Record<string, unknown>; version: number }>;
  work_packages: Array<{ id: string; objective: string; status: string; acceptance_spec_id: string | null }>;
  artifacts: { versions: number };
  attention: Array<{ id: string; kind: string; subject: string; priority: string }>;
  outcomes: Array<{ id: string; objective: string; acceptance_met: boolean }>;
  harness: { native: { provider: string; status: string }; dsh: { provider: string; status: string } };
  evolution_candidates: number;
  e2e_result: { all_ok: boolean; steps: Array<{ step: string; ok: boolean; detail: string }>; claim_id: string } | null;
}

export interface E2eRunResponse {
  ok: boolean;
  result: WorkbenchData["e2e_result"];
}

export async function apiPostJson<T = unknown>(path: string, body: unknown): Promise<T> {
  const res = await fetch(`${BASE}${path}`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(body),
  });
  if (!res.ok) {
    const b = await res.json().catch(() => ({}));
    throw new Error((b as { error?: string }).error ?? `HTTP ${res.status}`);
  }
  return (await res.json()) as T;
}

export interface CompilerRun {
  problem: { objective: string; domain: string; assumptions: Array<{ name: string; value: string }> };
  work_graph: { nodes: Array<{ id: string; name: string; nature: string; acceptance: string[] }>; edges: unknown[] };
  proposed: { work_packages: string[]; capability_gaps: Array<{ requirement: string; detail: string }>; risk_summary: string };
  validation: { passed: boolean; issues: Array<{ severity: string; message: string }> };
}

export interface DurableRun {
  run: { id: string; status: string; current_step: string | null; completed_steps: string[]; pending_steps: string[] };
}

export interface EvaluationOutcome {
  result: { decision: string; correctness: number; acceptance: boolean; policy: boolean; recovery: number; failures: string[] };
}

export interface ShadowOutcome {
  shadow_run: { comparison: { readiness: string; notes: string[] } };
}

export interface ReplayOutcome {
  replay_report: { reproduced: boolean; deviations: string[]; outcome: string };
}

export interface LoopAOutcome {
  loop_a: { all_ok: boolean; pi_approved: boolean; hypothesis: string; experiment_design_artifact_id: string };
}

export interface HubV2Data {
  solution_templates: Array<{ id: string; name: string; trust: string }>;
  domain_packs: { id: string } | null;
  evaluation_packs: Array<{ id: string; name: string }>;
  simulation_scenarios: string[] | null;
  work_capability_candidates: Array<{ id: string; name: string; status: string; trust: string }>;
  role_blueprints: string[] | null;
  workflow_templates: Array<{ id: string; name: string; signals: string[] }>;
}
