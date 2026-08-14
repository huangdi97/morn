import { useEffect, useState } from "react";
import { apiGet } from "../api";
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
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    apiGet<HubData>("/hub")
      .then(setData)
      .catch((e: Error) => setError(e.message));
  }, []);

  if (error) return <ErrorBox message={error} />;
  if (!data) return <Loading />;

  return (
    <div className="page">
      <header className="page-header">
        <h1>Hub — Registry</h1>
      </header>
      <div className="grid">
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
