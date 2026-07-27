import { NbSection, NbSectionHeader } from "./nb";
import "../styles/core-engine.css";

const BLOCKS = [
  {
    id: "pyo3",
    label: "PYO3 BRIDGE",
    desc: "Zero-copy FFI layer",
    pins: ["GND", "VCC", "DATA", "CLK"],
    x: 40,
    y: 30,
    w: 300,
    h: 140,
  },
  {
    id: "hnsw",
    label: "HNSW INDEX",
    desc: "Hierarchical NSW, 128d",
    pins: ["ADDR", "DATA", "WE", "OE"],
    x: 400,
    y: 30,
    w: 300,
    h: 140,
  },
  {
    id: "bm25",
    label: "BM25 FTS",
    desc: "Full-text, k1=1.2, unicode",
    pins: ["ADDR", "DATA", "WE", "INT"],
    x: 760,
    y: 30,
    w: 300,
    h: 140,
  },
  {
    id: "query",
    label: "QUERY ENGINE",
    desc: "RRF fusion planner",
    pins: ["REQ", "RSP", "BUSY", "IRQ"],
    x: 330,
    y: 260,
    w: 440,
    h: 130,
  },
  {
    id: "wal",
    label: "WAL",
    desc: "Write-ahead log, ACID",
    pins: ["TX", "RX", "FLUSH", "SYNC"],
    x: 190,
    y: 480,
    w: 280,
    h: 120,
  },
  {
    id: "storage",
    label: "STORAGE ENGINE",
    desc: "MMAP + page cache",
    pins: ["A0-A7", "D0-D7", "CS", "RDY"],
    x: 630,
    y: 480,
    w: 280,
    h: 120,
  },
];

const CONNECTIONS = [
  { from: "pyo3", to: "query" },
  { from: "hnsw", to: "query" },
  { from: "bm25", to: "query" },
  { from: "query", to: "wal" },
  { from: "wal", to: "storage" },
  { from: "query", to: "storage" },
];

function getBlockPos(id: string): { cx: number; cy: number; w: number; h: number } {
  const b = BLOCKS.find((x) => x.id === id);
  if (!b) return { cx: 0, cy: 0, w: 0, h: 0 };
  return { cx: b.x + b.w / 2, cy: b.y + b.h / 2, w: b.w, h: b.h };
}

function tracePath(from: string, to: string): string {
  const f = getBlockPos(from);
  const t = getBlockPos(to);
  const sx = f.cx;
  const sy = f.cy + f.h / 2;
  const ex = t.cx;
  const ey = t.cy - t.h / 2;
  const my = (sy + ey) / 2;
  return `M ${sx} ${sy} L ${sx} ${my} L ${ex} ${my} L ${ex} ${ey}`;
}

export function NbCoreEngine() {
  return (
    <NbSection variant="lg" ariaLabel="Engine layers">
      <NbSectionHeader
        monoLabel="[BLUEPRINT]"
        headline="Engine schematic."
        sub="Each component is a self-contained module. Traces show the data path."
      />

      <div className="nc-schematic-wrap">
        <div className="nc-schematic">
          <svg
            className="nc-schematic-svg"
            viewBox="0 0 1100 640"
            preserveAspectRatio="xMidYMid meet"
            aria-hidden="true"
          >
            <defs>
              <pattern id="nc-pcb-grid" width="40" height="40" patternUnits="userSpaceOnUse">
                <circle cx="2" cy="2" r="1.5" fill="var(--border-visible)" />
                <circle cx="22" cy="22" r="0.8" fill="var(--border)" />
              </pattern>
            </defs>

            <rect width="1100" height="640" fill="url(#nc-pcb-grid)" />

            {CONNECTIONS.map((conn, i) => {
              const dash = i % 2 === 0 ? "none" : "8 5";
              return (
                <path
                  key={conn.from + conn.to}
                  d={tracePath(conn.from, conn.to)}
                  fill="none"
                  stroke="var(--amber)"
                  strokeWidth="2"
                  strokeDasharray={dash}
                  opacity="0.6"
                />
              );
            })}

            {CONNECTIONS.map((conn) => {
              const f = getBlockPos(conn.from);
              const t = getBlockPos(conn.to);
              const my = (f.cy + f.h / 2 + t.cy - t.h / 2) / 2;
              return (
                <g key={`dot-${conn.from}-${conn.to}`}>
                  <circle cx={f.cx} cy={my} r="3" fill="var(--amber)" opacity="0.8" />
                  <circle cx={t.cx} cy={my} r="3" fill="var(--amber)" opacity="0.8" />
                </g>
              );
            })}
          </svg>

          {BLOCKS.map((block) => (
            <div
              key={block.id}
              className="nc-schematic-chip"
              style={{
                left: `${block.x}px`,
                top: `${block.y}px`,
                width: `${block.w}px`,
                height: `${block.h}px`,
              }}
            >
              <div className="nc-chip-header">
                <span className="nc-chip-badge">{block.label.slice(0, 4)}</span>
                <span className="nc-chip-label">{block.label}</span>
              </div>
              <div className="nc-chip-desc">{block.desc}</div>
              <div className="nc-chip-pins">
                {block.pins.map((pin) => (
                  <span key={pin} className="nc-pin">
                    {pin}
                  </span>
                ))}
              </div>
            </div>
          ))}
        </div>
      </div>

      <div className="nc-schematic-features">
        {[
          {
            num: "01",
            title: "Rust Core",
            desc: "Memory safety without GC. Deterministic multi-threaded execution.",
          },
          {
            num: "02",
            title: "HNSW Index",
            desc: "Hierarchical Navigable Small World for sub-ms ANN vector search.",
          },
          {
            num: "03",
            title: "BM25 Engine",
            desc: "Full-text with unicode tokenizer. Combine with vectors via RRF.",
          },
          {
            num: "04",
            title: "WAL Durability",
            desc: "Write-Ahead Log with automatic crash recovery. Zero data loss.",
          },
        ].map((f) => (
          <article key={f.num} className="nc-schematic-feat">
            <span className="nc-num-marker">{f.num}</span>
            <h3 className="nc-schematic-feat-title">{f.title}</h3>
            <p className="nc-schematic-feat-desc">{f.desc}</p>
          </article>
        ))}
      </div>
    </NbSection>
  );
}
