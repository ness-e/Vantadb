"""FND-05 — Prototipo: context manager para VantaDB (Python).

Gap cubierto: PY-4 — ``VantaDB`` (sync) no tiene ``__enter__``/``__exit__``, así que
``with VantaDB(path) as db:`` no funciona y el usuario debe acordarse de ``close()``
(riesgo de DB sin flush/durabilidad).

Este archivo es un PROTOTIPO/ejemplo — NO modifica el SDK. Muestra el patrón
idiomático que el SDK debería exponer, implementado como subclase aquí.

Referente: ``open()`` / ``socket`` (context manager que LIBERA el recurso).
OJO: NO copiar el CM de ``sqlite3.Connection``, que hace commit/rollback y NO cierra
(fuente: docs.python.org/3/library/sqlite3.html) — para VantaDB ``close()`` es la
barrera de durabilidad (drena ops in-flight + flush), así que ``__exit__`` debe
llamar a ``close()``.

Ejecutar:
    python docs/examples/fnd05_python_context_manager.py
"""

from __future__ import annotations

from contextlib import AbstractContextManager
from typing import Any, Self

import vantadb_py
from vantadb_py import VantaDB


class ManagedVantaDB(VantaDB, AbstractContextManager["ManagedVantaDB"]):
    """VantaDB con soporte de ``with``.

    Este es el cambio mínimo que faltaría en el SDK real (ver recomendación al
    final del archivo): ``__enter__`` devuelve self, ``__exit__`` llama a
    ``close()`` — exactamente el patrón de ``open()``.

    Ejemplo de uso idiomático (target):

        with ManagedVantaDB(":memory:") as db:
            db.put("ns", "k", "payload")
        # close() ya se llamó — WAL flusheado, sin leak
    """

    def __enter__(self) -> Self:
        return self

    def __exit__(self, exc_type: Any, exc_value: Any, traceback: Any) -> None:
        # Durabilidad: drena operaciones in-flight y flushea antes de cerrar.
        self.close()


def demo() -> None:
    print("== FND-05 prototipo: with VantaDB(...) as db ==")

    # Uso idiomático actual (sin CM) — lo que el usuario debe escribir HOY:
    db = VantaDB(":memory:")
    try:
        db.put("agent/main", "k1", "hello", vector=[0.1, 0.2, 0.3])
        hits = db.search_memory("agent/main", [0.1, 0.2, 0.3], top_k=1)
        assert hits[0].key == "k1"
    finally:
        db.close()  # fácil de olvidar → WAL sin flush

    # Uso idiomático objetivo — el CM garantiza close():
    with ManagedVantaDB(":memory:") as db2:
        db2.put("agent/main", "k2", "world", vector=[0.1, 0.2, 0.3])
        hits = db2.search_memory("agent/main", [0.1, 0.2, 0.3], top_k=1)
        assert hits[0].key == "k2"
        print("  hit:", hits[0].key, "score:", round(hits[0].score, 4))
    # acá close() ya corrió — db2 está cerrado

    # El wrapper async ya es idiomático en el SDK (no es gap):
    import asyncio

    async def async_demo() -> None:
        async with vantadb_py.AsyncVantaDB(":memory:") as db3:
            await db3.put("agent/main", "k3", "async", vector=[0.1, 0.2, 0.3])
            hits = await db3.search_memory("agent/main", [0.1, 0.2, 0.3], top_k=1)
            assert hits[0].key == "k3"
            print("  async hit:", hits[0].key)

    asyncio.run(async_demo())
    print("OK — todos los asserts pasaron")


if __name__ == "__main__":
    demo()

# ── Recomendación de implementación (para el SDK real, NO aplicada aquí) ──
#
# En `vantadb-python/src/lib.rs`, dentro de `#[pymethods] impl VantaDB`, agregar:
#
#     /// Context manager support: `with VantaDB(path) as db:`.
#     fn __enter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> { slf }
#
#     fn __exit__(
#         &self,
#         py: Python<'_>,
#         _exc_type: PyObject,
#         _exc_value: PyObject,
#         _traceback: PyObject,
#     ) -> PyResult<()> {
#         self.close(py)   // close() ya existe y hace flush + drain
#     }
#
# Y actualizar `vantadb_py/__init__.pyi` y `vantadb_py/vantadb_py.pyi` con
# `def __enter__(self) -> VantaDB: ...` y `def __exit__(self, ...) -> None: ...`.