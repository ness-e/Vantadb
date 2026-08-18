import { SearchResult } from "../vanta";

interface Props {
  results: SearchResult[] | null;
  /** Master-detail (VS-03): clicking a result opens it in the right Inspector. */
  onSelect?: (r: SearchResult) => void;
}

export default function ResultsList({ results, onSelect }: Props) {
  if (results === null) {
    return <p className="muted">Run a search to see results.</p>;
  }
  if (results.length === 0) {
    return <p className="muted">No matches.</p>;
  }
  return (
    <ol className="results">
      {results.map((r) => (
        <li
          key={`${r.namespace}:${r.id}`}
          onClick={onSelect ? () => onSelect(r) : undefined}
          className={onSelect ? "cursor-pointer" : undefined}
          title={onSelect ? "Ver en inspector" : undefined}
        >
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