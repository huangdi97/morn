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

export interface V115ControlPlaneData {
  work: Array<{
    id: string;
    generation: number;
    spec: { goal: string; profile_ref: string; required_conditions: string[] };
    status: {
      phase: string;
      observed_generation: number;
      conditions: Array<{ condition_type: string; status: string; reason: string }>;
      active_binding: string | null;
    };
  }>;
  source_of_truth_bindings: Array<Record<string, unknown>>;
  execution_bindings: Array<Record<string, unknown>>;
  execution_manifests: Array<Record<string, unknown>>;
  binding_migrations: Array<Record<string, unknown>>;
  attempts: Array<Record<string, unknown>>;
  reconciliations: Array<Record<string, unknown>>;
  outcomes: Array<Record<string, unknown>>;
  acceptance_decisions: Array<Record<string, unknown>>;
  value_assessments: Array<Record<string, unknown>>;
  note: string;
}

export interface V115Status {
  architecture: {
    definition: string;
    protocol_version: { major: number; minor: number; patch: number };
    semantic_slots: string[];
    semantic_invariants: Array<{ id: string; summary: string }>;
    control_model: string;
    composition_runtime: { name: string; role: string; reference_version: string; business_truth: boolean };
  };
  providers: {
    harness: Array<{ id: string; status: string }>;
    execution_environment: string[];
    authority: string;
  };
  profiles: Array<{ id: string; version: { major: number; minor: number; patch: number } }>;
  capability_supply_chain: {
    stages: string[];
    artifact_compilers: string[];
    qualification_is_not_admission: boolean;
  };
  factory_profile: {
    id: string;
    version: { major: number; minor: number; patch: number };
    minimum_isolation: string;
    required_execution_guarantees: string[];
    required_guarantees: string[];
    production_write: boolean;
    first_wedge: string;
  };
  claims: {
    local_engineering: string;
    real_dsh: string;
    real_pi: string;
    real_factory: string;
    production_write: string;
  };
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
  domain_packs: string[];
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

export interface CreatorDraftOutcome {
  draft: {
    name: string;
    profile_ref: string;
    site_ref: string | null;
    autonomy: string;
    problem: {
      objective: string;
      domain: string;
      constraints: Array<{ name: string; value: string; source: string }>;
    };
    work_graph: {
      nodes: Array<{ id: string; name: string; nature: string; acceptance: string[] }>;
      edges: unknown[];
    };
    proposed: {
      work_packages: string[];
      capability_gaps: Array<{ requirement: string; detail: string }>;
      unresolved_gaps: string[];
      risk_summary: string;
    };
    validation: { passed: boolean; issues: Array<{ severity: string; message: string }> };
    readiness_gates: string[];
    unresolved: string[];
    execution_started: boolean;
    canonicalization: string;
  };
  canonical_write: boolean;
  next: string[];
}

export interface OpenApiCompileOutcome {
  candidate: {
    record: {
      manifest: {
        id: string;
        name: string;
        provider_ref: string;
        provides: string[];
        kind: string;
      };
      stage: string;
      qualification_refs: string[];
      admitted_sites: string[];
    };
    report: {
      compiler: string;
      source_ref: string;
      discovered_operations: string[];
      warnings: string[];
    };
  };
  admission: string;
  next: string[];
}

export interface SolutionInstantiationOutcome {
  plan: {
    solution_package_ref: string;
    site_ref: string | null;
    work: {
      id: string;
      generation: number;
      spec: {
        goal: string;
        constraints: string[];
        required_conditions: string[];
        acceptance_ref: string | null;
        source_solution_ref: string | null;
        profile_ref: string;
      };
      status: {
        observed_generation: number;
        phase: string;
        active_binding: string | null;
      };
    };
    unresolved_gates: string[];
  };
  state: string;
  execution_started: boolean;
  note: string;
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
export interface FlywheelOutcome {
  patterns: Array<{ kind: string; affected_work: string; count: number }>;
  candidates: Array<{ id: string; candidate_type: string; affected_work: string; proposed_change: string; expected_benefit: string }>;
}

export interface DistillOutcome {
  candidate: { id: string; source_step: string; program_name: string; status: string };
  regression: { passed: boolean; program_matches_actor: number; long_tail_fallback_count: number; cost_reduction: number };
}

export interface CertifyOutcome {
  capability: { id: string; name: string; version: string; status: string };
}

export interface ManagedOutcome {
  run: { id: string; status: string; slo: { metric: string; target: string; acceptance_method: string } };
}

export interface ReplacementOutcome {
  comparison: { candidate_meets_critical: boolean; reasons: string[]; baseline: { quality: number; human_minutes: number; cost_estimate: number }; candidate: { quality: number; human_minutes: number; cost_estimate: number } };
}

export interface HubV3Data {
  certified_capabilities: Array<{ id: string; name: string; version: string; status: string }>;
  capability_releases: Array<{ id: string; version: string }>;
  replacement_records: Array<{ id: string; work: string; decision: string }>;
}
export interface OpintPredictOutcome {
  prediction: { id: string; predictor_id: string; target: string; value: number; interval_lo: number; interval_hi: number; confidence: number; context_match: boolean };
}

export interface OpintRegistry {
  predictors: Array<{ id: string; name: string; target: string; status: string; n: number; insufficient_data: boolean; predictions: number }>;
  episodes: number;
  snapshots: number;
  rollback_receipts: number;
}