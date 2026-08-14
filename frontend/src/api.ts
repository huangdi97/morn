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
