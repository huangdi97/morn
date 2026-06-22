import { ComponentType } from "react";

export type SlotPosition = "header:left" | "header:right" | "sidebar:top" | "sidebar:bottom" | "workbench:before" | "workbench:after" | "footer";

export interface SlotPlugin {
  id: string;
  position: SlotPosition;
  label: string;
  component: ComponentType;
  order?: number;
}

export interface SlotMeta {
  label: string;
  icon?: string;
  pluginId: string;
  priority?: number;
}

export interface SlotRegistration {
  id: string;
  component: ComponentType<any>;
  meta: SlotMeta;
}

class SlotRegistryImpl {
  private slots = new Map<string, SlotRegistration[]>();

  register(slotName: string, registration: SlotRegistration) {
    const list = [...(this.slots.get(slotName) || [])];
    list.push(registration);
    list.sort((a, b) => (b.meta.priority ?? 0) - (a.meta.priority ?? 0));
    this.slots.set(slotName, list);
  }

  get(slotName: string): SlotRegistration[] {
    return this.slots.get(slotName) || [];
  }

  unregister(slotName: string, id: string) {
    const list = this.slots.get(slotName);
    if (list) {
      this.slots.set(slotName, list.filter(r => r.id !== id));
    }
  }

  clear() {
    this.slots.clear();
  }

  /** @internal iterate all slots (for unregister-by-id across all slots) */
  [Symbol.iterator]() {
    return this.slots[Symbol.iterator]();
  }
}

export const slotRegistry = new SlotRegistryImpl();

const POSITION_TO_SLOT: Record<string, string> = {
  "footer": "status-bar",
  "sidebar:top": "console-panels",
  "sidebar:bottom": "console-panels",
  "console:tab": "console-panels",
  "studio:panel": "studio-tools",
};

export function registerPlugin(plugin: SlotPlugin): void {
  const slotName = POSITION_TO_SLOT[plugin.position] || plugin.position;
  slotRegistry.register(slotName, {
    id: plugin.id,
    component: plugin.component,
    meta: {
      label: plugin.label,
      pluginId: plugin.id,
      priority: plugin.order ?? 0,
    },
  });
}

export function getPlugins(position: SlotPosition): SlotRegistration[] {
  const slotName = POSITION_TO_SLOT[position] || position;
  return slotRegistry.get(slotName);
}

export function unregisterPlugin(id: string): void {
  for (const [slotName] of slotRegistry) {
    slotRegistry.unregister(slotName, id);
  }
}

export function clearPlugins(): void {
  slotRegistry.clear();
}