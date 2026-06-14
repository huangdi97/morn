import { ComponentType } from "react";

export type SlotPosition = "header:left" | "header:right" | "sidebar:top" | "sidebar:bottom" | "workbench:before" | "workbench:after" | "footer";

export interface SlotPlugin {
  id: string;
  position: SlotPosition;
  label: string;
  component: ComponentType;
  order?: number;
}

const registry = new Map<SlotPosition, SlotPlugin[]>();

export function registerPlugin(plugin: SlotPlugin): void {
  const existing = registry.get(plugin.position) ?? [];
  existing.push(plugin);
  existing.sort((a, b) => (a.order ?? 0) - (b.order ?? 0));
  registry.set(plugin.position, existing);
}

export function getPlugins(position: SlotPosition): SlotPlugin[] {
  return registry.get(position) ?? [];
}

export function unregisterPlugin(id: string): void {
  for (const [position, plugins] of registry) {
    const filtered = plugins.filter((p) => p.id !== id);
    if (filtered.length !== plugins.length) {
      registry.set(position, filtered);
    }
  }
}

export function clearPlugins(): void {
  registry.clear();
}