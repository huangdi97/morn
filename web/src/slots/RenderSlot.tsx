import React from "react";
import { slotRegistry } from "./SlotRegistry";

interface RenderSlotProps {
  name: string;
  fallback?: React.ReactNode;
}

export const RenderSlot: React.FC<RenderSlotProps> = ({ name, fallback }) => {
  const registrations = slotRegistry.get(name);

  if (registrations.length === 0) {
    return fallback ?? null;
  }

  return (
    <>
      {registrations.map(reg => (
        <reg.component key={reg.id} />
      ))}
    </>
  );
};