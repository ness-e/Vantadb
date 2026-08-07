import { FormEvent, useState } from "react";
import { ingest, IngestItem, vantaErrorMessage } from "../vanta";

interface Props {
  onDone: (ids: string[]) => void;
  runError: (msg: string) => void;
}

export default function IngestForm({ onDone, runError }: Props) {
  const [id, setId] = useState("");
  const [text, setText] = useState("");
  const [namespace, setNamespace] = useState("");
  const [busy, setBusy] = useState(false);

  async function handleSubmit(e: FormEvent) {
    e.preventDefault();
    const item: IngestItem = { id: id || undefined, text, namespace: namespace || undefined };
    setBusy(true);
    try {
      const ids = await ingest([item]);
      onDone(ids);
      setId("");
      setText("");
    } catch (err) {
      runError(vantaErrorMessage(err));
    } finally {
      setBusy(false);
    }
  }

  return (
    <section className="panel">
      <h2>Ingest</h2>
      <form className="stack" onSubmit={handleSubmit}>
        <input
          value={id}
          onChange={(e) => setId(e.target.value)}
          placeholder="Id (optional — backend assigns one)"
          aria-label="Record id"
        />
        <textarea
          value={text}
          onChange={(e) => setText(e.target.value)}
          placeholder="Text content"
          rows={3}
          aria-label="Text content"
          required
        />
        <input
          value={namespace}
          onChange={(e) => setNamespace(e.target.value)}
          placeholder="Namespace (defaults to 'default')"
          aria-label="Namespace"
        />
        <button type="submit" disabled={busy || !text.trim()}>
          {busy ? "Storing…" : "Add record"}
        </button>
      </form>
    </section>
  );
}