# SHOW-03 — demo RAG-sobre-PDFs 100% local (prototipo estrella)

Subís un PDF → chunk → embed → chat con citas **verificables**, con 1 comando,
100% local, cero credenciales.

## 1 comando

```bash
PYTHONUTF8=1 python examples/rag_pdf_chat/rag_pdf_demo.py
```

Con otro PDF y otra pregunta:

```bash
PYTHONUTF8=1 python examples/rag_pdf_chat/rag_pdf_demo.py --pdf mis-notas.pdf --pregunta "Que dice sobre RAG?"
```

Salida esperada:

```text
[ingesta] 3 chunks de 3 paginas desde 'fixture.pdf'
INGESTA OK
[pregunta] Que es VantaDB?
[cita] (p. 1) VantaDB es un motor de memoria embebido y persistente
...
CHAT OK
CITA VERIFICADA: la cita existe en el PDF
```

## Prueba e2e (determinista)

```bash
PYTHONUTF8=1 python -m pytest examples/rag_pdf_chat/test_rag_pdf_chat.py -v
```

3 tests: cita-substring-del-PDF + top-1 esperado + 1-comando orquesta todo.
Asserts de **contenido exacto**, nunca de timing ni validación visual.

## Por qué es determinista y local-first

- **Sin dependencias nuevas:** el extractor PDF es stdlib-only (regex sobre
  streams `BT…ET` con operadores `Tj`/`TJ`) y el fixture (`fixture.pdf`, 3
  páginas ASCII) lo escribe el propio demo — lo que se extrae es lo que se
  escribió, garantizado por test. No se implementó ningún ingestor en el core
  (eso es MGR-25, fuera de alcance a propósito).
- **Sin proveedor de embeddings:** vectores deterministas de 8 dims por hash
  de tokens — el motor jamás llama afuera, sin red, sin ONNX, sin credenciales.
- **Cita verificable por construcción:** cada chunk se guarda con proveniencia
  (`source`/`page`/`chunk`) y el chat aserta `cita in texto_pdf` en runtime;
  si el retrieval devolviera algo ajeno al PDF, el demo falla en vez de
  alucinar.
- Cada test usa su propio `tempfile.mkdtemp`: sin estado cruzado, sin flakiness.
  Base conceptual: `examples/python/agent_memory.py`; patrón demo 1-comando:
  `examples/agent_memory_cli/` (SHOW-04).
