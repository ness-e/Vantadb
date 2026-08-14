# Python SDK

> Movido desde `.opencode/AGENTS.md` — referencia on-demand. Consultar cuando toques `vantadb-python/`. Si editas, actualiza también el puntero en AGENTS.md.

```bash
:: Hermetic venv (tests MUST use this — never a global install)
dev-tools/setup_venv.ps1         # creates target/audit-venv + maturin build
target/audit-venv/Scripts/python -m pytest vantadb-python/tests/test_sdk.py -v

:: Editable install from source
pip install -e ./vantadb-python

:: PyPI name differs from import
pip install vantadb-py      # distribution
import vantadb_py            # module (underscore)
```

Built via `maturin` with PyO3. Requires Python ≥3.11.
