// GraphEdge.tsx (GRAFO-02): arista como línea (three-stdlib Line2 vía drei
// <Line>) entre las posiciones 2D de source/target. La arista que toca el
// nodo activo se resalta (manga: tinta vs neon).
// La posición la escribe la simulación en positionsRef (fuera del render
// React); este componente la lee por frame vía useFrame — cero re-renders.
import { Line } from "@react-three/drei";
import { useFrame } from "@react-three/fiber";
import { useRef } from "react";
import type { Line2, LineSegments2 } from "three-stdlib";
import type { VantaGraphEdgeInfo } from "../../vanta";
import type { PositionsRef } from "./useGraphData";

const EDGE_COLOR = "#3a3a3a";
const ACTIVE_COLOR = "#FFB800";

interface Props {
  edge: VantaGraphEdgeInfo;
  active: boolean;
  positionsRef: PositionsRef;
}

export default function GraphEdge({ edge, active, positionsRef }: Props) {
  const lineRef = useRef<Line2 | LineSegments2>(null);

  useFrame(() => {
    const l = lineRef.current;
    if (!l) return;
    const a = positionsRef.current.get(edge.source);
    const b = positionsRef.current.get(edge.target);
    if (!a || !b) return; // nodo sin posición todavía o evictado
    l.geometry.setPositions([a[0], a[1], 0, b[0], b[1], 0]);
    l.computeLineDistances();
  });

  return (
    <Line
      ref={lineRef}
      points={[[0, 0, 0], [0, 0, 0]]}
      color={active ? ACTIVE_COLOR : EDGE_COLOR}
      lineWidth={active ? 2 : 1}
      transparent
      opacity={active ? 1 : 0.55}
    />
  );
}