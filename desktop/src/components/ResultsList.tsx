import { SearchResult } from "../vanta";

interface Props {
  results: SearchResult[] | null;
}

export default function ResultsList({ results }: Props) {
  if (results === null) {
    return <p className="muted">Run a search to see results.</p>;
  }
  if (results.length === 0) {
    return <p className="muted">No matches.</p>;
  }
  return (
    <ol className="results">
      {results.map((r) => (
        <li key={`${r.namespace}:${r.id}`}>
          <div className="row-between">
            <code>{r.id}</code>
            <span className="score">{r.score.toFixed(3)}</span>
          </div>
          <p>{r.text}</p>
          <span className="tag">{r.namespace}</span>
        </li>
      ))}
    </ol>
  );
}