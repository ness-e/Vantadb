import { useState } from "react";

const GRAPH_NODES = [
  { x: 160, y: 90, label: "agent:main", size: 14 },
  { x: 320, y: 50, label: "memory:001", size: 10 },
  { x: 280, y: 180, label: "context:rag", size: 11 },
  { x: 80, y: 200, label: "vector:embed", size: 9 },
  { x: 420, y: 130, label: "hnsw:idx", size: 10 },
  { x: 370, y: 240, label: "edge:weight", size: 8 },
  { x: 150, y: 290, label: "bm25:score", size: 9 },
  { x: 460, y: 60, label: "namespace:db", size: 12 },
];

const GRAPH_EDGES: [number, number][] = [
  [0, 1],
  [0, 3],
  [0, 2],
  [1, 4],
  [2, 4],
  [2, 5],
  [3, 6],
  [4, 7],
  [1, 7],
  [2, 6],
];

export function EngineGraphTopology() {
  const [activeNode, setActiveNode] = useState<number | null>(null);

  const isEdgeHighlighted = (a: number, b: number) => {
    if (activeNode === null) return false;
    return a === activeNode || b === activeNode;
  };

  return (
    <svg
      viewBox="0 0 540 330"
      className="engine-svg-graph"
      aria-label="Graph database node connection visualization"
    >
      <defs>
        <radialGradient id="ng" r="50%">
          <stop offset="0%" stopColor="var(--amber)" stopOpacity="0.3" />
          <stop offset="100%" stopColor="var(--amber)" stopOpacity="0" />
        </radialGradient>
      </defs>
      {GRAPH_EDGES.map(([a, b]) => (
        <line
          key={`${a}-${b}`}
          x1={GRAPH_NODES[a].x}
          y1={GRAPH_NODES[a].y}
          x2={GRAPH_NODES[b].x}
          y2={GRAPH_NODES[b].y}
          stroke={isEdgeHighlighted(a, b) ? "var(--amber)" : "var(--border)"}
          strokeWidth={isEdgeHighlighted(a, b) ? "1.5" : "1"}
          strokeDasharray={isEdgeHighlighted(a, b) ? "none" : "2 2"}
          className="engine-graph-edge"
        />
      ))}
      {GRAPH_NODES.map((n, i) => (
        <g
          key={n.label}
          className="engine-graph-node"
          onMouseEnter={() => setActiveNode(i)}
          onMouseLeave={() => setActiveNode(null)}
        >
          {activeNode === i && <circle cx={n.x} cy={n.y} r={n.size * 3} fill="url(#ng)" />}
          <circle
            cx={n.x}
            cy={n.y}
            r={n.size}
            fill="var(--surface)"
            stroke={activeNode === i ? "var(--amber)" : "var(--border)"}
            strokeWidth="1"
            className="engine-graph-ring"
          />
          <circle
            cx={n.x}
            cy={n.y}
            r="3"
            fill={activeNode === i ? "var(--amber)" : "var(--steel)"}
          />
          <text
            x={n.x}
            y={n.y - n.size - 5}
            textAnchor="middle"
            fill={activeNode === i ? "var(--foreground)" : "var(--muted)"}
            fontSize="9"
            fontFamily="var(--font-mono)"
            fontWeight="600"
            className="engine-graph-label"
          >
            {n.label}
          </text>
        </g>
      ))}
    </svg>
  );
}
