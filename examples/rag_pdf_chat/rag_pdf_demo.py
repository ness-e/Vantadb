"""SHOW-03 — demo RAG-sobre-PDF 100% local: PDF -> chunk -> embed -> chat con citas.

1 comando, cero credenciales::

    python examples/rag_pdf_chat/rag_pdf_demo.py [--pdf <archivo>] [--db <dir>] [--keep-db]

Fases:
  ingesta -> extrae el texto del PDF (stdlib-only, sin dependencias),
      lo divide en chunks con overlap y lo guarda en VantaDB con
      vectores deterministicos (hash de tokens, sin proveedor externo).
  chat -> retrieval hibrido (keyword + vector) y devuelve la cita del
      chunk ganador con assert mecanico: la cita es substring del PDF.

Sin red, sin ONNX, sin timing: los asserts comparan contenido exacto.
"""

from __future__ import annotations

import argparse
import hashlib
import re
import shutil
import sys
import tempfile
from pathlib import Path

from vantadb import Client

NAMESPACE = "rag/pdf-demo"
CHUNK_SIZE = 500
CHUNK_OVERLAP = 50
EMBED_DIM = 8
MEMORY_LIMIT = 128 * 1024 * 1024

FIXTURE_NAME = "fixture.pdf"

# Texto del fixture (ASCII a proposito: el PDF minimo usa WinAnsi + escapes
# simples; la consola se lee con PYTHONUTF8=1 segun precedente FIND-114/116).
FIXTURE_PAGES: list[list[str]] = [
    [
        "VantaDB es un motor de memoria embebido y persistente",
        "para aplicaciones de IA con prioridad local.",
        "Guarda recuerdos como registros (namespace + key + payload)",
        "y los recupera con busqueda hibrida: BM25 lexical + HNSW",
        "vectorial combinados con RRF.",
    ],
    [
        "RAG significa recuperacion aumentada por generacion:",
        "primero se recuperan los fragmentos relevantes y despues",
        "se genera la respuesta citando esos fragmentos.",
        "Sin citas verificables no hay RAG, solo alucinacion con estilo.",
    ],
    [
        "El pipeline local es: extraer texto, dividir en chunks con",
        "solapamiento, calcular embeddings sin red y guardar cada chunk",
        "con su proveniencia (archivo, pagina, indice).",
        "El chat recupera el mejor chunk y lo devuelve como cita.",
    ],
]

DEFAULT_QUESTION = "Que es VantaDB?"


def _pdf_escape(line: str) -> str:
    return line.replace("\\", "\\\\").replace("(", "\\(").replace(")", "\\)")


def write_fixture_pdf(path: str | Path) -> Path:
    """Escribe un PDF minimo valido (3 paginas, Helvetica, texto ASCII)."""
    path = Path(path)
    objs: list[bytes] = []
    objs.append(b"<< /Type /Catalog /Pages 2 0 R >>")
    page_refs = " ".join(f"{4 + i * 2} 0 R" for i in range(len(FIXTURE_PAGES)))
    objs.append(f"<< /Type /Pages /Kids [{page_refs}] /Count {len(FIXTURE_PAGES)} >>".encode("ascii"))
    objs.append(b"<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>")
    for i, lines in enumerate(FIXTURE_PAGES):
        content = ["BT /F1 12 Tf 72 750 Td 14 TL"]
        content += [f"({_pdf_escape(line)}) Tj T*" for line in lines]
        content.append("ET")
        stream = "\n".join(content).encode("ascii")
        objs.append(
            f"<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] "
            f"/Resources << /Font << /F1 3 0 R >> >> /Contents {4 + i * 2 + 1} 0 R >>".encode("ascii")
        )
        objs.append(f"<< /Length {len(stream)} >>\nstream\n".encode("ascii") + stream + b"\nendstream")
    out = bytearray(b"%PDF-1.4\n")
    offsets = [0]
    for n, body in enumerate(objs, start=1):
        offsets.append(len(out))
        out += f"{n} 0 obj\n".encode("ascii") + body + b"\nendobj\n"
    xref_at = len(out)
    out += f"xref\n0 {len(objs) + 1}\n".encode("ascii")
    out += b"0000000000 65535 f \n"
    for off in offsets[1:]:
        out += f"{off:010d} 00000 n \n".encode("ascii")
    out += (
        f"trailer\n<< /Size {len(objs) + 1} /Root 1 0 R >>\n"
        f"startxref\n{xref_at}\n%%EOF\n"
    ).encode("ascii")
    path.write_bytes(bytes(out))
    return path


def ensure_fixture(pdf_arg: str | None) -> Path:
    """Devuelve el PDF a usar: el indicado con --pdf o el fixture (lo crea si falta)."""
    if pdf_arg:
        pdf = Path(pdf_arg)
        if not pdf.is_file():
            raise SystemExit(f"PDF no encontrado: {pdf}")
        return pdf
    pdf = Path(__file__).with_name(FIXTURE_NAME)
    if not pdf.is_file():
        write_fixture_pdf(pdf)
    return pdf


_TJ_RE = re.compile(rb"\((?:\\.|[^()\\])*\)\s*Tj")
_TJ_ARRAY_RE = re.compile(rb"\[(.*?)\]\s*TJ", re.DOTALL)
_STR_RE = re.compile(rb"\((?:\\.|[^()\\])*\)")
_STREAM_RE = re.compile(rb"stream\r?\n(.*?)endstream", re.DOTALL)


def _unescape(raw: bytes) -> str:
    out = bytearray()
    it = iter(range(len(raw)))
    for i in it:
        byte = raw[i : i + 1]
        if byte == b"\\" and i + 1 < len(raw):
            nxt = raw[i + 1 : i + 2]
            out += {b"n": b"\n", b"r": b"\r", b"t": b"\t"}.get(nxt, nxt)
            next(it)
        else:
            out += byte
    return out.decode("latin-1")


def extract_pdf_pages(pdf: str | Path) -> list[str]:
    """Extrae el texto de cada stream de contenido, en orden (una pagina por stream)."""
    data = Path(pdf).read_bytes()
    pages: list[str] = []
    for stream in _STREAM_RE.findall(data):
        lines: list[str] = []
        tmp = _TJ_ARRAY_RE.sub(lambda m: b" ".join(_STR_RE.findall(m.group(1))), stream)
        for match in _TJ_RE.findall(tmp):
            inner = match[: match.rfind(b")")][1:]
            lines.append(_unescape(inner))
        text = "\n".join(line for line in lines if line.strip())
        if text:
            pages.append(text)
    if not pages:
        raise ValueError(f"sin texto extraible en {pdf}")
    return pages


def chunk_pages(pages: list[str], size: int = CHUNK_SIZE, overlap: int = CHUNK_OVERLAP) -> list[dict]:
    """Divide cada pagina en chunks con overlap; cada chunk lleva su proveniencia."""
    chunks: list[dict] = []
    for page_no, page in enumerate(pages, start=1):
        step = size - overlap
        for start in range(0, len(page), step):
            text = page[start : start + size]
            if not text.strip():
                continue
            chunks.append({"page": page_no, "index": len(chunks), "text": text})
            if start + size >= len(page):
                break
    return chunks


def embed(text: str, dim: int = EMBED_DIM) -> list[float]:
    """Vector determinista por hash de tokens (100% local, sin proveedor)."""
    vec = [0.0] * dim
    for token in re.findall(r"[a-z0-9]+", text.lower()):
        vec[int.from_bytes(hashlib.sha256(token.encode()).digest()[:4], "big") % dim] += 1.0
    norm = sum(value * value for value in vec) ** 0.5
    return [value / norm for value in vec] if norm else vec


def ingest_chunks(db: Client, namespace: str, chunks: list[dict], source: str = FIXTURE_NAME) -> int:
    """Guarda cada chunk con vector explicito + metadata de proveniencia."""
    for chunk in chunks:
        db.put(
            namespace,
            f"chunk-{chunk['index']:03d}",
            chunk["text"],
            metadata={"source": source, "page": chunk["page"], "chunk": chunk["index"]},
            vector=embed(chunk["text"]),
        )
    db.flush()
    return len(chunks)


def chat(db: Client, namespace: str, question: str, texto_pdf: str, quiet: bool = False) -> str:
    """Recupera el mejor chunk y lo devuelve como cita (assert mecanico incluido)."""
    hits = db.search(namespace, embed(question), text_query=question, top_k=3)
    assert hits, f"sin resultados para: {question!r}"
    cita = hits[0].payload
    assert cita in texto_pdf, f"cita no verificable contra el PDF: {cita!r}"
    if not quiet:
        meta = dict(hits[0].metadata)
        print(f"[pregunta] {question}")
        print(f"[cita] (p. {meta.get('page')}) {cita}")
    return cita


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--pdf", default=None, help="PDF a consultar (default: fixture)")
    parser.add_argument("--db", default=None, help="dir persistido (default: temporal)")
    parser.add_argument("--keep-db", action="store_true", help="no borrar el dir al final")
    parser.add_argument("--pregunta", default=DEFAULT_QUESTION, help="pregunta del chat")
    args = parser.parse_args(argv)

    pdf = ensure_fixture(args.pdf)
    pages = extract_pdf_pages(pdf)
    texto_pdf = "\n".join(pages)
    chunks = chunk_pages(pages)

    tmp = args.db is None
    db_dir = args.db or tempfile.mkdtemp(prefix="show03-demo-")
    try:
        db = Client(db_dir, memory_limit_bytes=MEMORY_LIMIT)
        try:
            total = ingest_chunks(db, NAMESPACE, chunks, source=pdf.name)
            print(f"[ingesta] {total} chunks de {len(pages)} paginas desde '{pdf.name}'")
            print("INGESTA OK")
            cita = chat(db, NAMESPACE, args.pregunta, texto_pdf)
            assert "VantaDB" in cita or args.pregunta != DEFAULT_QUESTION
            print("CHAT OK")
            print("CITA VERIFICADA: la cita existe en el PDF")
        finally:
            db.close()
        return 0
    finally:
        if tmp and not args.keep_db:
            shutil.rmtree(db_dir, ignore_errors=True)


if __name__ == "__main__":
    sys.exit(main())
