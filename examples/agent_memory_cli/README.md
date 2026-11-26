# SHOW-04 — demo agente-con-memoria (prueba de aceptación viva del MVP)

Un agente **guarda un dato en la sesión 1 y lo recuerda en la sesión 2**,
con assert mecánico de contenido. 1 comando, 100% local, cero credenciales.

## 1 comando

```bash
python examples/agent_memory_cli/agent_memory_demo.py
```

Salida esperada:

```text
[sesion 1] guardo en 'agent/demo-session/fact:mascota': La mascota del usuario se llama Firulais.
SESION 1 OK
[sesion 2] recuerdo de 'agent/demo-session/fact:mascota': La mascota del usuario se llama Firulais.
SESION 2 OK
MEMORIA VERIFICADA: la sesion 2 recuerda lo guardado en la sesion 1
```

## Prueba e2e (determinista)

```bash
python -m pytest examples/agent_memory_cli/test_agent_memory_cli.py -v
```

3 tests: `get` entre sesiones + `search` con el hit esperado + 1-comando
orquesta ambas sesiones. Asserts de **contenido exacto**, nunca de timing.

## Por qué es determinista y local-first

- Vectores **explícitos y fijos** (`[1.0, 0.0, 0.0]`): el motor jamás llama a
  un proveedor de embeddings — sin red, sin ONNX, sin credenciales.
- Dos sesiones = dos ciclos `Client → flush → close → Client` sobre el **mismo
  directorio persistido** (temporal por defecto; `--db <dir> --keep-db` para
  inspeccionarlo).
- Cada test usa su propio `tempfile.mkdtemp`: sin estado cruzado, sin flakiness.

## Base conceptual (scenes + `inject_context`)

La demo usa la API mínima estable (`put` / `memory.get` / `search`) sobre el
mismo motor que las tools MCP `scene_*` (lectura) e `inject_context`
(escritura a thread, `thread_id` numérico). Plantilla de estructura:
`examples/python/agent_memory.py`; patrón de persistencia:
`vantadb-python/tests/test_sdk.py::test_memory_close_and_reopen`.
Una variante `scene_*` queda como slice futuro si se exige literalidad.
