import { SearchResult } from "../vanta";

interface Props {
  results: SearchResult[] | null;
  /** Master-detail (VS-03): clicking a result opens it in the right Inspector. */
  onSelect?: (r: SearchResult) => void;
}

const CARD =
  "list-none border-2 border-foreground bg-card px-3 py-2 shadow-[3px_3px_0_0_#000] dark:shadow-[3px_3px_0_0_#FBF9F5] [&_p]:my-1";

export default function ResultsList({ results, onSelect }: Props) {
  if (results === null) {
    return <p className="text-muted-foreground">Run a search to see results.</p>;
  }
  if (results.length === 0) {
    return <p className="text-muted-foreground">No matches.</p>;
  }
  return (
    <ol className="mt-3 flex flex-col gap-2 p-0">
      {results.map((r) => (
        <li
          key={`${r.namespace}:${r.id}`}
          onClick={onSelect ? () => onSelect(r) : undefined}
          className={`${CARD} ${onSelect ? "cursor-pointer" : ""}`}
          title={onSelect ? "Ver en inspector" : undefined}
        >
          <div className="flex justify-between">
            <code>{r.id}</code>
            <span className="font-bold text-neon">{r.score.toFixed(3)}</span>
          </div>
          <p>{r.text}</p>
          <span className="inline-block border-2 border-foreground bg-neon px-2 py-px font-tech text-xs text-accent-foreground">
            {r.namespace}
          </span>
        </li>
      ))}
    </ol>
  );
}
