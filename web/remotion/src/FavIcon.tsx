import React from "react";
import { useCurrentFrame } from "remotion";
import { VantaMark } from "./Mark";

/**
 * FavIcon — the page icon. A single small MARK (the mascot face, no ring text,
 * transparent background) for use as the site favicon.
 *
 * Render via: npm run render:favicon → out/favicon.png
 */
export const FavIcon: React.FC = () => {
  // favicon is a static still (single frame 0); blink is instant at frame 0
  useCurrentFrame();
  return (
    <div
      style={{
        width: 200,
        height: 200,
        backgroundColor: "transparent",
        display: "flex",
        alignItems: "center",
        justifyContent: "center",
      }}
    >
      <VantaMark scale={1} />
    </div>
  );
};