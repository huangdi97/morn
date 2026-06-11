import { getApiConfig } from "./Settings";

declare global {
  interface Window {
    __TAURI__?: unknown;
  }
}

const isTauri = typeof window.__TAURI__ !== "undefined";

function getBaseUrl(): string {
  const config = getApiConfig();
  return config.serverUrl.replace(/\/+$/, "");
}

function getApiHeaders(): Record<string, string> {
  const config = getApiConfig();
  const headers: Record<string, string> = {
    "Content-Type": "application/json",
  };
  if (config.apiKey) {
    headers["X-API-Key"] = config.apiKey;
  }
  return headers;
}

function isRemote(): boolean {
  return getApiConfig().mode === "remote";
}

export const api = {
  async sendMessage(text: string): Promise<any> {
    if (isRemote()) {
      const res = await fetch(`${getBaseUrl()}/api/chat`, {
        method: "POST",
        headers: getApiHeaders(),
        body: JSON.stringify({ message: text }),
      });
      return res.json();
    }
    if (isTauri) {
      const { invoke } = await import("@tauri-apps/api/core");
      return invoke("send_message", { text });
    }
    const res = await fetch("/api/chat", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ message: text }),
    });
    return res.json();
  },

  async getStatus(): Promise<any> {
    if (isRemote()) {
      const res = await fetch(`${getBaseUrl()}/api/status`, {
        headers: getApiHeaders(),
      });
      return res.json();
    }
    if (isTauri) {
      const { invoke } = await import("@tauri-apps/api/core");
      return invoke("get_status");
    }
    const res = await fetch("/api/status");
    return res.json();
  },

  async clearHistory(): Promise<any> {
    if (isRemote()) {
      await fetch(`${getBaseUrl()}/api/clear`, {
        method: "POST",
        headers: getApiHeaders(),
      });
      return;
    }
    if (isTauri) {
      const { invoke } = await import("@tauri-apps/api/core");
      return invoke("clear_history");
    }
    await fetch("/api/clear", { method: "POST" });
  },

  async deleteComponent(id: string): Promise<any> {
    if (isRemote()) {
      const res = await fetch(`${getBaseUrl()}/api/studio/components/${id}`, {
        method: "DELETE",
        headers: getApiHeaders(),
      });
      return res.json();
    }
    if (isTauri) {
      const { invoke } = await import("@tauri-apps/api/core");
      return invoke("delete_component", { id });
    }
    const res = await fetch(`/api/studio/components/${id}`, { method: "DELETE" });
    return res.json();
  },

  async getSystemStatus(): Promise<any> {
    if (isRemote()) {
      const res = await fetch(`${getBaseUrl()}/api/status`, {
        headers: getApiHeaders(),
      });
      return res.json();
    }
    if (isTauri) {
      const { invoke } = await import("@tauri-apps/api/core");
      return invoke("get_system_status");
    }
    const res = await fetch("/api/console/status");
    return res.json();
  },

  async getComponentTopology(): Promise<any> {
    if (isRemote()) {
      const res = await fetch(`${getBaseUrl()}/api/topology`, {
        headers: getApiHeaders(),
      });
      return res.json();
    }
    if (isTauri) {
      const { invoke } = await import("@tauri-apps/api/core");
      return invoke("get_component_topology");
    }
    const res = await fetch("/api/console/topology");
    return res.json();
  },

  async getPresetPersona(name: string): Promise<any> {
    if (isRemote()) {
      const res = await fetch(`${getBaseUrl()}/api/personas/presets/${name}`, {
        headers: getApiHeaders(),
      });
      return res.json();
    }
    if (isTauri) {
      const { invoke } = await import("@tauri-apps/api/core");
      return invoke("get_preset_persona", { name });
    }
    const res = await fetch(`/api/personas/presets/${name}`);
    return res.json();
  },

  async listPresetPersonas(): Promise<any> {
    if (isRemote()) {
      const res = await fetch(`${getBaseUrl()}/api/personas/presets`, {
        headers: getApiHeaders(),
      });
      return res.json();
    }
    if (isTauri) {
      const { invoke } = await import("@tauri-apps/api/core");
      return invoke("list_preset_personas");
    }
    const res = await fetch("/api/personas/presets");
    return res.json();
  },

  async listComponentTypes(): Promise<any> {
    if (isRemote()) {
      const res = await fetch(`${getBaseUrl()}/api/studio/component-types`, {
        headers: getApiHeaders(),
      });
      return res.json();
    }
    if (isTauri) {
      const { invoke } = await import("@tauri-apps/api/core");
      return invoke("list_component_types");
    }
    const res = await fetch("/api/studio/component-types");
    return res.json();
  },

  async listComponents(typeFilter: string | null = null): Promise<any> {
    if (isRemote()) {
      const params = typeFilter ? `?typeFilter=${encodeURIComponent(typeFilter)}` : "";
      const res = await fetch(`${getBaseUrl()}/api/studio/components${params}`, {
        headers: getApiHeaders(),
      });
      return res.json();
    }
    if (isTauri) {
      const { invoke } = await import("@tauri-apps/api/core");
      return invoke("list_components", { typeFilter });
    }
    const params = typeFilter ? `?typeFilter=${encodeURIComponent(typeFilter)}` : "";
    const res = await fetch(`/api/studio/components${params}`);
    return res.json();
  },

  async getComponent(id: string): Promise<any> {
    if (isRemote()) {
      const res = await fetch(`${getBaseUrl()}/api/studio/components/${id}`, {
        headers: getApiHeaders(),
      });
      return res.json();
    }
    if (isTauri) {
      const { invoke } = await import("@tauri-apps/api/core");
      return invoke("get_component", { id });
    }
    const res = await fetch(`/api/studio/components/${id}`);
    return res.json();
  },

  async listAgentTemplates(): Promise<any> {
    if (isRemote()) {
      const res = await fetch(`${getBaseUrl()}/api/studio/agent-templates`, {
        headers: getApiHeaders(),
      });
      return res.json();
    }
    if (isTauri) {
      const { invoke } = await import("@tauri-apps/api/core");
      return invoke("list_agent_templates");
    }
    const res = await fetch("/api/studio/agent-templates");
    return res.json();
  },

  async testComponent(type: string, id: string, config: string, input: string): Promise<any> {
    if (isRemote()) {
      const res = await fetch(`${getBaseUrl()}/api/studio/components/${id}/test`, {
        method: "POST",
        headers: getApiHeaders(),
        body: JSON.stringify({ type, config, input }),
      });
      return res.json();
    }
    if (isTauri) {
      const { invoke } = await import("@tauri-apps/api/core");
      return invoke("test_component", { id, input, componentType: type, config });
    }
    const res = await fetch(`/api/studio/components/${id}/test`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ type, config, input }),
    });
    return res.json();
  },

  async testComponentRerun(type: string, id: string, stepIndex: number, newInput = ""): Promise<any> {
    if (isRemote()) {
      const res = await fetch(`${getBaseUrl()}/api/studio/components/${id}/test/steps/${stepIndex}`, {
        method: "POST",
        headers: getApiHeaders(),
        body: JSON.stringify({ type, newInput }),
      });
      return res.json();
    }
    if (isTauri) {
      const { invoke } = await import("@tauri-apps/api/core");
      return invoke("test_component_rerun", {
        id,
        componentType: type,
        stepIndex,
        newInput,
      });
    }
    const res = await fetch(`/api/studio/components/${id}/test/steps/${stepIndex}`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ type, newInput }),
    });
    return res.json();
  },

  async listBotStore(): Promise<any> {
    if (isRemote()) {
      const res = await fetch(`${getBaseUrl()}/api/store/bots`, {
        headers: getApiHeaders(),
      });
      return res.json();
    }
    if (isTauri) {
      const { invoke } = await import("@tauri-apps/api/core");
      return invoke("list_bot_store");
    }
    const res = await fetch("/api/store/bots");
    return res.json();
  },

  async createComponent(def: any): Promise<any> {
    if (isRemote()) {
      const res = await fetch(`${getBaseUrl()}/api/studio/components`, {
        method: "POST",
        headers: getApiHeaders(),
        body: JSON.stringify(def),
      });
      return res.json();
    }
    if (isTauri) {
      const { invoke } = await import("@tauri-apps/api/core");
      return invoke("create_component", {
        name: def.name,
        componentType: def.componentType ?? def.component_type ?? def.type,
        configJson: def.configJson ?? def.config_json ?? def.config,
      });
    }
    const res = await fetch("/api/studio/components", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(def),
    });
    return res.json();
  },

  async updateComponent(id: string, def: any): Promise<any> {
    if (isRemote()) {
      const res = await fetch(`${getBaseUrl()}/api/studio/components/${id}`, {
        method: "PUT",
        headers: getApiHeaders(),
        body: JSON.stringify(def),
      });
      return res.json();
    }
    if (isTauri) {
      const { invoke } = await import("@tauri-apps/api/core");
      return invoke("update_component", {
        id,
        name: def.name,
        componentType: def.componentType ?? def.component_type ?? def.type,
        configJson: def.configJson ?? def.config_json ?? def.config,
        status: def.status,
      });
    }
    const res = await fetch(`/api/studio/components/${id}`, {
      method: "PUT",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(def),
    });
    return res.json();
  },

  async createAgentFromDescription(nl: string): Promise<any> {
    if (isRemote()) {
      const res = await fetch(`${getBaseUrl()}/api/studio/agents/from-description`, {
        method: "POST",
        headers: getApiHeaders(),
        body: JSON.stringify({ nl }),
      });
      return res.json();
    }
    if (isTauri) {
      const { invoke } = await import("@tauri-apps/api/core");
      return invoke("create_agent_from_description", { nl });
    }
    const res = await fetch("/api/studio/agents/from-description", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ nl }),
    });
    return res.json();
  },

  async assembleAgent(def: any, registryJson = ""): Promise<any> {
    if (isRemote()) {
      const res = await fetch(`${getBaseUrl()}/api/studio/agents/assemble`, {
        method: "POST",
        headers: getApiHeaders(),
        body: JSON.stringify({ def, registryJson }),
      });
      return res.json();
    }
    if (isTauri) {
      const { invoke } = await import("@tauri-apps/api/core");
      return invoke("assemble_agent", {
        name: def.name,
        persona: def.persona,
        model: def.model,
        tools: def.tools ?? [],
        knowledge: def.knowledge ?? [],
        skills: def.skills ?? [],
        registryJson,
      });
    }
    const res = await fetch("/api/studio/agents/assemble", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ def, registryJson }),
    });
    return res.json();
  },

  async publishComponent(id: string): Promise<any> {
    if (isRemote()) {
      const res = await fetch(`${getBaseUrl()}/api/studio/components/${id}/publish`, {
        method: "POST",
        headers: getApiHeaders(),
      });
      return res.json();
    }
    if (isTauri) {
      const { invoke } = await import("@tauri-apps/api/core");
      return invoke("publish_component", { id });
    }
    const res = await fetch(`/api/studio/components/${id}/publish`, { method: "POST" });
    return res.json();
  },

  async getMarketListings(typeFilter: string | null = null): Promise<any> {
    if (isRemote()) {
      const params = typeFilter ? `?typeFilter=${encodeURIComponent(typeFilter)}` : "";
      const res = await fetch(`${getBaseUrl()}/api/market/listings${params}`, {
        headers: getApiHeaders(),
      });
      return res.json();
    }
    if (isTauri) {
      const { invoke } = await import("@tauri-apps/api/core");
      return invoke("get_market_listings", { typeFilter });
    }
    const params = typeFilter ? `?typeFilter=${encodeURIComponent(typeFilter)}` : "";
    const res = await fetch(`/api/market/listings${params}`);
    return res.json();
  },

  async installBotFromStore(botId: string, templateId: string): Promise<any> {
    if (isRemote()) {
      const res = await fetch(`${getBaseUrl()}/api/store/bots/${botId}/install`, {
        method: "POST",
        headers: getApiHeaders(),
        body: JSON.stringify({ templateId }),
      });
      return res.json();
    }
    if (isTauri) {
      const { invoke } = await import("@tauri-apps/api/core");
      return invoke("install_bot_from_store", { botId, templateId });
    }
    const res = await fetch(`/api/store/bots/${botId}/install`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ templateId }),
    });
    return res.json();
  },
};