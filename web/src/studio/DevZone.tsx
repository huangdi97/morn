import { useState } from "react";

type PluginType = "theme" | "channel" | "ui_slot" | "protocol";

interface ExamplePlugin {
  name: string;
  type: PluginType;
  description: string;
  details: string;
}

const EXAMPLE_PLUGINS: ExamplePlugin[] = [
  {
    name: "Cyber Theme",
    type: "theme",
    description: "Custom UI theme with neon colors",
    details: "Manifest + theme.css + main.js — 3 files",
  },
  {
    name: "WeChat Channel",
    type: "channel",
    description: "Connect to WeChat for messaging",
    details: "Manifest + channel.js + main.js — 3 files",
  },
  {
    name: "Web Search Tool",
    type: "ui_slot",
    description: "Add web search to agent toolbox",
    details: "Manifest + panel.html + main.js — 3 files",
  },
  {
    name: "MCP Server Protocol",
    type: "protocol",
    description: "Add MCP server capabilities",
    details: "Manifest + protocol.json + main.js — 3 files",
  },
  {
    name: "Analytics Widget",
    type: "ui_slot",
    description: "Dashboard analytics panel",
    details: "Manifest + panel.html + main.js — 3 files",
  },
];

const PLUGIN_TYPE_LABELS: Record<PluginType, string> = {
  theme: "Theme",
  channel: "Channel",
  ui_slot: "UI Slot",
  protocol: "Protocol",
};

const PLUGIN_TYPE_COLORS: Record<PluginType, string> = {
  theme: "rgb(168, 130, 255)",
  channel: "rgb(34, 197, 94)",
  ui_slot: "rgb(99, 102, 241)",
  protocol: "rgb(255, 159, 67)",
};

function generateManifest(name: string, pluginType: PluginType, author: string): string {
  return JSON.stringify(
    {
      name: name || "my-plugin",
      version: "1.0.0",
      description: "",
      author: author || "",
      plugin_type: pluginType,
      entry: "main.js",
    },
    null,
    2
  );
}

export function DevZone() {
  const [pluginName, setPluginName] = useState("");
  const [pluginType, setPluginType] = useState<PluginType>("theme");
  const [author, setAuthor] = useState("");
  const [manifest, setManifest] = useState("");
  const [copied, setCopied] = useState(false);
  const [nlInput, setNlInput] = useState("");
  const [generating, setGenerating] = useState(false);
  const [generatedPath, setGeneratedPath] = useState("");
  const [generatedFiles, setGeneratedFiles] = useState<string[]>([]);

  const handleGenerateWithAI = async () => {
    setGenerating(true);
    setGeneratedPath("");
    setGeneratedFiles([]);
    try {
      const { invoke } = await import("@tauri-apps/api/core");
      await invoke<string>("generate_plugin_from_nl", { nl: nlInput });
      setGeneratedFiles(["manifest.json", "main.js"]);
    } catch (e) {
      console.error("AI generation failed", e);
      setGeneratedPath("Error: " + e);
    } finally {
      setGenerating(false);
    }
  };

  const handleInstall = async () => {
    try {
      const { invoke } = await import("@tauri-apps/api/core");
      await invoke("plugin_install", { path: generatedPath.replace("/manifest.json", "") });
    } catch (e) {
      console.error("Install failed", e);
    }
  };

  const handleGenerate = () => {
    const json = generateManifest(pluginName, pluginType, author);
    setManifest(json);
    setCopied(false);
  };

  const handleCopy = async () => {
    try {
      await navigator.clipboard.writeText(manifest);
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    } catch {
      // fallback
    }
  };

  const handleUseTemplate = (example: ExamplePlugin) => {
    setPluginName(example.name.replace(/\s+/g, "-").toLowerCase());
    setPluginType(example.type);
    setAuthor("");
  };

  return (
    <div style={{ padding: "24px", maxWidth: "900px" }}>
      <h2 style={{ color: "var(--text-primary)", marginBottom: "24px", fontSize: "20px", fontWeight: 600 }}>
        Developer Zone
      </h2>

      {/* Scaffold Plugin Section */}
      <div
        style={{
          background: "var(--bg-secondary)",
          border: "1px solid var(--border)",
          borderRadius: "8px",
          padding: "20px",
          marginBottom: "24px",
        }}
      >
        <h3 style={{ color: "var(--text-primary)", marginBottom: "16px", fontSize: "16px", fontWeight: 600 }}>
          Scaffold Plugin
        </h3>

        <div style={{ display: "flex", flexDirection: "column", gap: "12px" }}>
          <div>
            <label style={{ display: "block", color: "var(--text-secondary)", fontSize: "13px", marginBottom: "4px" }}>
              Plugin Name
            </label>
            <input
              value={pluginName}
              onChange={(e) => setPluginName(e.target.value)}
              placeholder="e.g. my-awesome-plugin"
              style={{
                width: "100%",
                padding: "8px 12px",
                borderRadius: "6px",
                border: "1px solid var(--border)",
                background: "var(--bg-tertiary)",
                color: "var(--text-primary)",
                fontSize: "14px",
                outline: "none",
              }}
            />
          </div>

          <div>
            <label style={{ display: "block", color: "var(--text-secondary)", fontSize: "13px", marginBottom: "4px" }}>
              Plugin Type
            </label>
            <select
              value={pluginType}
              onChange={(e) => setPluginType(e.target.value as PluginType)}
              style={{
                width: "100%",
                padding: "8px 12px",
                borderRadius: "6px",
                border: "1px solid var(--border)",
                background: "var(--bg-tertiary)",
                color: "var(--text-primary)",
                fontSize: "14px",
                outline: "none",
              }}
            >
              {(["theme", "channel", "ui_slot", "protocol"] as const).map((t) => (
                <option key={t} value={t}>{PLUGIN_TYPE_LABELS[t]}</option>
              ))}
            </select>
          </div>

          <div>
            <label style={{ display: "block", color: "var(--text-secondary)", fontSize: "13px", marginBottom: "4px" }}>
              Author
            </label>
            <input
              value={author}
              onChange={(e) => setAuthor(e.target.value)}
              placeholder="e.g. your-name"
              style={{
                width: "100%",
                padding: "8px 12px",
                borderRadius: "6px",
                border: "1px solid var(--border)",
                background: "var(--bg-tertiary)",
                color: "var(--text-primary)",
                fontSize: "14px",
                outline: "none",
              }}
            />
          </div>

          <button
            onClick={handleGenerate}
            style={{
              alignSelf: "flex-start",
              padding: "8px 20px",
              borderRadius: "6px",
              background: "var(--accent)",
              color: "#fff",
              border: "none",
              fontSize: "14px",
              fontWeight: 500,
              cursor: "pointer",
            }}
          >
            Generate
          </button>
        </div>

        {manifest && (
          <div
            style={{
              marginTop: "16px",
              background: "#0d1117",
              border: "1px solid var(--border)",
              borderRadius: "6px",
              overflow: "hidden",
            }}
          >
            <div
              style={{
                display: "flex",
                justifyContent: "space-between",
                alignItems: "center",
                padding: "8px 12px",
                borderBottom: "1px solid var(--border)",
              }}
            >
              <span style={{ color: "var(--text-secondary)", fontSize: "12px", fontFamily: "var(--font-mono)" }}>
                manifest.json
              </span>
              <button
                onClick={handleCopy}
                style={{
                  padding: "4px 12px",
                  borderRadius: "4px",
                  border: "1px solid var(--border)",
                  background: "var(--bg-tertiary)",
                  color: "var(--text-primary)",
                  fontSize: "12px",
                  cursor: "pointer",
                }}
              >
                {copied ? "Copied!" : "Copy"}
              </button>
            </div>
            <pre
              style={{
                margin: 0,
                padding: "12px",
                color: "#e6edf3",
                fontSize: "13px",
                fontFamily: "var(--font-mono)",
                lineHeight: 1.5,
                overflowX: "auto",
                whiteSpace: "pre",
              }}
            >
              {manifest}
            </pre>
          </div>
        )}
      </div>

      {/* AI Plugin Generator Section */}
      <div
        style={{
          background: "var(--bg-secondary)",
          border: "1px solid var(--border)",
          borderRadius: "8px",
          padding: "20px",
          marginBottom: "24px",
        }}
      >
        <h3 style={{ color: "var(--text-primary)", marginBottom: "16px", fontSize: "16px", fontWeight: 600 }}>
          AI Plugin Generator
        </h3>

        <div style={{ display: "flex", flexDirection: "column", gap: "12px" }}>
          <div>
            <label style={{ display: "block", color: "var(--text-secondary)", fontSize: "13px", marginBottom: "4px" }}>
              Describe your plugin in natural language...
            </label>
            <textarea
              value={nlInput}
              onChange={(e) => setNlInput(e.target.value)}
              placeholder='e.g. "A weather widget that shows the forecast for the current location" or "A dark theme plugin with purple accents"'
              rows={4}
              style={{
                width: "100%",
                padding: "8px 12px",
                borderRadius: "6px",
                border: "1px solid var(--border)",
                background: "var(--bg-tertiary)",
                color: "var(--text-primary)",
                fontSize: "14px",
                outline: "none",
                resize: "vertical",
                fontFamily: "inherit",
              }}
            />
          </div>

          <div style={{ display: "flex", gap: "8px", alignItems: "center" }}>
            <button
              onClick={handleGenerateWithAI}
              disabled={generating || !nlInput.trim()}
              style={{
                padding: "8px 20px",
                borderRadius: "6px",
                background: generating ? "var(--border)" : "var(--accent)",
                color: "#fff",
                border: "none",
                fontSize: "14px",
                fontWeight: 500,
                cursor: generating || !nlInput.trim() ? "not-allowed" : "pointer",
              }}
            >
              {generating ? "Generating..." : "Generate with AI"}
            </button>
          </div>

          {generatedPath && (
            <div
              style={{
                padding: "12px",
                borderRadius: "6px",
                background: generatedPath.startsWith("Error")
                  ? "rgba(239, 68, 68, 0.1)"
                  : "rgba(34, 197, 94, 0.1)",
                border: generatedPath.startsWith("Error")
                  ? "1px solid rgba(239, 68, 68, 0.3)"
                  : "1px solid rgba(34, 197, 94, 0.3)",
                color: generatedPath.startsWith("Error") ? "rgb(239, 68, 68)" : "rgb(34, 197, 94)",
                fontSize: "13px",
              }}
            >
              {generatedPath.startsWith("Error") ? (
                <p>{generatedPath}</p>
              ) : (
                <>
                  <p style={{ marginBottom: "8px" }}>
                    Plugin generated at: <code style={{ fontSize: "12px" }}>{generatedPath}</code>
                  </p>
                  {generatedFiles.length > 0 && (
                    <>
                      <p style={{ marginBottom: "4px", fontWeight: 500 }}>Generated files:</p>
                      <ul style={{ margin: 0, paddingLeft: "16px", listStyle: "disc" }}>
                        {generatedFiles.map((f) => (
                          <li key={f}>{f}</li>
                        ))}
                      </ul>
                    </>
                  )}
                  <button
                    onClick={handleInstall}
                    style={{
                      marginTop: "8px",
                      padding: "6px 16px",
                      borderRadius: "4px",
                      background: "rgb(34, 197, 94)",
                      color: "#fff",
                      border: "none",
                      fontSize: "13px",
                      cursor: "pointer",
                    }}
                  >
                    Install Plugin
                  </button>
                </>
              )}
            </div>
          )}
        </div>
      </div>

      {/* Example Plugins Section */}
      <div>
        <h3 style={{ color: "var(--text-primary)", marginBottom: "16px", fontSize: "16px", fontWeight: 600 }}>
          Example Plugins
        </h3>
        <div
          style={{
            padding: "12px 16px",
            background: "var(--bg-secondary)",
            border: "1px solid var(--border)",
            borderRadius: "8px",
            marginBottom: "16px",
            fontSize: "13px",
            color: "var(--text-secondary)",
            lineHeight: 1.5,
          }}
        >
          Get started fast with these ready-made plugin templates. Click <strong>Use Template</strong> to
          pre-fill the scaffold form above, or describe your own plugin in the AI generator.
          <br />
          Full example code is available at <code>src/themes/examples/</code> in the source tree.
        </div>
        {EXAMPLE_PLUGINS.length === 0 ? (
          <div style={{
            padding: "24px",
            textAlign: "center",
            color: "var(--text-secondary)",
            background: "var(--bg-secondary)",
            border: "1px dashed var(--border)",
            borderRadius: "8px",
            fontSize: "14px",
          }}>
            No example plugins loaded. Generate one using AI or create manually.
          </div>
        ) : (
        <div style={{ display: "grid", gridTemplateColumns: "repeat(auto-fill, minmax(240px, 1fr))", gap: "12px" }}>
          {EXAMPLE_PLUGINS.map((example) => (
            <div
              key={example.name}
              style={{
                background: "var(--bg-secondary)",
                border: "1px solid var(--border)",
                borderRadius: "8px",
                padding: "16px",
                transition: "all 0.15s ease",
              }}
            >
              <div style={{ fontWeight: 600, color: "var(--text-primary)", marginBottom: "6px", fontSize: "15px" }}>
                {example.name}
              </div>
              <span
                style={{
                  display: "inline-block",
                  fontSize: "11px",
                  padding: "2px 8px",
                  borderRadius: "4px",
                  background: `${PLUGIN_TYPE_COLORS[example.type]}20`,
                  color: PLUGIN_TYPE_COLORS[example.type],
                  marginBottom: "8px",
                }}
              >
                {PLUGIN_TYPE_LABELS[example.type]}
              </span>
              <div style={{ fontSize: "13px", color: "var(--text-secondary)", marginBottom: "4px", lineHeight: "1.4" }}>
                {example.description}
              </div>
              <div style={{ fontSize: "11px", color: "var(--text-secondary)", marginBottom: "12px", fontFamily: "var(--font-mono)" }}>
                {example.details}
              </div>
              <button
                onClick={() => handleUseTemplate(example)}
                style={{
                  width: "100%",
                  padding: "6px 12px",
                  borderRadius: "6px",
                  background: "var(--accent)",
                  color: "#fff",
                  border: "none",
                  cursor: "pointer",
                  fontSize: "13px",
                  fontWeight: 500,
                }}
              >
                Use Template
              </button>
            </div>
          ))}
        </div>
      )}
      </div>
    </div>
  );
}