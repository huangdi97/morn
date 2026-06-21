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
  async sendMessage(text: string, signal?: AbortSignal): Promise<any> {
    if (isRemote()) {
      const res = await fetch(`${getBaseUrl()}/api/chat`, {
        method: "POST",
        headers: getApiHeaders(),
        body: JSON.stringify({ message: text }),
        signal,
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
      signal,
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

  async getAgentPoolStatus(): Promise<any> {
    if (isRemote()) {
      const res = await fetch(`${getBaseUrl()}/api/agents/pool`, {
        headers: getApiHeaders(),
      });
      return res.json();
    }
    if (isTauri) {
      const { invoke } = await import("@tauri-apps/api/core");
      return invoke("get_agent_pool_status");
    }
    const res = await fetch("/api/agents/pool");
    return res.json();
  },

  async getExecutionDetails(id: string): Promise<any> {
    if (isRemote()) {
      const res = await fetch(`${getBaseUrl()}/api/executions/${id}`, {
        headers: getApiHeaders(),
      });
      return res.json();
    }
    if (isTauri) {
      const { invoke } = await import("@tauri-apps/api/core");
      return invoke("get_execution_details", { id });
    }
    const res = await fetch(`/api/executions/${id}`);
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

  async createTeam(name: string, description: string, ownerId: string): Promise<any> {
    if (isRemote()) {
      const res = await fetch(`${getBaseUrl()}/api/studio/teams`, {
        method: "POST",
        headers: getApiHeaders(),
        body: JSON.stringify({ name, description, owner_id: ownerId }),
      });
      return res.json();
    }
    if (isTauri) {
      const { invoke } = await import("@tauri-apps/api/core");
      return invoke("create_team", { name, description, ownerId });
    }
    const res = await fetch("/api/studio/teams", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ name, description, owner_id: ownerId }),
    });
    return res.json();
  },

  async listTeamTemplates(): Promise<any> {
    if (isRemote()) {
      const res = await fetch(`${getBaseUrl()}/api/studio/team-templates`, {
        headers: getApiHeaders(),
      });
      return res.json();
    }
    if (isTauri) {
      const { invoke } = await import("@tauri-apps/api/core");
      return invoke("list_team_templates");
    }
    const res = await fetch("/api/studio/team-templates");
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

  async getMarketListings(typeFilter: string | null = null, priceFilter: string | null = null): Promise<any> {
    if (isRemote()) {
      const params = new URLSearchParams();
      if (typeFilter) params.set("typeFilter", typeFilter);
      if (priceFilter) params.set("priceFilter", priceFilter);
      const res = await fetch(`${getBaseUrl()}/api/market/listings?${params}`, {
        headers: getApiHeaders(),
      });
      return res.json();
    }
    if (isTauri) {
      const { invoke } = await import("@tauri-apps/api/core");
      return invoke("get_hub_listings", { typeFilter, priceFilter });
    }
    const params = new URLSearchParams();
    if (typeFilter) params.set("typeFilter", typeFilter);
    if (priceFilter) params.set("priceFilter", priceFilter);
    const res = await fetch(`/api/hub/listings?${params}`);
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

  async hubPublish(params: { name: string; description: string; itemType: string; price: number; author: string; version: string; category: string; screenshots: string }): Promise<any> {
    if (isRemote()) {
      const res = await fetch(`${getBaseUrl()}/api/hub/publish`, {
        method: "POST",
        headers: getApiHeaders(),
        body: JSON.stringify(params),
      });
      return res.json();
    }
    if (isTauri) {
      const { invoke } = await import("@tauri-apps/api/core");
      return invoke("hub_publish", {
        name: params.name,
        description: params.description,
        itemType: params.itemType,
        price: params.price,
        author: params.author,
        version: params.version,
        category: params.category,
        screenshots: params.screenshots,
      });
    }
    const res = await fetch("/api/hub/publish", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(params),
    });
    return res.json();
  },

  async getCostSummary(): Promise<any> {
    if (isRemote()) {
      const res = await fetch(`${getBaseUrl()}/api/cost/summary`, {
        headers: getApiHeaders(),
      });
      return res.json();
    }
    if (isTauri) {
      const { invoke } = await import("@tauri-apps/api/core");
      return invoke("get_cost_summary");
    }
    const res = await fetch("/api/cost/summary");
    return res.json();
  },

  async searchMarketListings(query: string | null = null, typeFilter: string | null = null): Promise<any> {
    if (isRemote()) {
      const params = new URLSearchParams();
      if (query) params.set("query", query);
      if (typeFilter) params.set("typeFilter", typeFilter);
      const res = await fetch(`${getBaseUrl()}/api/market/search?${params}`, {
        headers: getApiHeaders(),
      });
      return res.json();
    }
    if (isTauri) {
      const { invoke } = await import("@tauri-apps/api/core");
      return invoke("search_hub_listings", { query, typeFilter });
    }
    const params = new URLSearchParams();
    if (query) params.set("query", query);
    if (typeFilter) params.set("typeFilter", typeFilter);
    const res = await fetch(`/api/hub/search?${params}`);
    return res.json();
  },

  async submitReview(listingId: string, rating: number, comment: string): Promise<any> {
    if (isRemote()) {
      const res = await fetch(`${getBaseUrl()}/api/market/reviews`, {
        method: "POST",
        headers: getApiHeaders(),
        body: JSON.stringify({ listingId, rating, comment }),
      });
      return res.json();
    }
    if (isTauri) {
      const { invoke } = await import("@tauri-apps/api/core");
      return invoke("submit_review", { listingId, rating, comment });
    }
    const res = await fetch("/api/market/reviews", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ listingId, rating, comment }),
    });
    return res.json();
  },

  async getRecentLogs(): Promise<any> {
    if (isRemote()) {
      const res = await fetch(`${getBaseUrl()}/api/logs/recent`, {
        headers: getApiHeaders(),
      });
      return res.json();
    }
    if (isTauri) {
      const { invoke } = await import("@tauri-apps/api/core");
      return invoke("get_recent_logs");
    }
    const res = await fetch("/api/logs/recent");
    return res.json();
  },

  async getListingReviews(listingId: string): Promise<any> {
    if (isRemote()) {
      const res = await fetch(`${getBaseUrl()}/api/market/reviews/${listingId}`, {
        headers: getApiHeaders(),
      });
      return res.json();
    }
    if (isTauri) {
      const { invoke } = await import("@tauri-apps/api/core");
      return invoke("get_listing_reviews", { listingId });
    }
    const res = await fetch(`/api/market/reviews/${listingId}`);
    return res.json();
  },

  async listWorkflowTemplates(): Promise<any> {
    if (isRemote()) {
      const res = await fetch(`${getBaseUrl()}/api/workflow/templates`, {
        headers: getApiHeaders(),
      });
      return res.json();
    }
    if (isTauri) {
      const { invoke } = await import("@tauri-apps/api/core");
      return invoke("list_workflow_templates");
    }
    const res = await fetch("/api/workflow/templates");
    return res.json();
  },

  async saveWorkflowTemplate(template: any): Promise<any> {
    if (isRemote()) {
      const res = await fetch(`${getBaseUrl()}/api/workflow/templates`, {
        method: "POST",
        headers: getApiHeaders(),
        body: JSON.stringify(template),
      });
      return res.json();
    }
    if (isTauri) {
      const { invoke } = await import("@tauri-apps/api/core");
      return invoke("save_workflow_template", { template });
    }
    const res = await fetch("/api/workflow/templates", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(template),
    });
    return res.json();
  },

  async getLastError(): Promise<any> {
    if (isTauri) {
      const { invoke } = await import("@tauri-apps/api/core");
      return invoke("get_last_error");
    }
    const res = await fetch("/api/recovery/last-error");
    return res.json();
  },

  async retryLastOperation(): Promise<any> {
    if (isTauri) {
      const { invoke } = await import("@tauri-apps/api/core");
      return invoke("retry_last_operation");
    }
    const res = await fetch("/api/recovery/retry", { method: "POST" });
    return res.json();
  },
};