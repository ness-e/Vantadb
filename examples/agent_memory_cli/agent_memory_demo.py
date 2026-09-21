"""SHOW-04 — demo agente-con-memoria: prueba de aceptacion viva del MVP.

1 comando, 100% local, cero credenciales::

    python examples/agent_memory_cli/agent_memory_demo.py [--db <dir>] [--keep-db]

Dos sesiones del motor sobre el MISMO directorio persistido:
  sesion 1 -> guarda el dato (put + flush + close).
  sesion 2 -> reabre y lo recuerda (memory.get + assert de CONTENIDO).

Sin proveedor de embeddings (vectores explicitos fijos), sin red, sin timing:
los asserts comparan contenido exacto, nunca latencias.

Base conceptual: el mismo motor de memoria que exponen las tools MCP
``scene_*`` (lectura) e ``inject_context`` (escritura a thread); la demo usa
la API minima estable (``put``/``memory.get``/``search``) para maxima
determinacion con minima friccion.
"""

from __future__ import annotations

import argparse
import shutil
import sys
import tempfile

from vantadb import Client

NAMESPACE = "agent/demo-session"
KEY = "fact:mascota"
DATO_SESION_1 = "La mascota del usuario se llama Firulais."
# Vector fijo 3-d: el motor nunca llama a un proveedor externo.
VECTOR = [1.0, 0.0, 0.0]
METADATA = {"type": "preference", "sesion": 1}
MEMORY_LIMIT = 128 * 1024 * 1024


def _payload_of(record) -> str:
    try:
        return record["payload"]
    except (TypeError, KeyError):
        return record.payload


def sesion_1_guardar(db_dir: str) -> str:
    """Sesion 1: guarda el dato y cierra (flush + close = fin de sesion)."""
    db = Client(db_dir, memory_limit_bytes=MEMORY_LIMIT)
    try:
        db.put(NAMESPACE, KEY, DATO_SESION_1, metadata=METADATA, vector=VECTOR)
        db.flush()
    finally:
        db.close()
    print(f"[sesion 1] guardo en '{NAMESPACE}/{KEY}': {DATO_SESION_1}")
    print("SESION 1 OK")
    return DATO_SESION_1


def sesion_2_recordar(db_dir: str) -> str:
    """Sesion 2: reabre el MISMO dir y recuerda (assert mecanico de contenido)."""
    db = Client(db_dir, memory_limit_bytes=MEMORY_LIMIT)
    try:
        record = db.memory.get(NAMESPACE, KEY)
        assert record is not None, f"sesion 2 no recuerda: {NAMESPACE}/{KEY} ausente"
        recordada = _payload_of(record)
        assert DATO_SESION_1 in recordada, f"contenido distinto: {recordada!r}"
        assert "Firulais" in recordada, f"dato clave perdido: {recordada!r}"
    finally:
        db.close()
    print(f"[sesion 2] recuerdo de '{NAMESPACE}/{KEY}': {recordada}")
    print("SESION 2 OK")
    return recordada


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--db", default=None, help="dir persistido (default: temporal)")
    parser.add_argument("--keep-db", action="store_true", help="no borrar el dir al final")
    args = parser.parse_args(argv)

    tmp = args.db is None
    db_dir = args.db or tempfile.mkdtemp(prefix="show04-demo-")
    try:
        sesion_1_guardar(db_dir)
        recordada = sesion_2_recordar(db_dir)
        assert recordada == DATO_SESION_1
        print("MEMORIA VERIFICADA: la sesion 2 recuerda lo guardado en la sesion 1")
        return 0
    finally:
        if tmp and not args.keep_db:
            shutil.rmtree(db_dir, ignore_errors=True)


if __name__ == "__main__":
    sys.exit(main())
