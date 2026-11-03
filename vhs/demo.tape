# VHS Script: VantaDB 60-Second Terminal Demo
# Render with: vhs vhs/demo.tape -> produces assets/demo.gif & assets/demo.mp4
# Requirements: vhs (https://github.com/charmbracelet/vhs), vanta-cli on PATH

Output assets/demo.gif
Output assets/demo.mp4

Set Shell "bash"
Set FontSize 15
Set Width 1280
Set Height 720
Set Padding 20
Set FontFamily "JetBrains Mono"
Set Theme "Catppuccin Macchiato"
Set WindowBar "Colorful"
Set LetterSpacing 0
Set LineHeight 1.2

# --- ESCENA 1: Presentación e Inspección de Salud (0s - 12s) ---
Type "# ⚡ VantaDB: Zero-Docker, Local-First Long-Term Memory for AI Agents"
Enter
Sleep 1.5s

Type "vanta-cli config doctor"
Sleep 500ms
Enter
Sleep 3.5s

# --- ESCENA 2: Onboarding MCP & Profile Ultra-Ligero (12s - 24s) ---
Type "# 🚀 Iniciando Servidor MCP en perfil 'brain' (Solo 12 tools, ~2.4k tokens vs 23k)"
Enter
Sleep 1s

Type "VANTADB_MCP_PROFILE=brain vanta-cli server --mcp --db ~/.vantadb"
Sleep 500ms
Enter
Sleep 2.5s
# Simular handshake JSON-RPC stdio
Type '{"jsonrpc":"2.0","id":1,"method":"tools/list"}'
Enter
Sleep 2.5s
Ctrl+C
Sleep 1s

# --- ESCENA 3: Ingesta Cognitiva L0 -> L2 y Embeddings Duales (24s - 38s) ---
Type "# 🧠 Ingesta Cognitiva L0 -> L2 con embeddings locales (EmbeddingGemma 2 / E5)"
Enter
Sleep 1s

Type "vanta-cli put --db ~/.vantadb --namespace agent/architecture --key arch_01 --payload 'Decisión: usar Fjall LSM + WAL CRC32C en lugar de SQLite para eliminar locks multi-instancia en Cursor y Claude Code.'"
Sleep 500ms
Enter
Sleep 2.5s

Type "vanta-cli put --db ~/.vantadb --namespace agent/architecture --key arch_02 --payload 'Regla: el perfil brain de MCP reduce en 85% el context bloat en IDEs con limite de herramientas.'"
Sleep 500ms
Enter
Sleep 2.5s

# --- ESCENA 4: Recall Híbrido Sub-2ms y Concurrencia Multi-Instancia MCP-35 (38s - 50s) ---
Type "# 🔍 Búsqueda Híbrida BM25 + HNSW (Latencia p50 < 2ms, Sin Locks)"
Enter
Sleep 1s

Type "vanta-cli search --db ~/.vantadb --namespace agent/architecture --query 'locks multi-instancia' --top 2"
Sleep 500ms
Enter
Sleep 3s

# --- ESCENA 5: Inspección Visual de Memoria (50s - 60s) ---
Type "# 📊 Inspección de Estado de Memoria y Grafo de Conocimiento"
Enter
Sleep 1s

Type "vanta-cli memory inspect --db ~/.vantadb --stats"
Sleep 500ms
Enter
Sleep 4s

Type "# ✅ VantaDB: Un binario Rust. 0 Docker. 0 Latencia Cloud. Memoria Infinita."
Enter
Sleep 2s
