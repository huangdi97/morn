import { Fragment } from "react";
import { SlotPosition, getPlugins } from "./SlotRegistry";

interface RenderSlotProps {
  position: SlotPosition;
}

export function RenderSlot({ position }: RenderSlotProps) {
  const plugins = getPlugins(position);

  if (plugins.length === 0) return null;

  return (
    <Fragment>
      {plugins.map((plugin) => {
        const Component = plugin.component;
        return <Component key={plugin.id} />;
      })}
    </Fragment>
  );
}