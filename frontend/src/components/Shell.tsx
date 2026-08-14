import { ReactNode } from "react";
import { NavLink } from "react-router-dom";

const links = [
  { to: "/workbench", label: "Workbench" },
  { to: "/studio", label: "Studio" },
  { to: "/console", label: "Console" },
  { to: "/hub", label: "Hub" },
];

export function Shell({ children }: { children: ReactNode }) {
  return (
    <div className="app-shell">
      <aside className="sidebar">
        <div className="brand">Morn</div>
        <nav>
          {links.map((l) => (
            <NavLink key={l.to} to={l.to} className={({ isActive }) => (isActive ? "nav-link active" : "nav-link")}>
              {l.label}
            </NavLink>
          ))}
        </nav>
      </aside>
      <main className="content">{children}</main>
    </div>
  );
}
