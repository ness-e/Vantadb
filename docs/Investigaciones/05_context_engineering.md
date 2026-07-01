# Investigación 5 — Context Engineering y Mercado

> Investigación de mercado sobre "Context Engineering" y el problema de memoria persistente en agentes de IA.

## Hallazgos Críticos

1. **El problema tiene nombre**: "Context Engineering" (Tobi Lutke, Shopify CEO, 2025)
2. **Claude Code es el caso más grande**: `claude-mem` (SQLite) llegó a 89K GitHub stars
3. **AnythingLLM usa LanceDB**: Reemplazable por VantaDB con hybrid search
4. **CrewAI tiene el problema más agudo**: Sin aislamiento por usuario en producción
5. **LangGraph pide exactamente VantaDB**: Misma API en dev y prod

## Canal de Distribución Más Eficiente

**MCP Server.** Cursor, Windsurf, Claude Code, OpenCode y Cline soportan MCP. Un solo servidor MCP de VantaDB funciona en todos los IDEs simultáneamente.
