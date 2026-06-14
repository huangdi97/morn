import { useState, useEffect } from "react";

// @ts-ignore
const invoke = window.__TAURI__
  ? (window as any).__TAURI__.invoke || ((window as any).__TAURI__["@tauri-apps/api/core"]?.invoke)
  : async (cmd: string, args?: Record<string, unknown>) => {
      const res = await fetch(`/api/${cmd}`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify(args || {}),
      });
      return res.json();
    };

interface ComponentTypeDef {
  type_name: string;
  interfaces: string[];
  implements: string[];
  config_schema: Record<string, unknown>;
  author: string;
  version: string;
}

export function ComponentTypeManager() {
  const [types, setTypes] = useState<ComponentTypeDef[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState("");
  const [typeName, setTypeName] = useState("");
  const [interfaces, setInterfaces] = useState("");
  const [implements_, setImplements_] = useState("");
  const [configSchema, setConfigSchema] = useState("{}");
  const [author, setAuthor] = useState("");
  const [version, setVersion] = useState("0.1.0");
  const [message, setMessage] = useState("");

  const load = async () => {
    try {
      setLoading(true);
      const result: ComponentTypeDef[] = await invoke("list_component_types");
      setTypes(result);
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => { load(); }, []);

  const handleRegister = async () => {
    try {
      setMessage("");
      await invoke("register_component_type", {
        typeName,
        interfaces: interfaces.split(",").map((s) => s.trim()).filter(Boolean),
        implements: implements_.split(",").map((s) => s.trim()).filter(Boolean),
        configSchema: JSON.parse(configSchema),
        author: author || "user",
        version: version || "0.1.0",
      });
      setMessage(`Type '${typeName}' registered`);
      setTypeName("");
      setInterfaces("");
      setImplements_("");
      setConfigSchema("{}");
      await load();
    } catch (e) {
      setMessage(`Error: ${e}`);
    }
  };

  const handleUnregister = async (name: string) => {
    try {
      await invoke("unregister_component_type", { typeName: name });
      setMessage(`Type '${name}' removed`);
      await load();
    } catch (e) {
      setMessage(`Error: ${e}`);
    }
  };

  return (
    <div className="component-type-manager">
      <h2>Component Type Manager</h2>

      {error && <div className="error">{error}</div>}

      <div style={{ display: "flex", gap: "24px" }}>
        {/* List */}
        <div style={{ flex: 1 }}>
          <h3>Registered Types ({types.length})</h3>
          {loading ? (
            <p>Loading...</p>
          ) : (
            <table style={{ width: "100%", borderCollapse: "collapse" }}>
              <thead>
                <tr>
                  <th>Name</th>
                  <th>Interfaces</th>
                  <th>Implements</th>
                  <th>Version</th>
                  <th></th>
                </tr>
              </thead>
              <tbody>
                {types.map((t) => (
                  <tr key={t.type_name}>
                    <td>{t.type_name}</td>
                    <td>{t.interfaces.join(", ")}</td>
                    <td>{t.implements.join(", ")}</td>
                    <td>{t.version}</td>
                    <td>
                      {!["tool", "knowledge", "skill", "persona", "memory", "model", "agent", "pipeline"].includes(
                        t.type_name
                      ) && (
                        <button onClick={() => handleUnregister(t.type_name)}>Delete</button>
                      )}
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          )}
        </div>

        {/* Register Form */}
        <div style={{ width: "320px" }}>
          <h3>Register New Type</h3>
          <div style={{ display: "flex", flexDirection: "column", gap: "8px" }}>
            <input placeholder="Type name" value={typeName} onChange={(e) => setTypeName(e.target.value)} />
            <input placeholder="Interfaces (comma-separated)" value={interfaces} onChange={(e) => setInterfaces(e.target.value)} />
            <input placeholder="Implements (comma-separated)" value={implements_} onChange={(e) => setImplements_(e.target.value)} />
            <textarea
              placeholder='Config schema (JSON)'
              value={configSchema}
              onChange={(e) => setConfigSchema(e.target.value)}
              rows={3}
            />
            <input placeholder="Author (default: user)" value={author} onChange={(e) => setAuthor(e.target.value)} />
            <input placeholder="Version (default: 0.1.0)" value={version} onChange={(e) => setVersion(e.target.value)} />
            <button onClick={handleRegister}>Register</button>
          </div>
          {message && <p style={{ marginTop: "8px", fontSize: "13px" }}>{message}</p>}
        </div>
      </div>
    </div>
  );
}
