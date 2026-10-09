import { useEffect, useState } from "react";
import { apiGet, HubV2Data, HubV3Data, V115Status } from "../api";
import { Card, EmptyState, ErrorBox, KeyValue, Loading, StatusPill } from "../components/ui";


interface SupplyCapability {
  manifest: {
    id: string;
    name: string;
    provider_ref: string;
    version: { major: number; minor: number; patch: number };
    digest: string | null;
    provenance: { source_ref: string; source_digest: string | null };
    authority: { allow: string[]; deny: string[]; maximum_effect: string };
    economics: { latency_p95_ms: number | null; estimated_cost_micros: number | null };
  };
  stage: string;
  qualification_refs: string[];
  release_refs: string[];
  admission_refs: unknown[];
}
interface SupplyQualification {
  id: string;
  manifest_id: string;
  status: string;
  context_of_use: string[];
  qualification_evidence: { known_failure_modes: string[] };
}
interface SupplyRelease {
  id: string;
  manifest_id: string;
  status: string;
  content_digest: string;
  signature_ref: string | null;
}
interface SupplyAdmission {
  id: string;
  manifest_id: string;
  status: string;
  site_ref: string;
  profile_ref: string;
}
interface CapabilitySupplyData {
  capabilities: SupplyCapability[];
  qualifications: SupplyQualification[];
  releases: SupplyRelease[];
  admissions: SupplyAdmission[];
  lifecycle_events: Array<{ id: string; manifest_id: string; event_type: string }>;
}

interface HubData {
  domain_packs: string[];
  actor_templates: Array<{ id: string; name: string; trust: string }>;
  harness_templates: Array<{ id: string; name: string; trust: string }>;
  composition_runtimes: Array<{ id: string; name: string; trust: string }>;
  capability_compilers: Array<{ id: string; name: string; trust: string }>;
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
  const [supply, setSupply] = useState<CapabilitySupplyData | null>(null);
  const [supplyError, setSupplyError] = useState<string | null>(null);
  const [providerStatus, setProviderStatus] = useState<V115Status | null>(null);
  const [providerError, setProviderError] = useState<string | null>(null);

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
    apiGet<CapabilitySupplyData>("/v115/capabilities")
      .then(setSupply)
      .catch((e: Error) => setSupplyError(e.message));
    apiGet<V115Status>("/v115/status")
      .then(setProviderStatus)
      .catch((e: Error) => setProviderError(e.message));
  }, []);

  if (error) return <ErrorBox message={error} />;
  if (!data) return <Loading />;

  return (
    <div className="page">
      <header className="page-header">
        <h1>Hub — Registry</h1>
      </header>
      <section className="hub-provider-fabric" aria-label="Provider fabric registry">
        <div className="hub-supply-head">
          <span className="work-focus-eyebrow">PROVIDER FABRIC · V11.5</span>
          <h2>Provider Fabric</h2>
          <p>
            Registered means configured or known, not healthy. Only evidence-backed Healthy providers
            are selectable, and harness health never grants authority beyond the pinned Work/Profile boundary.
          </p>
        </div>
        {providerError ? (
          <p role="alert">The provider registry is unavailable: {providerError}.</p>
        ) : !providerStatus ? (
          <p role="status">Loading provider registry…</p>
        ) : (
          <>
            <div className="hub-supply-grid">
              {providerStatus.provider_catalog.map((provider) => (
                <article className="hub-capability" key={provider.id}>
                  <header className="hub-capability-header">
                    <h3>{provider.id}</h3>
                    <StatusPill value={provider.status} />
                  </header>
                  <KeyValue k="Family" v={provider.family} />
                  <KeyValue k="Version" v={provider.version} />
                  <KeyValue k="Protocols" v={provider.protocols.join(", ") || "None declared"} />
                  <KeyValue k="Features" v={provider.features.join(", ") || "None declared"} />
                  <KeyValue k="Digest" v={provider.digest ?? "Not pinned"} />
                  <KeyValue
                    k="Health lease"
                    v={provider.health_valid_until ? String(provider.health_valid_until) : "No live health lease"}
                  />
                  <KeyValue k="Evidence" v={provider.evidence_refs.join(" · ") || "No evidence recorded"} />
                </article>
              ))}
            </div>
            {providerStatus.provider_observations.length > 0 && (
              <p className="work-focus-empty">
                Runtime configuration observations:{" "}
                {providerStatus.provider_observations
                  .map((item) => `${item.provider_id}: ${item.current_status} — ${item.reason}`)
                  .join(" · ")}
              </p>
            )}
          </>
        )}
      </section>
      <section className="hub-supply-chain" aria-label="Governed capability supply chain">
        <div className="hub-supply-head">
          <span className="work-focus-eyebrow">GOVERNED ASSETS · V11.5</span>
          <h2>v11.5 Capability Supply Chain</h2>
          <p>
            Declared → Observed → Qualified → Released → Site-admitted are separate
            decisions. A digest or harness registration alone does not authorize execution.
          </p>
        </div>
        {supplyError ? (
          <p role="alert">The canonical capability registry is unavailable: {supplyError}.</p>
        ) : !supply ? (
          <p role="status">Loading capability lifecycle records…</p>
        ) : supply.capabilities.length === 0 ? (
          <div className="work-focus-empty">
            No canonical capabilities recorded yet. Studio compilation only creates Declared
            candidates; qualification, release and site admission require separate evidence.
          </div>
        ) : (
          <div className="hub-supply-grid">
            {supply.capabilities.map((cap) => {
              const manifestId = cap.manifest.id;
              const qualifications = supply.qualifications.filter((q) => q.manifest_id === manifestId);
              const releases = supply.releases.filter((r) => r.manifest_id === manifestId);
              const admissions = supply.admissions.filter((a) => a.manifest_id === manifestId);
              const latestRelease = releases[releases.length - 1];
              const limitations = qualifications.flatMap((q) => q.qualification_evidence.known_failure_modes);
              return (
                <article className="hub-capability" key={manifestId}>
                  <header className="hub-capability-header">
                    <h3>{cap.manifest.name}</h3>
                    <StatusPill value={cap.stage} />
                  </header>
                  <KeyValue k="Manifest" v={manifestId} />
                  <KeyValue k="Provider" v={cap.manifest.provider_ref} />
                  <KeyValue k="Published digest" v={latestRelease?.content_digest ?? "No release"} />
                  <KeyValue k="Release" v={latestRelease ? <StatusPill value={latestRelease.status} /> : "Not released"} />
                  <KeyValue k="Signature reference" v={latestRelease?.signature_ref ?? "Not supplied / unverified"} />
                  <KeyValue k="Qualifications" v={qualifications.map((q) => q.status).join(", ") || "None recorded"} />
                  <KeyValue
                    k="Site / Profile admission"
                    v={admissions.length
                      ? admissions.map((a) => `${a.site_ref} / ${a.profile_ref}: ${a.status}`).join(" · ")
                      : "Not admitted"}
                  />
                  <KeyValue k="Authority ceiling" v={cap.manifest.authority.maximum_effect} />
                  <KeyValue k="Allowed actions" v={cap.manifest.authority.allow.join(", ") || "None declared"} />
                  <KeyValue k="Denied actions" v={cap.manifest.authority.deny.join(", ") || "None declared"} />
                  <KeyValue k="Estimated cost (micros)" v={cap.manifest.economics.estimated_cost_micros ?? "Not measured"} />
                  <KeyValue k="Latency p95 (ms)" v={cap.manifest.economics.latency_p95_ms ?? "Not measured"} />
                  <KeyValue k="Known failure modes" v={limitations.join("; ") || "Not provided"} />
                  <KeyValue k="Source provenance" v={cap.manifest.provenance.source_ref} />
                </article>
              );
            })}
          </div>
        )}
      </section>
      <div className="grid">
        <AssetTable
          title="Enabled Domain Packs"
          rows={data.domain_packs.map((d) => ({ id: d, name: d, trust: "Enabled" }))}
        />
      </div>

      <details className="hub-reference-details">
        <summary>Reference catalogs &amp; legacy registries</summary>
        <p className="hub-reference-note">
          These template and v0.2/v0.3 registries remain available for compatibility.
          Provider Fabric and the v11.5 Capability Supply Chain above are the governed source of truth.
        </p>
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
        <AssetTable title="Actor Templates" rows={data.actor_templates} />
        <AssetTable title="Harness Templates" rows={data.harness_templates} />
        <AssetTable title="Composition Runtimes" rows={data.composition_runtimes} />
        <AssetTable title="Capability Compilers" rows={data.capability_compilers} />
        <AssetTable title="WorkPackage Templates" rows={data.work_package_templates} />
        <AssetTable title="Workcell Blueprints" rows={data.workcell_blueprints} />
        <AssetTable title="Evaluation Packs" rows={data.evaluation_packs} />
        <AssetTable title="Operational Object Types" rows={data.operational_object_types} />
        </div>
      </details>
    </div>
  );
}
