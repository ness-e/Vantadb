// GraphScene.tsx (GRAFO-02): contenido del <Canvas> — luces (toon necesita
// luz), OrbitControls y el grafo force-directed.
//
// La simulación d3-force corre en un frame loop controlado (useFrame), fuera
// del render React: las posiciones viven en positionsRef y GraphNode/GraphEdge
// las leen por frame vía useFrame (cero re-renders). La expansión incremental
// re-sync nodos/aristas a la simulación y la reheata (alpha) para que las
// fuerzas reacomoden el grafo completo; los nodos nuevos nacen en la posición
// del nodo expandido (efecto de crecimiento, no teleport). prefers-reduced-
// motion → layout radial estático (fallback sin animación, contrato a11y).
import { OrbitControls } from "@react-three/drei";
import { useFrame } from "@react-three/fiber";
import { useEffect, useMemo, useRef, useState } from "react";
import {
  forceCenter,
  forceCollide,
  forceLink,
  forceManyBody,
  forceSimulation,
  type ForceLink,
  type Simulation,
  type SimulationLinkDatum,
  type SimulationNodeDatum,
} from "d3-force";
import type { OrbitControls as OrbitControlsImpl } from "three-stdlib";
import type { VantaGraphEdgeInfo } from "../../vanta";
import GraphEdge from "./GraphEdge";
import GraphNode from "./GraphNode";
import { radialLayout, type GraphNode as GraphNodeData } from "./useGraphData";

export const TOP_LABELS = 20;

interface Props {
  nodes: GraphNodeData[];
  edges: VantaGraphEdgeInfo[];
  /** Revisión del grafo (useGraphData): re-sync de la simulación solo cuando
   * cambió el contenido, no en re-renders por setActiveId. */
  revision: number;
  activeId: string | null;
  /** Nodos resaltados por la consola IQL (resultado Read) — GRAFO-03. */
  highlightIds: ReadonlySet<string>;
  showLabels: boolean;
  fitSignal: number;
  onSelectNode: (id: string) => void;
}

/** Nodo de simulación: datos del grafo + campos que muta d3-force (x/y/vx/vy). */
interface SimNode extends GraphNodeData, SimulationNodeDatum {}

type SimLink = SimulationLinkDatum<SimNode>;

/** Ticks a esperar antes del fit automático: el primer batch nace en el
 * centro (bbox puntual); encuadrar tras desplegar, no sobre el punto. */
const FIT_AFTER_TICKS = 30;

/** Tamaño de nodo ∝ degree (hub más grande). Grado 0 (BFS) → mínimo. */
function nodeSize(degree: number | undefined): number {
  return Math.min(1.1, 0.35 + (degree ?? 0) * 0.14);
}

/** prefers-reduced-motion (drei v10 no exporta useReducedMotion; matchMedia
 * nativo basta). */
function usePrefersReducedMotion(): boolean {
  const [reduced, setReduced] = useState(
    () => window.matchMedia("(prefers-reduced-motion: reduce)").matches,
  );
  useEffect(() => {
    const mq = window.matchMedia("(prefers-reduced-motion: reduce)");
    const onChange = () => setReduced(mq.matches);
    mq.addEventListener("change", onChange);
    return () => mq.removeEventListener("change", onChange);
  }, []);
  return reduced;
}

export default function GraphScene({ nodes, edges, revision, activeId, highlightIds, showLabels, fitSignal, onSelectNode }: Props) {
  const reducedMotion = usePrefersReducedMotion();
  const controlsRef = useRef<OrbitControlsImpl>(null);
  // Posiciones vivas: la simulación (o el fallback radial) las escribe aquí;
  // los meshes las leen por frame vía useFrame (fuera del render React).
  const positionsRef = useRef<Map<string, [number, number]>>(new Map());
  // Datum de la simulación por id: conserva x/y/vx/vy entre syncs (los nodos
  // que persisten no reinician su posición al expandir).
  const simNodesRef = useRef<Map<string, SimNode>>(new Map());
  const simRef = useRef<Simulation<SimNode, SimLink> | null>(null);
  const linkForceRef = useRef<ForceLink<SimNode, SimLink> | null>(null);
  const tickCountRef = useRef(0);
  // Fit encolado (post-reset o primer batch): se ejecuta tras FIT_AFTER_TICKS
  // de simulación (o inmediato en reduced-motion).
  const fitRef = useRef(false);
  // Ignora re-renders sin cambio de grafo (ej. setActiveId).
  const revisionRef = useRef(-1);

  // Top-N labels por degree (evita saturar — contrato "solo top-N").
  const topLabelIds = useMemo(() => {
    const sorted = [...nodes].sort((a, b) => (b.degree ?? 0) - (a.degree ?? 0));
    return new Set(sorted.slice(0, TOP_LABELS).map((n) => n.id));
  }, [nodes]);

  // Fit: encuadra el bbox actual de positionsRef (vivo).
  const fit = () => {
    const c = controlsRef.current;
    const pos = positionsRef.current;
    if (!c || pos.size === 0) return;
    let minX = Infinity;
    let maxX = -Infinity;
    let minY = Infinity;
    let maxY = -Infinity;
    for (const [x, y] of pos.values()) {
      if (x < minX) minX = x;
      if (x > maxX) maxX = x;
      if (y < minY) minY = y;
      if (y > maxY) maxY = y;
    }
    const cx = (minX + maxX) / 2;
    const cy = (minY + maxY) / 2;
    const r = Math.max(maxX - minX, maxY - minY, 4) / 2;
    c.target.set(cx, cy, 0);
    c.object.position.set(cx, cy, r * 2.6);
    c.update();
  };

  // Fit manual / reset: si el grafo quedó vacío (reset limpió), encolar hasta
  // que el seed asíncrono llegue y la simulación despliegue.
  useEffect(() => {
    if (positionsRef.current.size > 0) fit();
    else fitRef.current = true;
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [fitSignal]);

  // Sync del grafo → simulación (o layout estático si reduced-motion). Se
  // re-corre en cada expansión: la simulación recibe nodos/aristas nuevos y
  // se reheata (alpha) para que las fuerzas reacomoden el grafo completo.
  useEffect(() => {
    if (revisionRef.current === revision) return;
    revisionRef.current = revision;

    if (reducedMotion) {
      simRef.current?.stop();
      simRef.current = null;
      positionsRef.current = radialLayout(nodes);
      if (fitRef.current) {
        fitRef.current = false;
        fit();
      }
      return;
    }

    if (nodes.length === 0) {
      simRef.current?.stop();
      simRef.current = null;
      simNodesRef.current = new Map();
      positionsRef.current = new Map();
      return;
    }

    // Los nodos nuevos nacen en la posición del nodo expandido (efecto de
    // crecimiento, no teleport); centro si no hay origen.
    const origin = positionsRef.current.get(activeId ?? "") ?? [0, 0];
    const simNodes: SimNode[] = [];
    for (const n of nodes) {
      const prev = simNodesRef.current.get(n.id);
      simNodes.push(prev ?? { ...n, x: origin[0], y: origin[1] });
    }
    simNodesRef.current = new Map(simNodes.map((n) => [n.id, n]));
    const simLinks: SimLink[] = edges.map((e) => ({ source: e.source, target: e.target }));

    let sim = simRef.current;
    if (!sim) {
      // Disuasión de hubs (research 03): repulsión global fuerte (charge -18)
      // evita que los hubs colapsen al centro; strength de link moderado (0.5)
      // + distancia 2.2 evita que un hub arrastre todo el grafo; forceCollide
      // con radio ∝ tamaño evita overlap. tick manual → frame loop controlado
      // del contrato (sin worker: 500 nodos < 1 ms/frame en main thread).
      linkForceRef.current = forceLink<SimNode, SimLink>(simLinks)
        .id((d) => d.id)
        .distance(2.2)
        .strength(0.5);
      sim = forceSimulation<SimNode>(simNodes)
        .force("link", linkForceRef.current)
        .force("charge", forceManyBody<SimNode>().strength(-18).distanceMax(25))
        .force("collide", forceCollide<SimNode>().radius((d) => nodeSize(d.degree) + 0.15).strength(0.85))
        .force("center", forceCenter<SimNode>(0, 0))
        .stop(); // tick manual en useFrame
      simRef.current = sim;
    } else {
      sim.nodes(simNodes);
      linkForceRef.current?.links(simLinks);
    }
    sim.alpha(0.35); // reheat: las fuerzas re-corren sobre el grafo nuevo
    if (fitRef.current) tickCountRef.current = 0;
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [revision, activeId, reducedMotion]);

  // Tick de la simulación fuera del render React: avanza d3-force un paso por
  // frame y escribe positionsRef (GraphNode/GraphEdge leen de ahí por frame).
  useFrame(() => {
    if (reducedMotion) return;
    const sim = simRef.current;
    if (!sim) return;
    sim.tick();
    const pos = positionsRef.current;
    for (const n of sim.nodes()) {
      if (n.x !== undefined && n.y !== undefined) pos.set(n.id, [n.x, n.y]);
    }
    // Fit pendiente: ejecutar cuando la simulación ya desplegó.
    if (fitRef.current && ++tickCountRef.current >= FIT_AFTER_TICKS) {
      fitRef.current = false;
      fit();
    }
  });

  return (
    <>
      <ambientLight intensity={0.75} />
      <directionalLight position={[6, 10, 14]} intensity={1.3} />
      <OrbitControls ref={controlsRef} makeDefault maxDistance={60} />

      {nodes.map((node) => (
        <GraphNode
          key={node.id}
          node={node}
          size={nodeSize(node.degree)}
          active={node.id === activeId}
          highlighted={highlightIds.has(node.id)}
          showLabel={showLabels && topLabelIds.has(node.id)}
          positionsRef={positionsRef}
          onSelect={onSelectNode}
        />
      ))}

      {edges.map((edge) => (
        <GraphEdge
          key={`${edge.source}:${edge.target}`}
          edge={edge}
          active={edge.source === activeId || edge.target === activeId}
          positionsRef={positionsRef}
        />
      ))}
    </>
  );
}