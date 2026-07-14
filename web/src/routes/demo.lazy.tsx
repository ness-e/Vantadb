import { createLazyRoute } from "@tanstack/react-router";
import { useCallback, useEffect, useRef, useState } from "react";

type HFExtractor = (
  text: string,
  options: { pooling: "mean"; normalize: true },
) => Promise<{ data: Float32Array }>;
type PipelineFn = (
  task: string,
  model: string,
  options?: { quantized?: boolean },
) => Promise<HFExtractor>;
import { NbSubpageHero } from "@/components/NbSubpageHero";
import { NbSection, NbSectionHeader } from "@/components/nb";
import { PendingComponent } from "@/components/PendingComponent";
import { useAnimationSafe } from "@/hooks/useAnimationSafe";
import { fadeUp } from "@/lib/motion-utils";
import "../styles/demo.css";

export const Route = createLazyRoute("/demo")({
  component: DemoPage,
  pendingComponent: PendingComponent,
});

const NAMESPACE = "agent-memories";
const SIMILARITY_THRESHOLD = 0.65;

type Memory = { key: string; payload: string; score: number };
type Msg = { role: "user" | "agent" | "system"; content: string; memories?: Memory[] };
type HitResult = { score: number; record: { key: string; payload: string } };
interface VantaDemoDB {
  put(opts: { namespace: string; key: string; payload: string; vector: number[] }): void;
  search(opts: {
    namespace: string;
    query_vector: number[];
    top_k: number;
    distance_metric: string;
  }): HitResult[];
}

function escapeHtml(s: string) {
  const d = document.createElement("div");
  d.textContent = s;
  return d.innerHTML;
}

function generateResponse(userMessage: string, memories: Memory[]) {
  if (memories.length === 0) {
    return `I've stored your message: "${userMessage}". This is your first memory — future messages will recall past context automatically.`;
  }
  const top = memories[0];
  return `I remember you mentioned "${top.payload}" earlier. Based on that context, I've noted your new message: "${userMessage}". I can recall ${memories.length} related memory/memories.`;
}

function DemoPage() {
  const chatRef = useRef<HTMLDivElement>(null);
  const inputRef = useRef<HTMLTextAreaElement>(null);
  const sectionRef = useRef<HTMLElement>(null);
  const chatShellRef = useRef<HTMLDivElement>(null);
  const codeRef = useRef<HTMLDivElement>(null);
  const infoRef = useRef<HTMLDivElement>(null);

  const [messages, setMessages] = useState<Msg[]>([
    { role: "system", content: "Initializing VantaDB WASM…" },
  ]);
  const [input, setInput] = useState("");
  const [busy, setBusy] = useState(false);
  const [status, setStatus] = useState<"loading" | "ready" | "error">("loading");
  const [statusText, setStatusText] = useState("Loading WASM…");
  const [dbReady, setDbReady] = useState(false);

  const dbRef = useRef<VantaDemoDB | null>(null);
  const embedRef = useRef<((text: string) => Promise<number[]>) | null>(null);

  const addMsg = useCallback((msg: Msg) => {
    setMessages((prev) => [...prev, msg]);
  }, []);

  const pushStatus = useCallback((state: "loading" | "ready" | "error", text: string) => {
    setStatus(state);
    setStatusText(text);
  }, []);

  useEffect(() => {
    if (chatRef.current) {
      chatRef.current.scrollTop = chatRef.current.scrollHeight;
    }
  }, [messages]);

  useAnimationSafe(() => {
    const el = chatShellRef.current;
    if (el) fadeUp(el);
  }, chatShellRef);

  useAnimationSafe(() => {
    const el = codeRef.current;
    if (el) fadeUp(el, { stagger: 0 });
  }, codeRef);

  useAnimationSafe(() => {
    const el = infoRef.current;
    if (el) fadeUp(el, { stagger: 0 });
  }, infoRef);

  useEffect(() => {
    let cancelled = false;

    async function boot() {
      try {
        pushStatus("loading", "Loading WASM…");

        const { VantaDB } = await import("../wasm/vantadb_wasm.js");

        pushStatus("loading", "Loading embedder…");

        let usingMock = false;
        try {
          const TRANSFORMERS_URL = "https://cdn.jsdelivr.net/npm/@huggingface/transformers@3.4.0";
          const mod = (await import(/* @vite-ignore */ TRANSFORMERS_URL)) as {
            pipeline: PipelineFn;
          };
          const pipeline = mod.pipeline as PipelineFn;
          const extractor = await pipeline("feature-extraction", "Xenova/all-MiniLM-L6-v2", {
            quantized: true,
          });
          embedRef.current = async (text: string) => {
            const output = await extractor(text, { pooling: "mean", normalize: true });
            return Array.from(output.data) as number[];
          };
        } catch {
          usingMock = true;
          embedRef.current = async (text: string) => {
            const dim = 384;
            let hash = 0;
            for (let i = 0; i < text.length; i++) {
              hash = (hash << 5) - hash + text.charCodeAt(i);
              hash |= 0;
            }
            const seed = Math.abs(hash);
            const arr = new Float32Array(dim);
            let s = seed;
            for (let i = 0; i < dim; i++) {
              s = (s * 1103515245 + 12345) & 0x7fffffff;
              arr[i] = (s / 0x7fffffff) * 2 - 1;
            }
            let norm = 0;
            for (let i = 0; i < dim; i++) norm += arr[i] * arr[i];
            norm = Math.sqrt(norm);
            for (let i = 0; i < dim; i++) arr[i] /= norm;
            return Array.from(arr);
          };
        }

        if (cancelled) return;

        pushStatus("loading", "Opening database…");
        let db: VantaDemoDB;
        let persistent = false;
        try {
          db = await VantaDB.connect_persistent("vanta-demo");
          persistent = true;
        } catch {
          db = VantaDB.open("vanta-demo");
        }
        dbRef.current = db;

        if (cancelled) return;

        setDbReady(true);
        setMessages([
          {
            role: "system",
            content: usingMock
              ? "Transformers.js failed to load. Using deterministic mock embeddings for demo purposes."
              : persistent
                ? "VantaDB connected with OPFS persistence. Your memories will persist across sessions."
                : "VantaDB connected (in-memory). Persistence via OPFS was not available.",
          },
        ]);
        pushStatus("ready", usingMock ? "Ready (mock embedder)" : "Ready");
      } catch (err: unknown) {
        if (!cancelled) {
          const msg = err instanceof Error ? err.message : String(err);
          pushStatus("error", `Init failed: ${msg}`);
          setMessages([{ role: "system", content: `Failed to initialize: ${msg}` }]);
        }
      }
    }

    void boot();
    return () => {
      cancelled = true;
    };
  }, [pushStatus]);

  const handleSend = useCallback(async () => {
    const text = input.trim();
    if (!text || !dbRef.current || !embedRef.current || busy) return;
    setInput("");
    setBusy(true);

    addMsg({ role: "user", content: text });
    pushStatus("loading", "Processing…");

    try {
      const vector = await embedRef.current(text);
      dbRef.current.put({
        namespace: NAMESPACE,
        key: `msg-${Date.now()}`,
        payload: text,
        vector: vector,
      });
      const hits = dbRef.current.search({
        namespace: NAMESPACE,
        query_vector: vector,
        top_k: 5,
        distance_metric: "Cosine",
      });
      const memories: Memory[] = (hits ?? [])
        .filter((h: HitResult) => h.score > SIMILARITY_THRESHOLD)
        .map((h: HitResult) => ({ key: h.record.key, payload: h.record.payload, score: h.score }));

      addMsg({ role: "agent", content: generateResponse(text, memories), memories });
      pushStatus("ready", "Memory saved");
    } catch (err: unknown) {
      const msg = err instanceof Error ? err.message : String(err);
      addMsg({ role: "system", content: `Error: ${msg}` });
      pushStatus("error", "Error");
    } finally {
      setBusy(false);
    }
  }, [input, busy, addMsg, pushStatus]);

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === "Enter" && !e.shiftKey) {
      e.preventDefault();
      handleSend();
    }
  };

  const handleInput = (e: React.ChangeEvent<HTMLTextAreaElement>) => {
    setInput(e.target.value);
    e.target.style.height = "auto";
    e.target.style.height = Math.min(e.target.scrollHeight, 120) + "px";
  };

  return (
    <main className="demo-page" ref={sectionRef}>
      <NbSubpageHero
        title={
          <span>
            Try VantaDB
            <br />
            in Your Browser
          </span>
        }
        sub="AI-powered vector memory running entirely client-side via WebAssembly. No server, no install."
      />

      <NbSection>
        <div ref={chatShellRef} className="demo-chat-shell">
          <div className="demo-status">
            <span className={`demo-dot ${status}`} />
            <span>{statusText}</span>
          </div>

          <div className="demo-chat" ref={chatRef}>
            {messages.map((msg, i) => (
              <div key={i} className={`message ${msg.role}`}>
                <div>{msg.content}</div>
                {msg.memories && msg.memories.length > 0 && (
                  <details className="memories">
                    <summary>Relevant memories ({msg.memories.length})</summary>
                    <ul>
                      {msg.memories.map((m, j) => (
                        <li key={j}>
                          <strong>{escapeHtml(m.key)}</strong> — {escapeHtml(m.payload)}{" "}
                          <span className="score">(score: {m.score.toFixed(3)})</span>
                        </li>
                      ))}
                    </ul>
                  </details>
                )}
              </div>
            ))}
          </div>

          <div className={`demo-input-area${busy ? " loading" : ""}`}>
            <textarea
              ref={inputRef}
              value={input}
              onChange={handleInput}
              onKeyDown={handleKeyDown}
              placeholder={dbReady ? "Type a message…" : "Initializing…"}
              disabled={!dbReady || busy}
              rows={1}
            />
            <button
              className="demo-send-btn"
              onClick={handleSend}
              disabled={!dbReady || busy || !input.trim()}
              aria-label="Send"
            >
              <svg
                width="18"
                height="18"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                strokeWidth="2"
                strokeLinecap="round"
                strokeLinejoin="round"
              >
                <line x1="22" y1="2" x2="11" y2="13" />
                <polygon points="22 2 15 22 11 13 2 9 22 2" />
              </svg>
            </button>
          </div>
        </div>
      </NbSection>

      <NbSection>
        <NbSectionHeader
          monoLabel="CODE"
          headline="How It Works"
          sub="~40 lines of JavaScript. Persistent vector memory in any browser."
        />
        <div ref={codeRef} className="demo-code-block">
          <span className="lang-label">JavaScript</span>
          <pre>{`import { VantaDB } from "vantadb-wasm";

const db = await VantaDB.connect_persistent("my-demo");
const embed = await loadEmbedder("all-MiniLM-L6-v2");

// store
const vector = await embed("agent memory");
db.put({ namespace: "demo", key: "msg-1", payload: "agent memory", vector: [vector] });

// search (BM25 + HNSW hybrid)
const hits = db.search({
  namespace: "demo",
  query_vector: vector,
  top_k: 5,
  distance_metric: "Cosine",
});
console.log(hits.map(h => h.record.payload));
// → ["agent memory"]`}</pre>
        </div>
        <div ref={infoRef} className="demo-connection-info">
          <span className="check">✓</span> Data persists via OPFS
          <span className="check" style={{ marginLeft: "1rem" }}>
            ✓
          </span>{" "}
          All-in-browser, zero server
          <span className="check" style={{ marginLeft: "1rem" }}>
            ✓
          </span>{" "}
          BM25 + HNSW hybrid search
        </div>
      </NbSection>
    </main>
  );
}
