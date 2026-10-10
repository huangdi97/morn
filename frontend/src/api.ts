// API client for the Morn shared backend.

/** Vite proxies /api in the browser; packaged Tauri assets have no Vite proxy. */
export function apiBaseForOrigin(locationHref: string): string {
  try {
    const origin = new URL(locationHref);
    if (origin.protocol === "tauri:" || origin.hostname === "tauri.localhost") {
      return "http://127.0.0.1:8090/api";
    }
  } catch {
    return "/api";
  }
  return "/api";
}

const BASE = apiBaseForOrigin(typeof window === "undefined" ? "" : window.location.href);

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

export interface UiExtensionRegistry {
  extensions: Array<{
    id: string;
    domain: string;
    surface: string;
    slot: string;
    title: string;
    renderer: string;
    data_endpoint: string | null;
    actions: Array<{
      id: string;
      label: string;
      method: "GET" | "POST";
      endpoint: string;
      authority_semantic: string | null;
    }>;
    required_profile: string | null;
    priority: number;
  }>;
  execution_model: string;
  arbitrary_remote_js: boolean;
  business_truth: boolean;
}

export interface V115ControlPlaneData {
  condition_evidence: Array<Record<string, unknown>>;
  capability_resolutions: Array<Record<string, unknown>>;
  profile_conformance_attestations: Array<Record<string, unknown>>;
  work: Array<{
    id: string;
    generation: number;
    spec: {
      goal: string;
      profile_ref: string;
      required_conditions: string[];
      source_solution_ref?: string | null;
      site_ref?: string | null;
    };
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
  execution_events?: Array<Record<string, unknown>>;
  execution_receipts: Array<Record<string, unknown>>;
  executor_outcome_reconciliations: Array<Record<string, unknown>>;
  interop_bindings: Array<Record<string, unknown>>;
  external_task_observations: Array<Record<string, unknown>>;
  binding_migrations: Array<Record<string, unknown>>;
  durable_workflow_bindings: Array<Record<string, unknown>>;
  attempts: Array<Record<string, unknown>>;
  reconciliations: Array<Record<string, unknown>>;
  outcomes: Array<Record<string, unknown>>;
  acceptance_decisions: Array<Record<string, unknown>>;
  value_assessments: Array<Record<string, unknown>>;
  value_assessment_support: Array<{
    assessment_id: string;
    subject: string;
    requires_real_site: boolean;
    customer_value_attestation_ref: string | null;
    customer_value_attestation_active: boolean;
    currently_supported: boolean;
    current_real_site_state: "proven" | "blocked-external" | "revoked" | null;
    current_real_site_claim_id: string | null;
    current_real_site_evidence_refs: string[];
  }>;
  note: string;
}

export interface V115DiscoveryData {
  xregistry: Array<{
    group: string;
    resource_id: string;
    version: string;
    name: string;
    capability_kind: string;
    provider_ref: string;
    provides: string[];
    provenance: {
      adapter_version: string;
      canonical_manifest_ref: string;
      canonical_manifest_digest: string | null;
      metadata_class: string;
    };
  }>;
  a2a_agent_cards: Array<{
    name: string;
    card_url: string;
    protocol_version: string;
    skills: Array<{ id: string; name: string }>;
    provenance: { canonical_manifest_ref: string; metadata_class: string };
  }>;
  oasf: Array<{
    name: string;
    taxonomy_version: string;
    skills: string[];
    domains: string[];
    provenance: { canonical_manifest_ref: string; metadata_class: string };
  }>;
  rejected: Array<{ manifest_id: string; projection: string; reason: string }>;
  metadata_class: string;
  business_truth: boolean;
  invariant: string;
}

export interface AcceptanceReviewerCatalog {
  reviewers: Array<{
    principal_id: string;
    acting_roles: string[];
    evidence_refs: string[];
    observed_at: number | string | Record<string, unknown>;
    valid_until: number | string | Record<string, unknown> | null;
  }>;
  deployment_attested: boolean;
  caller_can_self_assert_identity: boolean;
  final_review_requires_exact_out_of_band_authorization: boolean;
  authorization_ids_are_listed: boolean;
}

export interface SourceObservationCatalog {
  observations: Array<{
    attestation_id: string;
    workspace_id: string;
    work_package_id: string;
    work_generation: number;
    source_binding_id: string;
    fact_type: string;
    objective: string;
    source_ref: string;
    observed_facts: Record<string, unknown>;
    evidence_refs: string[];
    observed_at: number | string | Record<string, unknown>;
    valid_until: number | string | Record<string, unknown> | null;
  }>;
  deployment_attested: boolean;
  caller_can_submit_world_facts: boolean;
  consumed_attestations_are_listed: boolean;
}

export interface SourceOfTruthCatalog {
  bindings: Array<{
    id: string;
    site_ref: string | null;
    source_ref: string;
    authority_kind: "SystemOfRecord" | "Sensor" | "HumanAuthority" | "ValidatedComputation";
    authoritative_fact_types: string[];
    key_mapping_ref: string;
    query_capability_ref: string;
    freshness_sla_ms: number | null;
    conflict_policy: string;
    version_ref: string;
  }>;
  deployment_owned: boolean;
  caller_can_create_authority: boolean;
  note: string;
}

export interface V115Status {
  provider_catalog: Array<{
    id: string;
    family: string;
    version: string;
    digest: string | null;
    endpoint_ref: string | null;
    protocols: string[];
    features: string[];
    status: string;
    evidence_refs: string[];
    observed_at: number | string | Record<string, unknown>;
    health_valid_until: number | string | Record<string, unknown> | null;
  }>;
  provider_observations: Array<{
    provider_id: string;
    previous_status: string;
    current_status: string;
    reason: string;
    evidence_refs: string[];
    observed_at: number | string | Record<string, unknown>;
    health_valid_until: number | string | Record<string, unknown> | null;
  }>;
  provider_status_semantics: {
    registered: string;
    healthy: string;
  };
  harness_runtime: {
    dsh: {
      mode: "fixture" | "real";
      health: string;
      configured_execution_environment_ref: string | null;
      profile_configuration_ref: string | null;
      runtime_version: string | null;
      runtime_digest: string | null;
      wire_server_version: string | null;
    };
    pi: {
      mode: "fixture" | "real";
      health: string;
      configured_execution_environment_ref: string | null;
      runtime_version: string | null;
      runtime_digest: string | null;
    };
  };
  execution_environment_attestations: Array<{
    environment_ref: string;
    provider: string;
    isolation: string;
    required_guarantees: string[];
    runtime: string | null;
    runtime_identities: string[];
    evidence_refs: string[];
    observed_at: number | string | Record<string, unknown>;
    valid_until: number | string | Record<string, unknown> | null;
    active: boolean;
  }>;
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
    site_admission_requires_verified_signature_and_provenance: boolean;
    caller_can_self_assert_verification: boolean;
    verification_evidence: Array<{
      subject_digest: string;
      verifier_ref: string;
      signature_verified: boolean;
      provenance_verified: boolean;
      evidence_refs: string[];
    }>;
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
  evidence_policy: {
    classes: string[];
    classes_are_categorical: boolean;
    no_implicit_promotion: boolean;
    claims: Array<{
      id: string;
      subject: string;
      class: string;
      state: string;
      evidence_refs: string[];
      issuer: string;
      reason: string;
      observed_at: number | string | Record<string, unknown>;
    }>;
  };
  release_readiness: {
    build_identity_ref: string | null;
    axes: Array<{
      id: string;
      subject: string;
      evidence_class: string;
      state:
        | "proven"
        | "blocked-external"
        | "revoked"
        | "missing-evidence"
        | "identity-mismatch"
        | "runtime-unhealthy";
      reason: string;
      evidence_refs: string[];
    }>;
    deployment_scopes: Array<{
      scope:
        | "local-reference"
        | "deepseek-read-only"
        | "pi-read-only"
        | "customer-read-only-deepseek"
        | "customer-read-only-pi"
        | "production-write-deepseek"
        | "production-write-pi";
      ready: boolean;
      required_axes: string[];
      blockers: string[];
    }>;
    semantics: {
      aggregate_ready: false;
      reason: string;
    };
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
  harness: {
    native: { provider: string; status: string };
    dsh: { provider: string; status: string };
    pi: { provider: string; mode: string; features: Record<string, boolean> };
  };
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