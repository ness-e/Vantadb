// Data Explorer (ADMIN-07). Browses records of the active connection and runs
// semantic/text search with score display.
//
// Pagination: the core `vanta_list` command only accepts `namespace` + `limit`
// (no offset/cursor), so real paging would need changes across the connection
// trait, both adapters, and the engine. Instead "Load more" grows the limit
// (50 → 100 → …) and the fetch replaces the list — more records, no dupes.
// ponytail: grow-on-demand limit. Replace with real offset/cursor when the
// core exposes one.
import { FormEvent, useEffect, useState } from "react";
import { list, search, vantaErrorMessage } from "../vanta";

interface Row {
  id: string;
  namespace: string;
  text: string;
  score: number | null;
}

interface Props {
  active: boolean;
  busy: boolean;
  runError: (msg: string) => void;
}

const STEP = 50;

export default function DataExplorer({ active, busy, runError }: Props) {
  const [query, setQuery] = useState("");
  const [rows, setRows] = useState<Row[] | null>(null);
  const [limit, setLimit] = useState(STEP);
  const [loading, setLoading] = useState(false);
  const [mode, setMode] = useState<"list" | "search">("list");

  async function fetchRows(kind: "list" | "search", q: string, lim: number) {
    setLoading(true);
    try {
      const next: Row[] =
        kind === "search"
          ? (await search({ query: q, top_k: lim })).map((r) => ({
              id: r.id,
              namespace: r.namespace,
              text: r.text,
              score: r.score,
            }))
          : (await list({ limit: lim })).map((r) => ({
              id: r.id,
              namespace: r.namespace,
              text: r.text,
              score: null,
            }));
      setRows(next);
      setMode(kind);
    } catch (err) {
      runError(vantaErrorMessage(err));
    } finally {
      setLoading(false);
    }
  }

  // Browse the active connection on mount / when one appears.
  useEffect(() => {
    if (active) fetchRows("list", "", STEP);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [active]);

  function handleSubmit(e: FormEvent) {
    e.preventDefault();
    setLimit(STEP);
    const q = query.trim();
    fetchRows(q ? "search" : "list", q, STEP);
  }

  function handleMore() {
    const lim = limit + STEP;
    setLimit(lim);
    fetchRows(mode, query.trim(), lim);
  }

  return (
    <section className="panel" aria-label="Data explorer">
      <div className="panel-head">
        <h2>Data Explorer</h2>
        <span className="muted">
          {mode} · {rows ? `${rows.length} shown` : "idle"}
        </span>
      </div>

      <form className="row" onSubmit={handleSubmit}>
        <input
          value={query}
          onChange={(e) => setQuery(e.target.value)}
          placeholder="Semantic query (empty = browse records)"
          aria-label="Query records"
        />
        <button type="submit" disabled={busy || loading || !active}>
          {loading ? "Loading…" : "Fetch"}
        </button>
      </form>

      {!active ? (
        <p className="muted">No active connection — connect one to explore records.</p>
      ) : rows === null ? (
        <p className="muted">Loading records…</p>
      ) : rows.length === 0 ? (
        <p className="muted">No records{mode === "search" ? " match" : ""}.</p>
      ) : (
        <>
          <table className="explorer-table">
            <thead>
              <tr>
                <th>id</th>
                <th>ns</th>
                <th>text</th>
                {mode === "search" && <th>score</th>}
              </tr>
            </thead>
            <tbody>
              {rows.map((r) => (
                <tr key={`${r.namespace}:${r.id}`}>
                  <td>
                    <code>{r.id}</code>
                  </td>
                  <td>
                    <span className="tag">{r.namespace}</span>
                  </td>
                  <td className="explorer-text" title={r.text}>
                    {r.text}
                  </td>
                  {mode === "search" && (
                    <td className="score">{r.score != null ? r.score.toFixed(3) : "—"}</td>
                  )}
                </tr>
              ))}
            </tbody>
          </table>
          <div className="row explorer-more">
            <button onClick={handleMore} disabled={busy || loading}>
              Load more (+{STEP})
            </button>
          </div>
        </>
      )}
    </section>
  );
}
