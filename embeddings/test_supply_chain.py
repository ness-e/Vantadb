#!/usr/bin/env python3
"""
Offline supply-chain security tests for embeddings/{verify.py,download.py,manifest.json,manifest.lock}.
FIND-641 / T55: verify 100% fail-closed behavior with temporary fixtures (no network).
"""
from __future__ import annotations

import hashlib
import json
import pathlib
import sys
from types import SimpleNamespace

import pytest

HERE = pathlib.Path(__file__).resolve().parent
if str(HERE) not in sys.path:
    sys.path.insert(0, str(HERE))

import download  # noqa: E402
import verify  # noqa: E402


def _sha256_bytes(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def _minimal_manifest(models: list[dict], default: str = "m1") -> dict:
    return {"version": 1, "default": default, "models": models}


def _sample_models() -> list[dict]:
    models = []
    # Keep 3 en, 3 es, 3 combined + qwen3 exception to satisfy check_structure baseline
    specs = [
        ("m-en-1", "org/m-en-1", "1111111", 384, "en", "onnx/model.onnx"),
        ("m-en-2", "org/m-en-2", "2222222", 384, "en", "onnx/model.onnx"),
        ("m-en-3", "org/m-en-3", "3333333", 768, "en", "onnx/model.onnx"),
        ("m-es-1", "org/m-es-1", "4444444", 768, "es", "onnx/model.onnx"),
        ("m-es-2", "org/m-es-2", "5555555", 384, "es", "onnx/model.onnx"),
        ("m-es-3", "org/m-es-3", "6666666", 512, "es", "onnx/model.onnx"),
        ("m-co-1", "org/m-co-1", "7777777", 384, "combined", "onnx/model.onnx"),
        ("m-co-2", "org/m-co-2", "8888888", 1024, "combined", "onnx/model.onnx"),
        ("m-co-3", "org/m-co-3", "9999999", 768, "combined", "onnx/model.onnx"),
    ]
    for mid, repo, rev, dim, grp, onnx in specs:
        models.append(
            {
                "id": mid,
                "repo": repo,
                "rev": rev,
                "dim": dim,
                "size_onnx_mb": 100,
                "size_hf_mb": 100,
                "onnx": onnx,
                "langs": ["en"],
                "license": "MIT",
                "group": grp,
            }
        )
    models.append(
        {
            "id": "qwen3-embedding-8b",
            "repo": "Qwen/Qwen3-Embedding-8B",
            "rev": "1d8ad4c",
            "dim": 4096,
            "size_onnx_mb": None,
            "size_hf_mb": 16000,
            "onnx": None,
            "langs": ["en"],
            "license": "Apache-2.0",
            "group": "combined",
            "exception": ">3GB",
        }
    )
    return models


def _lock_for_models(models: list[dict], onnx_hash: str, tok_hash: str) -> dict:
    return {
        "version": 1,
        "models": [
            {
                "id": m["id"],
                "repo": m["repo"],
                "rev": m["rev"],
                "sha256": {
                    m["onnx"]: onnx_hash,
                    "tokenizer.json": tok_hash,
                },
            }
            for m in models
            if m.get("onnx") is not None
        ],
    }


def test_repo_manifest_and_lock_completeness() -> None:
    raw_manifest = (HERE / "manifest.json").read_text(encoding="utf-8")
    assert raw_manifest.endswith("\n"), "manifest.json must end with trailing newline"
    manifest = json.loads(raw_manifest)
    lock = json.loads((HERE / "manifest.lock").read_text(encoding="utf-8"))

    onnx_models = [m for m in manifest["models"] if m.get("onnx") is not None]
    assert len(onnx_models) == 9
    lock_by_id = {e["id"]: e for e in lock["models"]}
    assert set(lock_by_id.keys()) == {m["id"] for m in onnx_models}
    for m in onnx_models:
        entry = lock_by_id[m["id"]]
        assert entry["repo"] == m["repo"]
        assert entry["rev"] == m["rev"]
        assert m["onnx"] in entry["sha256"]
        assert "tokenizer.json" in entry["sha256"]


def test_verify_load_lock_rejects_missing_and_corrupt_json(tmp_path: pathlib.Path, monkeypatch: pytest.MonkeyPatch) -> None:
    missing_lock = tmp_path / "missing.lock"
    monkeypatch.setattr(verify, "LOCK", missing_lock)
    monkeypatch.setattr(verify, "_lock_cache", None)
    with pytest.raises(FileNotFoundError):
        verify.load_lock()

    corrupt_lock = tmp_path / "corrupt.lock"
    corrupt_lock.write_text("{not valid json", encoding="utf-8")
    monkeypatch.setattr(verify, "LOCK", corrupt_lock)
    monkeypatch.setattr(verify, "_lock_cache", None)
    with pytest.raises(ValueError):
        verify.load_lock()


def test_verify_locked_hashes_rejects_missing_lock_entry(tmp_path: pathlib.Path, monkeypatch: pytest.MonkeyPatch) -> None:
    lock_file = tmp_path / "manifest.lock"
    lock_file.write_text(json.dumps({"version": 1, "models": []}), encoding="utf-8")
    monkeypatch.setattr(verify, "LOCK", lock_file)
    monkeypatch.setattr(verify, "_lock_cache", None)

    model_dir = tmp_path / "models" / "unlisted-model"
    model_dir.mkdir(parents=True)
    (model_dir / "onnx").mkdir()
    (model_dir / "onnx" / "model.onnx").write_bytes(b"fake-onnx")
    (model_dir / "tokenizer.json").write_bytes(b"{}")

    assert verify.verify_locked_hashes("unlisted-model", model_dir) is False


def test_verify_locked_hashes_rejects_missing_file_and_hash_mismatch(tmp_path: pathlib.Path, monkeypatch: pytest.MonkeyPatch) -> None:
    onnx_bytes = b"valid-onnx-weights"
    tok_bytes = b'{"version":"1.0"}'
    lock_file = tmp_path / "manifest.lock"
    lock_file.write_text(
        json.dumps(
            {
                "version": 1,
                "models": [
                    {
                        "id": "m1",
                        "repo": "org/m1",
                        "rev": "1234567",
                        "sha256": {
                            "onnx/model.onnx": _sha256_bytes(onnx_bytes),
                            "tokenizer.json": _sha256_bytes(tok_bytes),
                        },
                    }
                ],
            }
        ),
        encoding="utf-8",
    )
    monkeypatch.setattr(verify, "LOCK", lock_file)
    monkeypatch.setattr(verify, "_lock_cache", None)

    model_dir = tmp_path / "models" / "m1"
    (model_dir / "onnx").mkdir(parents=True)
    (model_dir / "onnx" / "model.onnx").write_bytes(onnx_bytes)
    # tokenizer.json missing -> fail-closed
    assert verify.verify_locked_hashes("m1", model_dir) is False

    # Now add tampered tokenizer.json -> fail-closed
    (model_dir / "tokenizer.json").write_bytes(b'{"tampered":true}')
    assert verify.verify_locked_hashes("m1", model_dir) is False

    # Restore valid tokenizer.json -> passes
    (model_dir / "tokenizer.json").write_bytes(tok_bytes)
    assert verify.verify_locked_hashes("m1", model_dir) is True


def test_verify_model_does_not_fallback_to_arbitrary_rglob_onnx(tmp_path: pathlib.Path, monkeypatch: pytest.MonkeyPatch) -> None:
    onnx_bytes = b"expected-onnx"
    tok_bytes = b"{}"
    lock_file = tmp_path / "manifest.lock"
    lock_file.write_text(
        json.dumps(
            {
                "version": 1,
                "models": [
                    {
                        "id": "m1",
                        "repo": "org/m1",
                        "rev": "1234567",
                        "sha256": {
                            "onnx/model.onnx": _sha256_bytes(onnx_bytes),
                            "tokenizer.json": _sha256_bytes(tok_bytes),
                        },
                    }
                ],
            }
        ),
        encoding="utf-8",
    )
    models_dir = tmp_path / "models"
    model_dir = models_dir / "m1"
    model_dir.mkdir(parents=True)
    # Put an arbitrary unpinned .onnx instead of onnx/model.onnx
    (model_dir / "evil.onnx").write_bytes(b"evil")
    (model_dir / "tokenizer.json").write_bytes(tok_bytes)

    monkeypatch.setattr(verify, "LOCK", lock_file)
    monkeypatch.setattr(verify, "MODELS_DIR", models_dir)
    monkeypatch.setattr(verify, "_lock_cache", None)

    ort_called = False

    def fake_inference_session(*_a, **_kw):
        nonlocal ort_called
        ort_called = True
        raise AssertionError("InferenceSession must not be invoked on unverified/missing ONNX")

    monkeypatch.setitem(sys.modules, "onnxruntime", SimpleNamespace(InferenceSession=fake_inference_session))
    monkeypatch.setitem(sys.modules, "tokenizers", SimpleNamespace(Tokenizer=SimpleNamespace(from_file=lambda p: None)))

    m = {
        "id": "m1",
        "repo": "org/m1",
        "rev": "1234567",
        "dim": 384,
        "onnx": "onnx/model.onnx",
        "group": "en",
    }
    assert verify.verify_model(m) is False
    assert ort_called is False


def test_verify_check_structure_validates_parity_and_completeness(tmp_path: pathlib.Path, monkeypatch: pytest.MonkeyPatch) -> None:
    models = _sample_models()
    manifest = _minimal_manifest(models, default="m-co-1")
    h1 = _sha256_bytes(b"onnx")
    h2 = _sha256_bytes(b"tok")
    lock = _lock_for_models(models, h1, h2)

    lock_file = tmp_path / "manifest.lock"
    models_dir = tmp_path / "models"
    monkeypatch.setattr(verify, "LOCK", lock_file)
    monkeypatch.setattr(verify, "MODELS_DIR", models_dir)

    # 1. Valid 9/9 lock passes
    lock_file.write_text(json.dumps(lock), encoding="utf-8")
    monkeypatch.setattr(verify, "_lock_cache", None)
    assert verify.check_structure(manifest) is True

    # 2. Missing one ONNX model from lock -> fails
    incomplete = {"version": 1, "models": lock["models"][:-1]}
    lock_file.write_text(json.dumps(incomplete), encoding="utf-8")
    monkeypatch.setattr(verify, "_lock_cache", None)
    assert verify.check_structure(manifest) is False

    # 3. Rev mismatch -> fails
    drifted = json.loads(json.dumps(lock))
    drifted["models"][0]["rev"] = "0000000"
    lock_file.write_text(json.dumps(drifted), encoding="utf-8")
    monkeypatch.setattr(verify, "_lock_cache", None)
    assert verify.check_structure(manifest) is False


def test_download_fail_closed_without_update_lock(tmp_path: pathlib.Path, monkeypatch: pytest.MonkeyPatch) -> None:
    good_onnx = b"good-onnx-content"
    good_tok = b"good-tokenizer-content"
    bad_onnx = b"tampered-onnx-content"

    manifest = _minimal_manifest(
        [
            {
                "id": "m1",
                "repo": "org/m1",
                "rev": "1234567",
                "dim": 384,
                "size_onnx_mb": 10,
                "size_hf_mb": 10,
                "onnx": "onnx/model.onnx",
                "langs": ["en"],
                "license": "MIT",
                "group": "en",
            }
        ],
        default="m1",
    )
    lock_data = {
        "version": 1,
        "models": [
            {
                "id": "m1",
                "repo": "org/m1",
                "rev": "1234567",
                "sha256": {
                    "onnx/model.onnx": _sha256_bytes(good_onnx),
                    "tokenizer.json": _sha256_bytes(good_tok),
                },
            }
        ],
    }

    manifest_file = tmp_path / "manifest.json"
    lock_file = tmp_path / "manifest.lock"
    models_dir = tmp_path / "models"
    manifest_file.write_text(json.dumps(manifest), encoding="utf-8")
    original_lock_text = json.dumps(lock_data, indent=2) + "\n"
    lock_file.write_text(original_lock_text, encoding="utf-8")

    monkeypatch.setattr(download, "MANIFEST", manifest_file)
    monkeypatch.setattr(download, "LOCK", lock_file)
    monkeypatch.setattr(download, "MODELS_DIR", models_dir)

    # Inject fake huggingface_hub so download.py runs completely offline
    state = {"payload": bad_onnx}

    class FakeHfApi:
        def repo_info(self, repo, repo_type="model"):
            return SimpleNamespace(sha="fffffff9999999")

    def fake_snapshot_download(repo_id, revision, local_dir, allow_patterns):
        d = pathlib.Path(local_dir)
        (d / "onnx").mkdir(parents=True, exist_ok=True)
        (d / "onnx" / "model.onnx").write_bytes(state["payload"])
        (d / "tokenizer.json").write_bytes(good_tok)

    fake_hf_mod = SimpleNamespace(HfApi=FakeHfApi, snapshot_download=fake_snapshot_download)
    monkeypatch.setitem(sys.modules, "huggingface_hub", fake_hf_mod)

    # Tampered download WITHOUT --update-lock -> must fail, delete bad file, and NOT mutate manifest.lock
    monkeypatch.setattr(sys, "argv", ["download.py", "--only", "m1"])
    rc = download.main()
    assert rc != 0
    assert lock_file.read_text(encoding="utf-8") == original_lock_text
    assert not (models_dir / "m1" / "onnx" / "model.onnx").exists()

    # Good download WITHOUT --update-lock -> must pass and NOT mutate manifest.lock
    state["payload"] = good_onnx
    monkeypatch.setattr(sys, "argv", ["download.py", "--only", "m1"])
    rc = download.main()
    assert rc == 0
    assert lock_file.read_text(encoding="utf-8") == original_lock_text
    assert (models_dir / "m1" / "onnx" / "model.onnx").exists()

    # Tampered download WITH --update-lock -> explicitly updates manifest.lock
    state["payload"] = bad_onnx
    # Clean dir so snapshot_download writes fresh tampered payload
    (models_dir / "m1" / "onnx" / "model.onnx").unlink()
    monkeypatch.setattr(sys, "argv", ["download.py", "--only", "m1", "--update-lock"])
    rc = download.main()
    assert rc == 0
    updated_lock = json.loads(lock_file.read_text(encoding="utf-8"))
    assert updated_lock["models"][0]["sha256"]["onnx/model.onnx"] == _sha256_bytes(bad_onnx)


def test_download_rejects_404_main_fallback_without_update_lock(tmp_path: pathlib.Path, monkeypatch: pytest.MonkeyPatch) -> None:
    manifest = _minimal_manifest(
        [
            {
                "id": "m1",
                "repo": "org/m1",
                "rev": "1234567",
                "dim": 384,
                "size_onnx_mb": 10,
                "size_hf_mb": 10,
                "onnx": "onnx/model.onnx",
                "langs": ["en"],
                "license": "MIT",
                "group": "en",
            }
        ],
        default="m1",
    )
    lock_data = {
        "version": 1,
        "models": [
            {
                "id": "m1",
                "repo": "org/m1",
                "rev": "1234567",
                "sha256": {
                    "onnx/model.onnx": _sha256_bytes(b"good"),
                    "tokenizer.json": _sha256_bytes(b"tok"),
                },
            }
        ],
    }
    manifest_file = tmp_path / "manifest.json"
    lock_file = tmp_path / "manifest.lock"
    models_dir = tmp_path / "models"
    manifest_file.write_text(json.dumps(manifest), encoding="utf-8")
    lock_file.write_text(json.dumps(lock_data), encoding="utf-8")

    monkeypatch.setattr(download, "MANIFEST", manifest_file)
    monkeypatch.setattr(download, "LOCK", lock_file)
    monkeypatch.setattr(download, "MODELS_DIR", models_dir)

    fallback_called = False

    class FakeHfApi:
        def repo_info(self, repo, repo_type="model"):
            nonlocal fallback_called
            fallback_called = True
            return SimpleNamespace(sha="fffffff9999999")

    def fake_snapshot_download(repo_id, revision, local_dir, allow_patterns):
        raise RuntimeError("404 Revision Not Found")

    monkeypatch.setitem(sys.modules, "huggingface_hub", SimpleNamespace(HfApi=FakeHfApi, snapshot_download=fake_snapshot_download))
    monkeypatch.setattr(sys, "argv", ["download.py", "--only", "m1"])
    rc = download.main()
    assert rc != 0
    assert fallback_called is False
    assert json.loads(manifest_file.read_text(encoding="utf-8"))["models"][0]["rev"] == "1234567"

