"""SHOW-03 — e2e: RAG-sobre-PDF 100% local con cita verificable.

Contrato:
  PDF -> chunk -> embed -> chat con citas (assert mecanico: la cita existe
  en el texto extraido del PDF, nunca validacion visual).
Todo stdlib + `vantadb.Client` con vectores deterministicos: sin proveedor
externo, sin credenciales, sin red.
"""

import shutil
import subprocess
import sys
import tempfile
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))

from rag_pdf_demo import (
    NAMESPACE,
    chat,
    chunk_pages,
    embed,
    ensure_fixture,
    extract_pdf_pages,
    ingest_chunks,
)

DEMO = Path(__file__).with_name("rag_pdf_demo.py")


def _tmp_db():
    return tempfile.mkdtemp(prefix="show03-")


def test_cita_existe_en_el_pdf():
    # Arrange: fixture + ingesta en DB aislada.
    from vantadb import Client

    pdf = ensure_fixture(None)
    pages = extract_pdf_pages(pdf)
    texto_pdf = "\n".join(pages)
    chunks = chunk_pages(pages)
    db_dir = _tmp_db()
    try:
        db = Client(db_dir)
        try:
            ingest_chunks(db, NAMESPACE, chunks)
            # Act: pregunta cuya respuesta esta en la pagina 1.
            cita = chat(db, NAMESPACE, "Que es VantaDB?", texto_pdf, quiet=True)
        finally:
            db.close()
    finally:
        shutil.rmtree(db_dir, ignore_errors=True)

    # Assert mecanico: la cita es substring del PDF (no validacion visual).
    assert cita in texto_pdf, f"cita no verificable contra el PDF: {cita!r}"
    assert "VantaDB" in cita


def test_chat_recupera_el_chunk_esperado():
    # Arrange: ingesta con vectores deterministicos.
    from vantadb import Client

    pdf = ensure_fixture(None)
    pages = extract_pdf_pages(pdf)
    chunks = chunk_pages(pages)
    db_dir = _tmp_db()
    try:
        db = Client(db_dir)
        try:
            ingest_chunks(db, NAMESPACE, chunks)
            # Act: retrieval hibrido con el MISMO embed determinista.
            hits = db.search(
                NAMESPACE, embed("Que es VantaDB?"),
                text_query="VantaDB", top_k=3,
            )
        finally:
            db.close()
    finally:
        shutil.rmtree(db_dir, ignore_errors=True)

    # Assert: el top-1 habla de VantaDB (pagina 1 del fixture).
    assert hits, "sin hits para 'VantaDB'"
    assert "VantaDB" in hits[0].payload, f"hit inesperado: {hits[0].payload!r}"


def test_un_comando_orquesta_todo():
    # El contrato exige 1 comando: la demo imprime ingesta + chat + cita.
    proc = subprocess.run(
        [sys.executable, str(DEMO)],
        capture_output=True,
        text=True,
        timeout=180,
    )
    assert proc.returncode == 0, f"demo fallo:\n{proc.stdout}\n{proc.stderr}"
    assert "INGESTA OK" in proc.stdout
    assert "CHAT OK" in proc.stdout
    assert "CITA VERIFICADA" in proc.stdout
