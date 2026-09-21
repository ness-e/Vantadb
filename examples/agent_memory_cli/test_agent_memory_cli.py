"""SHOW-04 — e2e: el agente recuerda entre 2 sesiones (determinista, local).

Contrato:
  sesion 1 guarda un dato -> close/flush -> sesion 2 reabre el MISMO db dir
  y lo recuerda (assert de CONTENIDO, nunca de timing).
Todo con vectores explicitos fijos: sin proveedor externo, sin credenciales.
"""

import shutil
import subprocess
import sys
import tempfile
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))

from agent_memory_demo import (
    DATO_SESION_1,
    NAMESPACE,
    sesion_1_guardar,
    sesion_2_recordar,
)

DEMO = Path(__file__).with_name("agent_memory_demo.py")


def _tmp_db():
    return tempfile.mkdtemp(prefix="show04-")


def test_sesion2_recuerda_lo_guardado_en_sesion1():
    # Arrange: DB temporal aislada (una por test, sin flakiness cruzada).
    db_dir = _tmp_db()
    try:
        # Act sesion 1: guarda y cierra (flush+close dentro).
        sesion_1_guardar(db_dir)

        # Act sesion 2: reabre el MISMO dir y recuerda.
        recordada = sesion_2_recordar(db_dir)

        # Assert: contenido exacto, no timing.
        assert DATO_SESION_1 in recordada, f"sesion 2 no recuerda: {recordada!r}"
        assert "Firulais" in recordada
    finally:
        shutil.rmtree(db_dir, ignore_errors=True)


def test_search_devuelve_el_hit_esperado():
    db_dir = _tmp_db()
    try:
        sesion_1_guardar(db_dir)

        # Act: busqueda hibrida con el MISMO vector fijo del guardado.
        from vantadb import Client

        db = Client(db_dir)
        try:
            hits = db.search(
                NAMESPACE, [1.0, 0.0, 0.0], text_query="mascota", top_k=3
            )
        finally:
            db.close()

        # Assert: el hit esperado existe y su payload es el dato.
        keys = [h.key for h in hits]
        assert "fact:mascota" in keys, f"hit esperado ausente: {keys}"
        hit = next(h for h in hits if h.key == "fact:mascota")
        assert hit.payload == DATO_SESION_1
    finally:
        shutil.rmtree(db_dir, ignore_errors=True)


def test_un_comando_orquesta_ambas_sesiones():
    # El contrato exige 1 comando: la demo imprime ambas sesiones OK.
    proc = subprocess.run(
        [sys.executable, str(DEMO)],
        capture_output=True,
        text=True,
        timeout=180,
    )
    assert proc.returncode == 0, f"demo fallo:\n{proc.stdout}\n{proc.stderr}"
    assert "SESION 1 OK" in proc.stdout
    assert "SESION 2 OK" in proc.stdout
    assert "MEMORIA VERIFICADA" in proc.stdout
