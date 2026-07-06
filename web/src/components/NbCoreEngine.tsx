import { NbSection, NbSectionHeader } from "./nb";
import "../styles/core-engine.css";

const BLOCKS = [
  {
    id: "pyo3",
    label: "PYO3 BRIDGE",
    desc: "Zero-copy FFI",
    pins: ["GND", "VCC", "DATA", "CLK"],
    x: 10,
    y: 10,
    w: 140,
    h: 80,
  },
  {
    id: "hnsw",
    label: "HNSW INDEX",
    desc: "ANNS, 128d",
    pins: ["ADDR", "DATA", "WE", "OE"],
    x: 210,
    y: 10,
    w: 140,
    h: 80,
  },
  {
    id: "bm25",
    label: "BM25 FTS",
    desc: "Full-text, k1=1.2",
    pins: ["ADDR", "DATA", "WE", "INT"],
    x: 410,
    y: 10,
    w: 140,
    h: 80,
  },
  {
    id: "query",
    label: "QUERY ENGINE",
    desc: "SQL + RRF fusion",
    pins: ["REQ", "RSP", "BUSY", "IRQ"],
    x: 110,
    y: 150,
    w: 160,
    h: 90,
  },
  {
    id: "wal",
    label: "WAL",
    desc: "Crash-safe logging",
    pins: ["TX", "RX", "FLUSH", "SYNC"],
    x: 330,
    y: 150,
    w: 140,
    h: 80,
  },
  {
    id: "storage",
    label: "STORAGE",
    desc: "MMAP + page cache",
    pins: ["A0", "A1", "D0-D7", "CS"],
    x: 220,
    y: 290,
    w: 160,
    h: 80,
  },
];

const CONNECTIONS = [
  { from: "pyo3", to: "query", pinF: "DATA", pinT: "REQ" },
  { from: "hnsw", to: "query", pinF: "DATA", pinT: "IRQ" },
  { from: "bm25", to: "query", pinF: "DATA", pinT: "BUSY" },
  { from: "query", to: "wal", pinF: "RSP", pinT: "TX" },
  { from: "wal", to: "storage", pinF: "RX", pinT: "A0" },
  { from: "query", to: "storage", pinF: "REQ", pinT: "CS" },
];

function getBlockPos(id: string): { cx: number; cy: number; w: number; h: number } {
  const b = BLOCKS.find((x) => x.id === id)!;
  return { cx: b.x + b.w / 2, cy: b.y + b.h / 2, w: b.w, h: b.h };
}

export function NbCoreEngine() {
  return (
    <NbSection variant="lg" ariaLabel="Engine layers">
      <NbSectionHeader
        monoLabel="[BLUEPRINT]"
        headline="Engine schematic."
        sub="Each component is a self-contained chip. Traces show the data path."
      />

      <div className="nb-schematic">
        {/* ── PCB traces (SVG) ── */}
        <svg
          className="nb-schematic-svg"
          viewBox="0 0 580 420"
          preserveAspectRatio="xMidYMid meet"
          aria-hidden="true"
        >
          {/* Grid dots */}
          <defs>
            <pattern id="pcb-dot" width="20" height="20" patternUnits="userSpaceOnUse">
              <circle cx="1" cy="1" r="1" fill="var(--border-visible)" />
            </pattern>
          </defs>
          <rect width="580" height="420" fill="url(#pcb-dot)" />

          {/* Connection traces */}
          {CONNECTIONS.map((conn, i) => {
            const f = getBlockPos(conn.from);
            const t = getBlockPos(conn.to);
            const mx = (f.cx + t.cx) / 2;
            const my = (f.cy + t.cy) / 2;
            const midX = (f.cx + t.cx) / 2;
            return (
              <path
                key={conn.from + conn.to}
                d={`M ${f.cx} ${f.cy + f.h / 2} L ${f.cx} ${my} L ${t.cx} ${my} L ${t.cx} ${t.cy - t.h / 2}`}
                fill="none"
                stroke="var(--amber)"
                strokeWidth="1.5"
                strokeDasharray={i % 2 === 0 ? "none" : "6 3"}
                opacity="0.6"
              />
            );
          })}
        </svg>

        {/* ── Chip blocks ── */}
        {BLOCKS.map((block) => (
          <div
            key={block.id}
            className="nb-schematic-chip"
            style={{ left: `${block.x}px`, top: `${block.y}px`, width: `${block.w}px` }}
          >
            <div className="nb-schematic-chip-header">
              <span className="nb-schematic-chip-icon">◈</span>
              <span className="nb-schematic-chip-label">{block.label}</span>
            </div>
            <div className="nb-schematic-chip-desc">{block.desc}</div>
            <div className="nb-schematic-chip-pins">
              {block.pins.map((pin) => (
                <span key={pin} className="nb-schematic-pin">
                  {pin}
                </span>
              ))}
            </div>
          </div>
        ))}
      </div>

      {/* ── Feature cards ── */}
      <div className="nb-schematic-features">
        {[
          {
            num: "01",
            title: "Rust Core",
            desc: "Memory safety without GC. Deterministic multi-threaded performance.",
          },
          {
            num: "02",
            title: "HNSW Index",
            desc: "Hierarchical Navigable Small World for sub-ms vector search.",
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
          <article key={f.num} className="nb-schematic-feat">
            <span className="nb-num-marker">{f.num}</span>
            <h3 className="nb-schematic-feat-title">{f.title}</h3>
            <p className="nb-schematic-feat-desc">{f.desc}</p>
          </article>
        ))}
      </div>
    </NbSection>
  );
}
