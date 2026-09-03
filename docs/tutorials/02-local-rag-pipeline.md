---
title: "Local RAG Pipeline with VantaDB + Ollama"
status: active
tags: [vantadb, tutorial, guide, rag, ollama]
last_reviewed: 2026-08-02
aliases: []
---

# Local RAG Pipeline with VantaDB + Ollama

Retrieval-Augmented Generation (RAG) lets you ask natural language questions over your own documents — without sending data to a third-party API. This tutorial builds a **fully local RAG pipeline** using:

- **VantaDB** — embedded vector storage and search
- **Ollama** — local LLM + embedding model
- Your own documents (PDF text, markdown files, or plain text)

By the end you'll have a script that ingests documents and answers questions about them, all running on your machine.

## Prerequisites

```bash
pip install vantadb-py ollama pypdf
```

Install [Ollama](https://ollama.ai) and pull a model:

```bash
ollama pull llama3.2:3b
ollama pull nomic-embed-text
```

Skip the local installs entirely: [`docker-compose.yml`](../../docker-compose.yml) at the repo root runs VantaDB + Ollama in containers with `docker compose up -d --build`.

## 1. Connect VantaDB and set up embeddings

```python
from vantadb_py import VantaDB
import ollama

db = VantaDB("rag-knowledge.db")

def embed(text: str) -> list[float]:
    """Embed text locally with Ollama."""
    return ollama.embeddings(model="nomic-embed-text", prompt=text)["embedding"]
```

VantaDB is **BYO-vector**: you compute the embedding (OpenAI, Ollama, LiteLLM, or any model) and pass it to `put()` / `search_memory()`. Nothing leaves your machine.

## 2. Ingest a document (chunk + embed + store)

Let's ingest a text file by splitting it into overlapping chunks:

```python
# vanta-skip: requires Ollama running (embed()) and a knowledge-base.md fixture
def chunk_text(text: str, chunk_size: int = 512, overlap: int = 64) -> list[str]:
    """Split text into overlapping chunks at word boundaries."""
    words = text.split()
    chunks = []
    start = 0
    while start < len(words):
        end = start + chunk_size
        chunk = " ".join(words[start:end])
        chunks.append(chunk)
        start += chunk_size - overlap
    return chunks

def ingest_file(filepath: str, source: str = ""):
    path = Path(filepath)
    raw_text = path.read_text(encoding="utf-8")

    chunks = chunk_text(raw_text)
    print(f"Ingesting {len(chunks)} chunks from {path.name}...")

    for i, chunk in enumerate(chunks):
        db.put(
            "documents",                     # namespace
            f"{path.stem}-{i}",              # key
            chunk,                           # payload (BM25-indexed text)
            metadata={
                "source": source or path.name,
                "chunk_index": i,
                "total_chunks": len(chunks),
            },
            vector=embed(chunk),
        )

    print(f"  ✓ Stored {len(chunks)} chunks")

# Example: ingest a markdown file
from pathlib import Path
ingest_file("knowledge-base.md", source="product-docs")
```

For PDFs, use `pypdf`:

```python
from pypdf import PdfReader

def ingest_pdf(filepath: str):
    reader = PdfReader(filepath)
    full_text = "\n".join(page.extract_text() for page in reader.pages if page.extract_text())
    chunks = chunk_text(full_text)

    print(f"Ingesting {len(chunks)} chunks from PDF: {Path(filepath).name}")
    for i, chunk in enumerate(chunks):
        db.put(
            "documents",
            f"{Path(filepath).stem}-{i}",
            chunk,
            metadata={"source": Path(filepath).name, "chunk_index": i, "total_chunks": len(chunks)},
            vector=embed(chunk),
        )
    print(f"  ✓ Stored {len(chunks)} chunks")

ingest_pdf("manual.pdf")
```

> **Batch loading:** for thousands of chunks, use `put_batch()` with `keys=`, `vectors=`, `payloads=`, `metadatas=`, and `namespace=` — it is up to ~5x faster than sequential `put()` calls (Rayon parallelism):
>
> ```python
> db.put_batch(
>     None,                                  # entries is a required positional arg
>     keys=[f"{stem}-{i}" for i in range(len(chunks))],
>     vectors=[embed(c) for c in chunks],
>     payloads=chunks,
>     metadatas=[{"source": name, "chunk_index": str(i), "total_chunks": str(len(chunks))} for i in range(len(chunks))],
>     namespace="documents",
> )
> ```

## 3. Query the knowledge base

```python
# vanta-skip: requires Ollama running (embed() calls the local Ollama service)
def query_knowledge_base(question: str, top_k: int = 4):
    """Search for the most relevant document chunks."""
    return db.search_memory("documents", embed(question), top_k=top_k)

question = "How do I configure the database connection?"
results = query_knowledge_base(question)

print(f"\nTop results for: \"{question}\"\n")
for r in results:
    print(f"  [{r.score:.3f}] (source: {r.metadata['source']}, "
          f"chunk {r.metadata['chunk_index']+1}/{r.metadata['total_chunks']})")
    print(f"  {r.payload[:200]}\n")
```

## 4. Generate an answer with Ollama

Now feed the retrieved chunks as context to a local LLM:

```python
# vanta-skip: requires Ollama running (embed() calls the local Ollama service)
def ask(question: str, top_k: int = 4) -> str:
    # 1. Retrieve
    results = db.search_memory("documents", embed(question), top_k=top_k)

    if not results:
        return "No relevant documents found."

    # 2. Build context
    context = "\n\n".join(
        f"--- Document chunk (relevance: {r.score:.2f}) ---\n{r.payload}"
        for r in results
    )

    # 3. Generate
    prompt = f"""You are a helpful assistant. Answer the question based solely on the provided context. If the context doesn't contain the answer, say "I don't have enough information."

Context:
{context}

Question: {question}

Answer:"""

    response = ollama.chat(
        model="llama3.2:3b",
        messages=[{"role": "user", "content": prompt}],
        options={"temperature": 0.1},
    )
    return response["message"]["content"]

answer = ask("What database options are available?")
print(f"Answer: {answer}")
```

## 5. Full pipeline script

Here's the complete, copy-paste-ready script:

```python
#!/usr/bin/env python3
"""
Local RAG Pipeline — VantaDB + Ollama

Usage:
    python rag_pipeline.py ingest <file>     # Ingest a document
    python rag_pipeline.py query <question>  # Ask a question
"""

import sys
import ollama
from pathlib import Path
from vantadb_py import VantaDB

# ── Setup ──────────────────────────────────────────────────────────────

db = VantaDB("rag-knowledge.db")

def embed(text: str) -> list[float]:
    return ollama.embeddings(model="nomic-embed-text", prompt=text)["embedding"]

# ── Chunking ───────────────────────────────────────────────────────────

def chunk_text(text: str, chunk_size: int = 512, overlap: int = 64) -> list[str]:
    words = text.split()
    chunks, start = [], 0
    while start < len(words):
        end = start + chunk_size
        chunks.append(" ".join(words[start:end]))
        start += chunk_size - overlap
    return chunks

# ── Ingest ─────────────────────────────────────────────────────────────

def ingest(path_str: str):
    path = Path(path_str)
    if not path.exists():
        print(f"File not found: {path}")
        return

    if path.suffix.lower() == ".pdf":
        from pypdf import PdfReader
        reader = PdfReader(str(path))
        full_text = "\n".join(p.extract_text() for p in reader.pages if p.extract_text())
    else:
        full_text = path.read_text(encoding="utf-8")

    chunks = chunk_text(full_text)
    print(f"Ingesting {len(chunks)} chunks from {path.name}...")

    for i, chunk in enumerate(chunks):
        db.put(
            "documents",
            f"{path.stem}-{i}",
            chunk,
            metadata={"source": path.name, "chunk_index": i, "total_chunks": len(chunks)},
            vector=embed(chunk),
        )
    print("Done.")

# ── Query ──────────────────────────────────────────────────────────────

def query(question: str, top_k: int = 4) -> str:
    results = db.search_memory("documents", embed(question), top_k=top_k)
    if not results:
        return "No relevant documents found."

    context = "\n\n".join(
        f"--- Chunk (relevance: {r.score:.2f}) ---\n{r.payload}"
        for r in results
    )

    prompt = f"""Answer the question based solely on the context below.

Context:
{context}

Question: {question}

Answer:"""

    response = ollama.chat(
        model="llama3.2:3b",
        messages=[{"role": "user", "content": prompt}],
        options={"temperature": 0.1},
    )
    return response["message"]["content"]

# ── CLI ────────────────────────────────────────────────────────────────

if __name__ == "__main__":
    if len(sys.argv) < 3:
        print("Usage: python rag_pipeline.py ingest <file> | query <question>")
        sys.exit(1)

    command = sys.argv[1]
    arg = sys.argv[2]

    if command == "ingest":
        ingest(arg)
    elif command == "query":
        print(query(arg))
    else:
        print(f"Unknown command: {command}")
```

Run it:

```bash
python rag_pipeline.py ingest manual.pdf
python rag_pipeline.py query "How do I reset the admin password?"
```

## How it works

```
┌──────────────┐    ┌──────────────┐    ┌──────────────┐
│  Document     │───▶│  Chunk +     │───▶│  VantaDB     │
│  (PDF/MD/txt) │    │  Embed       │    │  Store       │
└──────────────┘    └──────────────┘    └──────────────┘
                                              │
                 ┌────────────────────────────┘
                 ▼
         ┌──────────────┐    ┌──────────────┐    ┌──────────────┐
         │  User query   │───▶│  Embed +     │───▶│  Context +   │
         │              │    │  Similarity  │    │  LLM (local) │
         └──────────────┘    └──────────────┘    └──────────────┘
                                                       │
                                                       ▼
                                              ┌──────────────┐
                                              │  Answer      │
                                              └──────────────┘
```

## Going further

- **Hybrid search:** Pass `text_query="..."` to `search_memory()` for better keyword matching on code or product names
- **Metadata filtering:** Tag chunks by chapter or section and filter with `filters={"source": "manual.pdf"}`
- **Document streaming:** Ingest large documents incrementally without loading everything into memory
- **WASM deployment:** This same pipeline runs in the browser — embed RAG directly in a static site

---

**Key takeaway:** A fully local RAG pipeline with VantaDB + Ollama requires ~60 lines of Python. No API keys, no cloud dependencies, no GPU required.
