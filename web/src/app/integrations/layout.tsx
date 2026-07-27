import type { Metadata } from "next";

export const metadata: Metadata = {
  title: "Ecosystem & Integrations · VantaDB — OpenAI, Ollama, CrewAI, MCP",
  description:
    "VantaDB se integra con OpenAI, Ollama, CrewAI, Haystack, DSPy, LiteLLM, Mem0, Letta y MCP. Native, experimental y coming soon.",
  openGraph: {
    title: "Ecosystem & Integrations · VantaDB",
    description:
      "Grid de integraciones: OpenAI, Ollama, CrewAI, Haystack, DSPy, LiteLLM, Mem0, Letta, MCP.",
    url: "https://vantadb.dev/integrations",
    siteName: "VantaDB",
    type: "website",
  },
  alternates: {
    canonical: "https://vantadb.dev/integrations",
  },
};

export default function Layout({ children }: { children: React.ReactNode }) {
  return children;
}
