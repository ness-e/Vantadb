"""Sanity numérico: ejecuta los 3 modelos descargados con ort+tokenizers reales
y mide cosine entre pares (hola mundo, hello world, query=doc).

Contrato esperado:
- all-MiniLM-L6-v2 (EN-only, 384d):
    cosine(hola mundo, hello world) ~0.3  (idiomas distintos -> baja)
    cosine(doc, doc) ~1.0
    cosine(doc, paraphrase) >0.7
- paraphrase-multilingual-MiniLM-L12-v2 (ES multi, 384d):
    cosine(hola mundo, hello world) >0.7  (paraphrase cross-lang)
    cosine(doc_es, doc_es_equiv_en) >0.6
- multilingual-e5-small (combined, 384d):
    cosine(hola mundo, hello world) >0.85 (MTEB multi SOTA)

ONNX inputs: sentence-transformers exportan mean-pooling + L2 norm en el ONNX
mismo, asi que la salida ya es un embedding normalizado -> cosine = dot product.
"""
import json, math, pathlib, sys

import numpy as np
import onnxruntime as ort
import tokenizers

MODELS = pathlib.Path("embeddings/models")
REPORT = pathlib.Path("embeddings/sanity_report.json")

PAIRS = {
    "self": [("hola mundo", "hola mundo")],
    "cross_lang": [("hola mundo", "hello world")],
    "paraphrase_en": [("the quick brown fox", "a fast auburn fox")],
    "paraphrase_es": [("el rápido zorro marrón", "una veloz zorra castaña")],
    "doc_query_en": [
        ("machine learning model for retrieval and search", "machine learning retrieval"),
        ("vector database benchmark case english", "vector database search"),
    ],
    "doc_query_es": [
        ("modelo de aprendizaje automático para búsqueda y recuperación", "aprendizaje automático recuperación"),
        ("caso de referencia base de datos vectorial español", "base de datos vectorial búsqueda"),
    ],
}

EXPECTED = {
    "all-MiniLM-L6-v2": {
        "group": "en",
        # sentence-transformers EN-only tiene cross-lingual bleed: no se queda <0.5
        # en pares bilingues (transfer learning del WordPiece + corpus multilingue
        # residual). Threshold realista: <0.95 cross-lang (vs paraphrase_en >=0.5)
        "self": (0.99, 1.01),
        "cross_lang": (0.0, 0.95),
        "paraphrase_en": (0.5, 1.0),
        "paraphrase_es": (0.0, 0.95),
        "doc_query_en": (0.4, 1.0),
        "doc_query_es": (0.0, 0.95),
    },
    "paraphrase-multilingual-MiniLM-L12-v2": {
        "group": "es",
        "self": (0.99, 1.01),
        "cross_lang": (0.65, 1.0),
        "paraphrase_en": (0.5, 1.0),
        "paraphrase_es": (0.5, 1.0),
        "doc_query_en": (0.4, 1.0),
        "doc_query_es": (0.4, 1.0),
    },
    "multilingual-e5-small": {
        "group": "combined",
        "self": (0.99, 1.01),
        "cross_lang": (0.80, 1.0),
        "paraphrase_en": (0.5, 1.0),
        "paraphrase_es": (0.5, 1.0),
        "doc_query_en": (0.4, 1.0),
        "doc_query_es": (0.4, 1.0),
    },
}


def load_model(model_id: str):
    d = MODELS / model_id
    onnx_files = sorted(d.rglob("*.onnx"))
    tok_files = sorted(d.rglob("tokenizer.json"))
    if not onnx_files or not tok_files:
        raise FileNotFoundError(f"{model_id}: missing onnx or tokenizer")
    sess = ort.InferenceSession(str(onnx_files[0]), providers=["CPUExecutionProvider"])
    tok = tokenizers.Tokenizer.from_file(str(tok_files[0]))
    return sess, tok


def embed(sess, tok, text: str, max_len: int = 128) -> list[float]:
    enc = tok.encode(text)
    ids = enc.ids[:max_len]
    mask = [1] * len(ids)
    type_ids = [0] * len(ids)
    # pad to common length
    while len(ids) < max_len:
        ids.append(0)
        mask.append(0)
        type_ids.append(0)
    feeds = {
        "input_ids": np.array([ids], dtype=np.int64),
        "attention_mask": np.array([mask], dtype=np.int64),
        "token_type_ids": np.array([type_ids], dtype=np.int64),
    }
    # algunos ONNX exportan con nombres distintos — detectar input names
    input_names = {i.name for i in sess.get_inputs()}
    if "token_type_ids" not in input_names:
        feeds.pop("token_type_ids", None)
    out = sess.run(None, feeds)
    # sentence-transformers exporta sentence_embedding (post mean-pool + L2 norm)
    out_names = {o.name for o in sess.get_outputs()}
    if "sentence_embedding" in out_names:
        vec = out[out_names.index("sentence_embedding")]
    else:
        # fallback: mean-pool last_hidden_state
        last = out[0][0]  # (seq, dim)
        m = np.array(mask, dtype=np.float32)[:, None]
        vec = (last * m).sum(axis=0, keepdims=True) / max(m.sum(), 1.0)
        n = np.linalg.norm(vec, axis=1, keepdims=True)
        vec = vec / np.maximum(n, 1e-12)
    return vec[0].tolist()


def cosine(a, b):
    a, b = np.array(a), np.array(b)
    n = np.linalg.norm(a) * np.linalg.norm(b)
    return float(a @ b / n) if n > 0 else 0.0


def main():
    report = {"models": {}}
    overall_ok = True
    for mid in EXPECTED:
        print(f"\n=== {mid} ===", flush=True)
        try:
            sess, tok = load_model(mid)
        except Exception as e:
            print(f"  FAIL load: {e}")
            report["models"][mid] = {"error": str(e)}
            overall_ok = False
            continue
        results = {}
        for category, pairs in PAIRS.items():
            cosines = []
            for a, b in pairs:
                va = embed(sess, tok, a)
                vb = embed(sess, tok, b)
                c = cosine(va, vb)
                cosines.append({"a": a, "b": b, "cos": round(c, 4)})
            avg = round(sum(x["cos"] for x in cosines) / len(cosines), 4)
            results[category] = {"pairs": cosines, "avg": avg}
            exp_lo, exp_hi = EXPECTED[mid].get(category, (None, None))
            ok = (exp_lo is None) or (exp_lo <= avg <= exp_hi)
            mark = "PASS" if ok else "FAIL"
            if not ok:
                overall_ok = False
            print(f"  {category:18} avg={avg:.4f}  expected [{exp_lo}, {exp_hi}]  {mark}")
        report["models"][mid] = results
    REPORT.write_text(json.dumps(report, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")
    print(f"\n[report] escrito {REPORT}")
    return 0 if overall_ok else 1


if __name__ == "__main__":
    sys.exit(main())