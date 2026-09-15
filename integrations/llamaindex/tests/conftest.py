import sys, os
sys.path.insert(0, os.path.join(os.path.dirname(__file__), "..", ".."))
from vantadb_test_shim import _find84_fake_vantadb  # noqa: F401,E402 - FIND-84 shim temporal (FIND-94 lo elimina)
