"""FIND-84 — shim de compat test-only (TEMPORAL, pendiente FIND-94).

Contexto: los 9 adapters construyen `vanta.VantaDB(...)` (API pre-0.5.0) pero
el SDK 0.5.0 solo expone `Client`/`connect` con metodos renombrados
(`search_memory`/`list_memory`/etc. ya no existen). Hasta que FIND-94 migre
el codigo PRODUCTIVO, este modulo provee un fake in-memory (`_FakeVantaDB`)
que implementa la superficie vieja. Alcance estrictamente tests: cero disco,
cero red, ningun `vectorstore.py` tocado (incluida la prohibicion FIND-69
sobre `integrations/dspy/vantadb_dspy/vectorstore.py`).

Uso: cada `integrations/<adapter>/tests/conftest.py` importa la fixture
(auto-use) de este modulo — importarla en el conftest la activa para toda
la suite del adapter, tanto en local (`pytest integrations/<a>/tests/`)
como en CI (`cd integrations/<a> && pytest tests/`, donde el conftest padre
NO carga por confcutdir=rootdir del adapter).
(vantadb_test_shim: el sys.path insert del conftest expone `integrations/`.)

Cuando FIND-94 complete la migracion prod, BORRAR este archivo + los 9
imports: las suites deben ejercitar el backend real via `tmp_path`
(fixtures ya migradas).
"""
from __future__ import annotations

from typing import Any, Dict, List, Optional

import pytest


class _FakeRecord:
    # Atributos que el codigo prod lee: key/id/payload/metadata/vector +
    # created_at_ms/updated_at_ms/version/node_id (langchain/llamaindex/mem0).
    __slots__ = ("key", "payload", "metadata", "vector",
                 "created_at_ms", "updated_at_ms", "version", "node_id")

    def __init__(self, key: str, payload: str,
                 metadata: Optional[Dict[str, Any]] = None,
                 vector: Optional[List[float]] = None):
        self.key = key
        self.payload = payload
        self.metadata = dict(metadata or {})
        self.vector = list(vector) if vector is not None else None
        self.created_at_ms = 0  # ponytail: determinista; fechas reales viven en metadata (crewai K_CREATED)
        self.updated_at_ms = 0
        self.version = 0
        self.node_id = key  # VantaMemoryRecord.node_id numerico en prod; key alcanza para roundtrip en tests

    @property
    def id(self) -> str:  # crewai borra por rec.id; langchain node_id=hit.id
        return self.key


class _FakeHit(_FakeRecord):
    __slots__ = ("score",)

    def __init__(self, key: str, payload: str,
                 metadata: Optional[Dict[str, Any]] = None,
                 vector: Optional[List[float]] = None,
                 score: float = 0.0):
        super().__init__(key, payload, metadata, vector)
        # CONVENCION DEL SHIM: score = similitud coseno (mayor = mejor,
        # 1.0 = identico). Divergencia documentada pendiente FIND-94: los
        # comentarios de langchain/llamaindex/mem0 describen el backend real
        # como distancia (menor = mejor, 0 = identico), mientras el test de
        # crewai (`test_search_returns_scored_match`, `score > 0.5` en match
        # exacto) solo pasa con similitud. Evidencia de motor no dirimente
        # (engine ordena DESC + filtra `>= min_score`, i.e. mayor = mejor).
        # FIND-94 (con backend real) debe fijar la convencion verdadera y
        # ajustar crewai-test o los 3 adapters; ninguna suite actual aserta
        # valores de distancia, asi que el shim no falsea ningun test verde.
        self.score = score


class _FakePage:
    """Pagina de list_memory: `.records` + `.next_cursor` + `dict(page)`."""

    def __init__(self, records: List[_FakeRecord], next_cursor: Optional[int]):
        self.records = records
        self.next_cursor = next_cursor

    def keys(self):  # soporte dict(page) (llamaindex vectorstore.py:179)
        return ("records", "next_cursor")

    def __getitem__(self, name: str):
        return getattr(self, name)


class _FakeVantaDB:
    """Sustituto in-memory de la clase `VantaDB` pre-0.5.0. Ignora `db_path`
    a proposito: los tests no deben escribir a disco (FIND-84)."""

    def __init__(self, db_path: str = "./vantadb_data", **kw: Any):
        self._db_path = db_path
        self._ns: Dict[str, Dict[str, _FakeRecord]] = {}

    # -- escrituras ------------------------------------------------------
    def put(self, namespace: str, key: str, text: str,
            metadata: Optional[Dict[str, Any]] = None,
            vector: Optional[List[float]] = None, **kw: Any) -> None:
        self._ns.setdefault(namespace, {})[str(key)] = _FakeRecord(
            str(key), text, metadata, vector)

    def put_batch(self, keys: List[str], vectors: List[Any],
                  payloads: List[str], metadatas: List[Dict[str, Any]],
                  namespace: str, **kw: Any) -> None:
        for k, v, p, m in zip(keys, vectors, payloads, metadatas):
            self.put(namespace, k, p, metadata=m, vector=list(v) if v is not None else None)

    def update_memory(self, namespace: str, key: str, text: str,
                      metadata: Optional[Dict[str, Any]] = None,
                      vector: Optional[List[float]] = None, **kw: Any) -> None:
        old = self._ns.get(namespace, {}).get(str(key))
        self.put(namespace, key, text,
                 metadata=dict(old.metadata) if metadata is None and old else metadata,
                 vector=list(old.vector) if vector is None and old and old.vector else vector)

    def delete_memory(self, namespace: str, key: str) -> bool:
        return self._ns.get(namespace, {}).pop(str(key), None) is not None

    def delete_namespace(self, namespace: str) -> None:
        self._ns.pop(namespace, None)

    # -- lecturas --------------------------------------------------------
    def get_memory(self, namespace: str, key: str) -> Optional[_FakeRecord]:
        return self._ns.get(namespace, {}).get(str(key))

    def list_namespaces(self) -> List[str]:
        return sorted(self._ns.keys())

    @staticmethod
    def _matches(rec: _FakeRecord, filters: Dict[str, Any]) -> bool:
        for fk, fv in (filters or {}).items():
            if rec.metadata.get(fk) != fv:
                return False
        return True

    def list_memory(self, namespace: str = "", *args: Any,
                    filters: Optional[Dict[str, Any]] = None,
                    limit: int = 100, cursor: Optional[int] = None,
                    **kw: Any) -> _FakePage:
        recs = [r for r in self._ns.get(namespace, {}).values()
                if self._matches(r, filters or {})]
        start = cursor or 0
        chunk = recs[start:start + limit]
        nxt = start + limit if start + limit < len(recs) else None
        return _FakePage(chunk, nxt)

    @staticmethod
    def _cosine(a: List[float], b: List[float]) -> float:
        dot = sum(x * y for x, y in zip(a, b))
        na = sum(x * x for x in a) ** 0.5
        nb = sum(y * y for y in b) ** 0.5
        if na == 0.0 or nb == 0.0:
            return 0.0
        return dot / (na * nb)

    @staticmethod
    def _is_vector(q: Any) -> bool:
        return isinstance(q, (list, tuple)) and all(isinstance(x, (int, float)) for x in q)

    def search_memory(self, namespace: str, query: Any, top_k: int = 10,
                      **kw: Any) -> List[_FakeHit]:
        recs = [r for r in self._ns.get(namespace, {}).values()
                if self._matches(r, kw.get("filters"))]
        text_q = kw.get("text_query")
        scored: List[_FakeHit] = []
        if text_q:
            q = str(text_q).casefold()
            recs = [r for r in recs if q in (r.payload or "").casefold()]
            scored = [_FakeHit(r.key, r.payload, r.metadata, r.vector, score=1.0) for r in recs]
        elif isinstance(query, str):
            q = query.casefold()
            recs = [r for r in recs if q in (r.payload or "").casefold()]
            scored = [_FakeHit(r.key, r.payload, r.metadata, r.vector, score=1.0) for r in recs]
        elif self._is_vector(query):
            # Query vectorial: solo records CON vector participan (como un
            # indice real); score = similitud coseno (1.0 = identico).
            # Convencion valida para ambas lecturas del repo: crewai ordena
            # desc y exige >0.5 en match exacto; mem0 `_normalize_score`
            # devuelve intacto todo valor en [0,1].
            for r in recs:
                if r.vector is None:
                    continue
                scored.append(_FakeHit(r.key, r.payload, r.metadata, r.vector,
                                       score=self._cosine(list(query), r.vector)))
            scored.sort(key=lambda h: h.score, reverse=True)
        else:
            scored = [_FakeHit(r.key, r.payload, r.metadata, r.vector, score=1.0) for r in recs]
        return scored[:top_k]


def _ensure_dspy_prediction() -> None:
    """Replica el fallback de `vantadb_dspy/vectorstore.py:10-19` cuando el
    import de `dspy` NO levanta ImportError sino que resuelve al directorio
    local `integrations/dspy/` (namespace package sin `Prediction`).

    Sin framework instalado, el `try: import dspy` del prod nunca falla y el
    fallback `dspy.Prediction` jamas se crea → `forward()` explota con
    AttributeError. Este parche test-only adjunta un `_Prediction`
    equivalente al modulo resuelto (sea namespace local o framework sin el
    atributo). Cero codigo prod tocado (prohibicion FIND-69)."""
    try:
        import dspy as _dspy_mod  # noqa: PLC0415
    except ImportError:
        return  # prod toma su propio fallback; nada que hacer
    if hasattr(_dspy_mod, "Prediction"):
        return

    class _Prediction:
        def __init__(self, passages=None):
            self.passages = passages or []

        def __getitem__(self, key):
            return getattr(self, key)

        def __setitem__(self, key, value):
            setattr(self, key, value)

    _dspy_mod.Prediction = _Prediction


_ensure_dspy_prediction()


def _patch(monkeypatch: pytest.MonkeyPatch) -> None:
    for mod_name in ("vantadb_py", "vantadb"):
        try:
            mod = __import__(mod_name)
        except ImportError:
            continue
        monkeypatch.setattr(mod, "VantaDB", _FakeVantaDB, raising=False)


@pytest.fixture(autouse=True)
def _find84_fake_vantadb(monkeypatch: pytest.MonkeyPatch):
    """Parchea `VantaDB` en cada test bajo `integrations/` (ver docstring)."""
    _patch(monkeypatch)
    yield
