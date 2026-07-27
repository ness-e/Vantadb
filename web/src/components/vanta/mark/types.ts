// Mark variant types — shared across all mark variants
// Each variant implements the MarkVariant interface so the hero can swap them.

export type BlinkState = "open" | "left-closed" | "right-closed" | "both-closed";

export interface MarkGraphState {
  hoveredNode: number | null;
}

export interface MarkInteractionState {
  pupilOffset: { x: number; y: number };
  sphereOffset: { x: number; y: number };
  blink: BlinkState;
  annoyed: boolean;
  hoveredNode: number | null;
  mouseInHero: boolean;
}

export interface MarkVariantProps {
  /** Ref to the wrap element that centers the mark (used for mouse tracking origin) */
  markWrapRef: React.RefObject<HTMLDivElement | null>;
  /** Interaction state (provided by useMarkInteraction hook) */
  state: MarkInteractionState;
  /** Click handler — triggers blink cycle */
  onClick: () => void;
  /** Node hover handler — mark looks at the node */
  onNodeHover: (idx: number | null) => void;
}

export const MARK_VARIANTS = ["classic", "neo", "mini"] as const;
export type MarkVariantName = (typeof MARK_VARIANTS)[number];
