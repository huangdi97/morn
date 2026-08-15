import { useEffect, useState } from "react";
import { apiGet, HubV2Data, HubV3Data } from "../api";
import { Card, EmptyState, ErrorBox, Loading, StatusPill } from "../components/ui";

interface HubData {
  domain_packs: Array<{ id: string; name: string; trust: string }>;
  actor_templates: Array<{ id: string; name: string; trust: string }>;
  harness_templates: Array<{ id: string; name: string; trust: string }>;
  work_package_templates: Array<{ id: string; name: string; trust: string }>;
  workcell_blueprints: Array<{ id: string; name: string; trust: string }>;
  evaluation_packs: Array<{ id: string; name: string; trust: string }>;
  operational_object_types: Array<{ id: string; name: string; trust: string; lifecycle: string }>;
}

function AssetTable({ title, rows }: { title: string; rows: Array<{ id: string; name: string; trust: string }> }) {
  return (
    <Card title={title}>
      {rows.length === 0 ? (
        <EmptyState label="No assets" />
      ) : (
        <table>
          <thead>
            <tr>
              <th>id</th>
              <th>name</th>
              <th>trust</th>
            </tr>
          </thead>
          <tbody>
            {rows.map((r) => (
              <tr key={r.id}>
                <td>{r.id}</td>
                <td>{r.name}</td>
                <td>
                  <StatusPill value={r.trust} />
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      )}
    </Card>
  );
}

export default function Hub() {
  const [data, setData] = useState<HubData | null>(null);
  const [v2, setV2] = useState<HubV2Data | null>(null);
  const [v3, setV3] = useState<HubV3Data | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    apiGet<HubData>("/hub")
      .then(setData)
      .catch((e: Error) => setError(e.message));
    apiGet<HubV2Data>("/hub2")
      .then(setV2)
      .catch(() => undefined);
    apiGet<HubV3Data>("/hub3")
      .then(setV3)
      .catch(() => undefined);
  }, []);

  if (error) return <ErrorBox message={error} />;
  if (!data) return <Loading />;

  return (
    <div className="page">
      <header className="page-header">
        <h1>Hub — Registry</h1>
      </header>
      <div className="grid">
        {v2 && (
          <>
            <AssetTable title="Solution Templates (v0.2)" rows={v2.solution_templates} />
            <Card title="Evaluation Packs (v0.2)">
              <ul>
                {v2.evaluation_packs.map((e) => (
                  <li key={e.id}>{e.name}</li>
                ))}
              </ul>
            </Card>
            <Card title="Simulation Scenarios (v0.2)">
              <ul>
                {(v2.simulation_scenarios ?? []).map((s) => (
                  <li key={s}>{s}</li>
                ))}
              </ul>
            </Card>
            <AssetTable title="Work Capability Candidates (v0.2)" rows={v2.work_capability_candidates} />
            <Card title="Workflow Templates (v0.2)">
              <ul>
                {v2.workflow_templates.map((w) => (
                  <li key={w.id}>
                    {w.name} — signals: {w.signals.join(", ")}
                  </li>
                ))}
              </ul>
            </Card>
          </>
        )}
        {v3 && (
          <>
            <AssetTable title="Certified Work Capabilities (v0.3)" rows={v3.certified_capabilities.map((c) => ({ id: c.id, name: c.name, trust: c.status }))} />
            <Card title="Capability Releases (v0.3)">
              <ul>
                {v3.capability_releases.map((r) => (
                  <li key={r.id}>
                    release {r.version}
                  </li>
                ))}
              </ul>
            </Card>
            <Card title="Replacement Records (v0.3)">
              <ul>
                {v3.replacement_records.map((r) => (
                  <li key={r.id}>
                    {r.work} — {r.decision}
                  </li>
                ))}
              </ul>
            </Card>
          </>
        )}
        <AssetTable title="Domain Packs" rows={data.domain_packs} />
        <AssetTable title="Actor Templates" rows={data.actor_templates} />
        <AssetTable title="Harness Templates" rows={data.harness_templates} />
        <AssetTable title="WorkPackage Templates" rows={data.work_package_templates} />
        <AssetTable title="Workcell Blueprints" rows={data.workcell_blueprints} />
        <AssetTable title="Evaluation Packs" rows={data.evaluation_packs} />
        <AssetTable title="Operational Object Types" rows={data.operational_object_types} />
      </div>
    </div>
  );
}
