import { useEffect, useState } from "react";
import { apiGet } from "../api";
import { Card, EmptyState, ErrorBox, Loading } from "../components/ui";

interface StudioData {
  work_packages: Array<{
    id: string;
    objective: string;
    execution_mode: {
      nature: string[];
      executor: string[];
      rationale: string;
    } | null;
    allowed_actions: string[];
    prohibited_actions: string[];
  }>;
  object_types: Array<{ id: string; name: string; allowed_states: string[] }>;
  roles: string[];
  manifest_preview: Record<string, unknown>;
}

export default function Studio() {
  const [data, setData] = useState<StudioData | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    apiGet<StudioData>("/studio")
      .then(setData)
      .catch((e: Error) => setError(e.message));
  }, []);

  if (error) return <ErrorBox message={error} />;
  if (!data) return <Loading />;

  return (
    <div className="page">
      <header className="page-header">
        <h1>Studio</h1>
      </header>
      <div className="grid">
        <Card title="WorkPackage Builder (real backend records)">
          {data.work_packages.length === 0 ? (
            <EmptyState label="No work packages yet" />
          ) : (
            data.work_packages.map((wp) => (
              <div key={wp.id} className="builder-item">
                <strong>{wp.objective}</strong>
                <div>
                  mode: {wp.execution_mode ? wp.execution_mode.nature.join(", ") : "—"} /{" "}
                  {wp.execution_mode ? wp.execution_mode.executor.join(", ") : "—"}
                </div>
                <div>allowed: {wp.allowed_actions.join(", ") || "—"}</div>
                <div>prohibited: {wp.prohibited_actions.join(", ") || "—"}</div>
              </div>
            ))
          )}
        </Card>

        <Card title="Domain / World Builder">
          <ul>
            {data.object_types.map((t) => (
              <li key={t.id}>
                {t.name} — states: {t.allowed_states.join(", ")}
              </li>
            ))}
          </ul>
        </Card>

        <Card title="Role & Harness Builder">
          <ul>
            {data.roles.map((r) => (
              <li key={r}>{r}</li>
            ))}
          </ul>
        </Card>

        <Card title="Solution Manifest Preview">
          <pre>{JSON.stringify(data.manifest_preview, null, 2)}</pre>
        </Card>
      </div>
    </div>
  );
}
