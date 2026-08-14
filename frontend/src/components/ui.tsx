import { ReactNode } from "react";

export function Card({ title, children }: { title: string; children: ReactNode }) {
  return (
    <section className="card">
      <h3 className="card-title">{title}</h3>
      <div className="card-body">{children}</div>
    </section>
  );
}

export function StatusPill({ value }: { value: string }) {
  const cls = value.toLowerCase().replace(/[^a-z0-9]/g, "-");
  return <span className={`pill pill-${cls}`}>{value}</span>;
}

export function Loading() {
  return <div className="state-box">Loading…</div>;
}

export function ErrorBox({ message }: { message: string }) {
  return <div className="state-box state-error">Error: {message}</div>;
}

export function EmptyState({ label }: { label: string }) {
  return <div className="state-box state-empty">{label}</div>;
}

export function KeyValue({ k, v }: { k: string; v: ReactNode }) {
  return (
    <div className="kv">
      <span className="kv-key">{k}</span>
      <span className="kv-value">{v}</span>
    </div>
  );
}
