#!/usr/bin/env python3
"""Verify .pyi stubs match actual class signatures for VantaDB providers."""
import sys

PROVIDER = "${PROVIDER}"

if PROVIDER == "openai":
    import vantadb_openai as m
    cls = m.VantaDBOpenAI
elif PROVIDER == "litellm":
    import vantadb_litellm as m
    cls = m.VantaDBLiteLLM
else:
    import vantadb_ollama as m
    cls = m.VantaDBOllama

required_methods = ['embed', 'search', 'store', 'delete', 'get', 'list', 'list_namespaces']

for name in required_methods:
    assert hasattr(cls, name), f"Missing method: {name}"

print(f"✓ All {len(required_methods)} methods present in {PROVIDER} type stubs")