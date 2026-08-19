// GraphNode.tsx (GRAFO-02): esfera toon (naranja #FF6B35) con outline negro
// estilo manga/linocut (D5) + halo para el nodo activo + label opcional.
// Tamaño ∝ degree (hub más grande). Click → expand BFS (contrato del
// orquestador); el Inspector queda para GRAFO-03 (node_id no es reversible a
// key de registro sin el core).
// La posición la escribe la simulación (d3-force) en positionsRef, fuera del
// render React; este componente la lee por frame vía useFrame — cero
// re-renders durante la simulación.
import { Html, Outlines } from "@react-three/drei";
import { useFrame } from "@react-three/fiber";
import { useRef } from "react";
import type { ThreeEvent } from "@react-three/fiber";
import type { Group } from "three";
import type { GraphNode as GraphNodeData, PositionsRef } from "./useGraphData";

export const NODE_COLOR = "#FF6B35"; // naranja toon (D4/D5)
const ACTIVE_HALO = "#FFB800";
const LABEL_MAX = 24;

interface Props {
  node: GraphNodeData;
  size: number;
  active: boolean;
  /** true solo para top-N labels (evita saturar la escena). */
  showLabel: boolean;
  positionsRef: PositionsRef;
  onSelect: (id: string) => void;
}

export default function GraphNode({ node, size, active, showLabel, positionsRef, onSelect }: Props) {
  const groupRef = useRef<Group>(null);

  useFrame(() => {
    const g = groupRef.current;
    if (!g) return;
    const p = positionsRef.current.get(node.id);
    if (p) g.position.set(p[0], p[1], 0);
  });

  const handleClick = (e: ThreeEvent<MouseEvent>) => {
    e.stopPropagation();
    onSelect(node.id);
  };
  const handleOver = (e: ThreeEvent<PointerEvent>) => {
    e.stopPropagation();
    document.body.style.cursor = "pointer";
  };
  const handleOut = () => {
    document.body.style.cursor = "auto";
  };

  const label = node.label.length > LABEL_MAX ? `${node.label.slice(0, LABEL_MAX)}…` : node.label;

  return (
    <group ref={groupRef} onClick={handleClick} onPointerOver={handleOver} onPointerOut={handleOut}>
      {/* esfera toon; fading = nodo en evicción (opacidad baja antes de remover) */}
      <mesh>
        <sphereGeometry args={[size, 20, 20]} />
        <meshToonMaterial color={NODE_COLOR} transparent={node.fading} opacity={node.fading ? 0.15 : 1} />
      </mesh>
      {/* outline manga (inverted hull, drei) */}
      <Outlines color="black" thickness={0.06} />
      {/* halo del nodo activo */}
      {active && (
        <mesh scale={1.7}>
          <sphereGeometry args={[size, 14, 14]} />
          <meshBasicMaterial color={ACTIVE_HALO} transparent opacity={0.22} depthWrite={false} />
        </mesh>
      )}
      {showLabel && (
        <Html
          center
          position={[0, size + 0.5, 0]}
          zIndexRange={[20, 0]}
          style={{
            pointerEvents: "none",
            whiteSpace: "nowrap",
            fontSize: 11,
            fontWeight: 700,
            color: active ? "#B45309" : "#1f2937",
            textShadow: "0 1px 2px rgba(255,255,255,0.9)",
          }}
        >
          {label}
        </Html>
      )}
    </group>
  );
}