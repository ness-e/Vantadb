// useGraphData.ts (GRAFO-02): estado del grafo visible en la lente IQL.
//
// Seed: namespace con más registros (list) → graphDegree (top hubs, sin
// aristas). Expand incremental: click en nodo → graphBfs(roots=[nodo],
// maxDepth=1, Both, limit=50) → merge dedup de nodos/aristas. Cap MAX_NODES:
// al exceder se evictan los nodos más viejos por inserción (nunca el activo
// ni sus vecinos directos) con fade (opacidad baja 350 ms antes de remover).
//
// ponytail: evicción FIFO por inserción — sin LRU por interacción. Ceiling
// conocido: hubs del seed pueden ser evictados si el usuario expande mucho;
// re-expandir los recupera. Swap a LRU si el grafo real supera 500 nodos.
import { useCallback, useEffect, useRef, useState } from "react";
import {
  graphBfs,
  graphDegree,
  list,
  vantaErrorMessage,
  type VantaGraphEdgeInfo,
  type VantaGraphNodeInfo,
} from "../../vanta";

export const MAX_NODES = 500;
export const EXPAND_LIMIT = 50;
const SEED_LIMIT = 30;
const FADE_MS = 350;

/** Nodo visible con metadatos de sesión (orden de inserción + fade). */
export interface GraphNode extends VantaGraphNodeInfo {
  addedAt: number;
  fading: boolean;
}

export interface GraphState {
  nodes: Map<string, GraphNode>;
  edges: Map<string, VantaGraphEdgeInfo>;
  activeId: string | null;
  busy: boolean;
  /** true cuando la evicción por tope cortó nodos (aviso en toolbar). */
  capped: boolean;
  /** Namespace del seed actual (aviso/contexto). */
  namespace: string | null;
}

const emptyState: GraphState = {
  nodes: new Map(),
  edges: new Map(),
  activeId: null,
  busy: false,
  capped: false,
  namespace: null,
};

/** Layout 2D radial en espiral ordenado por degree desc → hubs al centro.
 * Fallback estático para prefers-reduced-motion (el layout animado lo corre
 * d3-force en GraphScene). */
export function radialLayout(nodes: GraphNode[]): Map<string, [number, number]> {
  const sorted = [...nodes].sort((a, b) => (b.degree ?? 0) - (a.degree ?? 0));
  const positions = new Map<string, [number, number]>();
  const n = sorted.length;
  const R = 9;
  sorted.forEach((node, i) => {
    const t = n <= 1 ? 0 : i / (n - 1);
    const angle = i * 2.39996323; // golden angle → espiral sin colisiones
    const radius = 1.2 + t * (R - 1.2);
    positions.set(node.id, [Math.cos(angle) * radius, Math.sin(angle) * radius]);
  });
  return positions;
}

/** Referencia mutable a las posiciones 2D del grafo. La simulación (d3-force
 * en GraphScene) la escribe por frame; los meshes la leen sin re-render. */
export type PositionsRef = { readonly current: Map<string, [number, number]> };

/** Clave de arista sin dirección (evita líneas dobles superpuestas). */
function edgeKey(e: VantaGraphEdgeInfo): string {
  const [a, b] = e.source < e.target ? [e.source, e.target] : [e.target, e.source];
  return `${a}→${b}`;
}

export function useGraphData(
  onNotice: (msg: string) => void,
  onError: (msg: string) => void,
) {
  const [state, setState] = useState<GraphState>(emptyState);
  const busyRef = useRef(false);
  // Orden de inserción global (único, creciente) para la evicción FIFO.
  const counterRef = useRef(0);
  // Revisión del grafo: cambia solo cuando nodes/edges cambian de contenido
  // (seed, merge, evicción). GraphScene la usa para re-sync la simulación sin
  // re-renderizar por setActiveId (que no toca el grafo).
  const [revision, setRevision] = useState(0);

  /** Merge de un traversal BFS en el estado visible con dedup + cap + fade. */
  const mergeTraversal = useCallback((prev: GraphState, res: { nodes: VantaGraphNodeInfo[]; edges: VantaGraphEdgeInfo[] }, activeId: string): GraphState => {
    const nodes = new Map(prev.nodes);
    const edges = new Map(prev.edges);
    let changed = false;
    for (const n of res.nodes) {
      if (!nodes.has(n.id)) {
        nodes.set(n.id, { ...n, addedAt: counterRef.current++, fading: false });
        changed = true;
      }
    }
    for (const e of res.edges) {
      const k = edgeKey(e);
      if (!edges.has(k)) {
        edges.set(k, e);
        changed = true;
      }
    }

    if (!changed) {
      // BFS sin datos nuevos (hoja sin vecinos): no toca el grafo.
      return { ...prev, activeId, busy: false };
    }
    setRevision((r) => r + 1);

    if (nodes.size <= MAX_NODES) {
      return { ...prev, nodes, edges, activeId, busy: false, capped: false };
    }

    // Evicción FIFO: los más viejos por inserción, preservando el activo y
    // sus vecinos directos (el vecindario recién expandido).
    const keep = new Set<string>([activeId]);
    for (const e of edges.values()) {
      if (e.source === activeId) keep.add(e.target);
      if (e.target === activeId) keep.add(e.source);
    }
    const evicted: string[] = [];
    for (const [id, n] of nodes) {
      if (nodes.size - evicted.length <= MAX_NODES) break;
      if (keep.has(id) || n.fading) continue;
      evicted.push(id);
    }
    for (const id of evicted) {
      const n = nodes.get(id);
      if (n) nodes.set(id, { ...n, fading: true });
    }

    if (evicted.length > 0) {
      window.setTimeout(() => {
        setState((cur) => {
          const next = new Map(cur.nodes);
          for (const id of evicted) {
            if (next.get(id)?.fading) next.delete(id);
          }
          // Aristas huérfanas: las que referencian un nodo ya removido.
          const edges = new Map(cur.edges);
          for (const [k, e] of edges) {
            if (!next.has(e.source) || !next.has(e.target)) edges.delete(k);
          }
          return { ...cur, nodes: next, edges };
        });
        setRevision((r) => r + 1);
      }, FADE_MS);
    }

    return { ...prev, nodes, edges, activeId, busy: false, capped: evicted.length > 0 };
  }, []);

  /** Carga inicial / reset: hubs del namespace con más registros. Limpia el
   * canvas sincrónicamente para que reset() sea inmediato (y el fit pendiente
   * encaje con el layout nuevo, no el viejo). */
  const seed = useCallback(async () => {
    if (busyRef.current) return;
    busyRef.current = true;
    setState({ ...emptyState, busy: true });
    setRevision((r) => r + 1);
    try {
      const records = await list({ limit: 500 });
      const counts = new Map<string, number>();
      for (const r of records) {
        counts.set(r.namespace, (counts.get(r.namespace) ?? 0) + 1);
      }
      const ns = [...counts.entries()].sort((a, b) => b[1] - a[1])[0]?.[0];
      if (!ns) {
        onNotice("sin registros — el grafo arranca vacío");
        setState(emptyState);
        return;
      }
      const hubs = await graphDegree({ namespace: ns, limit: SEED_LIMIT });
      const nodes = new Map<string, GraphNode>();
      for (const h of hubs) nodes.set(h.id, { ...h, addedAt: counterRef.current++, fading: false });
      setState({ nodes, edges: new Map(), activeId: null, busy: false, capped: false, namespace: ns });
      setRevision((r) => r + 1);
      if (hubs.length === 0) onNotice(`namespace ${ns} sin nodos de grafo`);
    } catch (err) {
      onError(vantaErrorMessage(err));
      setState((s) => ({ ...s, busy: false }));
    } finally {
      busyRef.current = false;
    }
  }, [onNotice, onError]);

  useEffect(() => {
    void seed();
  }, [seed]);

  /** Click en nodo → BFS desde él (limit 50) → merge al grafo visible. */
  const expand = useCallback(
    async (id: string) => {
      if (busyRef.current) return;
      busyRef.current = true;
      setState((s) => ({ ...s, busy: true, activeId: id }));
      try {
        const res = await graphBfs({ roots: [id], maxDepth: 1, direction: "Both", limit: EXPAND_LIMIT });
        setState((prev) => mergeTraversal(prev, res, id));
      } catch (err) {
        onError(vantaErrorMessage(err));
        setState((s) => ({ ...s, busy: false }));
      } finally {
        busyRef.current = false;
      }
    },
    [mergeTraversal, onError],
  );

  /** Limpia el grafo y vuelve al seed. */
  const reset = useCallback(() => {
    void seed();
  }, [seed]);

  const setActiveId = useCallback((id: string | null) => {
    setState((s) => ({ ...s, activeId: id }));
  }, []);

  return {
    nodes: [...state.nodes.values()],
    edges: [...state.edges.values()],
    revision,
    activeId: state.activeId,
    busy: state.busy,
    capped: state.capped,
    namespace: state.namespace,
    nodeCount: state.nodes.size,
    edgeCount: state.edges.size,
    expand,
    reset,
    setActiveId,
  };
}