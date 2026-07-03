import init, { VantaDB } from "../pkg/vantadb_wasm.js";

const NAMESPACE = "agent-memories";
const SIMILARITY_THRESHOLD = 0.65;

const chat = document.getElementById("chat");
const input = document.getElementById("input");
const sendBtn = document.getElementById("send-btn");
const inputArea = document.getElementById("input-area");
const statusIndicator = document.getElementById("status-indicator");
const statusText = document.getElementById("status-text");

let db = null;
let embedFn = null;

function setStatus(state, text) {
  statusIndicator.className = state;
  statusText.textContent = text;
}

function addMessage(role, content, memories = null) {
  const el = document.createElement("div");
  el.className = `message ${role}`;

  const text = document.createElement("div");
  text.textContent = content;
  el.appendChild(text);

  if (memories && memories.length > 0) {
    const details = document.createElement("div");
    details.className = "memories";
    const summary = document.createElement("summary");
    summary.textContent = `Relevant memories (${memories.length})`;
    const det = document.createElement("details");
    det.appendChild(summary);
    const ul = document.createElement("ul");
    for (const m of memories) {
      const li = document.createElement("li");
      li.innerHTML = `<strong>${escapeHtml(m.key)}</strong> — ${escapeHtml(m.payload)} <span class="score">(score: ${m.score.toFixed(3)})</span>`;
      ul.appendChild(li);
    }
    det.appendChild(ul);
    details.appendChild(det);
    el.appendChild(details);
  }

  chat.appendChild(el);
  el.scrollIntoView({ behavior: "smooth", block: "nearest" });
}

function escapeHtml(s) {
  const d = document.createElement("div");
  d.textContent = s;
  return d.innerHTML;
}

function generateResponse(userMessage, memories) {
  if (memories.length === 0) {
    return `I've stored your message: "${userMessage}". This is your first memory — future messages will recall past context automatically.`;
  }
  const top = memories[0];
  return `I remember you mentioned "${top.payload}" earlier. Based on that context, I've noted your new message: "${userMessage}". I can recall ${memories.length} related memory/memories.`;
}

async function handleMessage() {
  const text = input.value.trim();
  if (!text || !db || !embedFn) return;
  input.value = "";
  input.style.height = "auto";

  addMessage("user", text);
  setStatus("", "Processing…");
  inputArea.classList.add("loading");
  input.disabled = true;

  try {
    const vector = await embedFn(text);
    db.put({ namespace: NAMESPACE, key: `msg-${Date.now()}`, payload: text, vector: [vector] });
    const hits = db.search({ namespace: NAMESPACE, query_vector: vector, top_k: 5, distance_metric: "Cosine" });
    const memories = (hits ?? [])
      .filter(h => h.score > SIMILARITY_THRESHOLD)
      .map(h => ({ key: h.record.key, payload: h.record.payload, score: h.score }));

    if (memories.length > 0) {
      addMessage("agent", generateResponse(text, memories), memories);
    } else {
      addMessage("agent", generateResponse(text, []));
    }
    setStatus("ready", "Memory saved");
  } catch (err) {
    addMessage("system", `Error: ${err.message || err}`);
    setStatus("error", "Error");
  } finally {
    inputArea.classList.remove("loading");
    input.disabled = false;
    input.focus();
  }
}

input.addEventListener("keydown", (e) => {
  if (e.key === "Enter" && !e.shiftKey) {
    e.preventDefault();
    handleMessage();
  }
});
sendBtn.addEventListener("click", handleMessage);

input.addEventListener("input", () => {
  input.style.height = "auto";
  input.style.height = Math.min(input.scrollHeight, 120) + "px";
});

async function initEmbedder() {
  try {
    const { pipeline } = await import("https://cdn.jsdelivr.net/npm/@huggingface/transformers@3.4.0");
    const extractor = await pipeline("feature-extraction", "Xenova/all-MiniLM-L6-v2", {
      quantized: true,
    });
    embedFn = async (text) => {
      const output = await extractor(text, { pooling: "mean", normalize: true });
      return Array.from(output.data);
    };
    return true;
  } catch (err) {
    console.warn("Transformers.js failed, using simple mock embedder:", err);
    embedFn = async (text) => {
      const dim = 384;
      let hash = 0;
      for (let i = 0; i < text.length; i++) {
        hash = ((hash << 5) - hash) + text.charCodeAt(i);
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
    return false;
  }
}

async function init() {
  try {
    setStatus("", "Loading WASM…");
    await init();
    setStatus("", "Loading embedder…");
    const usingMock = await initEmbedder();
    setStatus("", "Opening database…");
    db = await VantaDB.connect_persistent("vanta-demo");
    input.disabled = false;
    sendBtn.disabled = false;
    setStatus("ready", usingMock ? "Ready (mock embedder)" : "Ready");
    input.focus();
    addMessage("system", usingMock
      ? "Transformers.js failed to load. Using deterministic mock embeddings for demo purposes."
      : "VantaDB connected with OPFS persistence. Your memories will persist across sessions.");
  } catch (err) {
    setStatus("error", `Init failed: ${err.message || err}`);
    addMessage("system", `Failed to initialize: ${err.message || err}`);
  }
}

init().catch((err) => {
  setStatus("error", "Fatal error");
  addMessage("system", `Fatal: ${err.message || err}`);
});
