import { type KeyboardEvent, useState } from "react";
import { SearchResult } from "../vanta";

interface Props {
  results: SearchResult[] | null;
  /** Master-detail (VS-03): clicking a result opens it in the right Inspector. */
  onSelect?: (r: SearchResult) => void;
}

const CARD =
  "list-none border-2 border-foreground bg-card px-3 py-2 shadow-ink-sm [&_p]:my-1";

export default function ResultsList({ results, onSelect }: Props) {
  // UX-02: resultado abierto en el Inspector → aria-selected del listbox.
  const [openKey, setOpenKey] = useState<string | null>(null);

  // UX-02: Enter/Espacio abren el Inspector desde el resultado enfocado. El
  // guard e.target === e.currentTarget evita que teclas en hijos disparen el
  // master-detail.
  function handleKey(e: KeyboardEvent<HTMLLIElement>, r: SearchResult) {
    if (e.target !== e.currentTarget) return;
    if (e.key !== "Enter" && e.key !== " ") return;
    e.preventDefault();
    setOpenKey(`${r.namespace}:${r.id}`);
    onSelect?.(r);
  }

  if (results === null) {
    return <p className="text-muted-foreground">Run a search to see results.</p>;
  }
  if (results.length === 0) {
    return <p className="text-muted-foreground">No matches.</p>;
  }
  return (
    <ol role="listbox" aria-label="Resultados de búsqueda" className="mt-3 flex flex-col gap-2 p-0">
      {results.map((r) => {
        const key = `${r.namespace}:${r.id}`;
        return (
          <li
            key={key}
            role="option"
            aria-selected={openKey === key}
            onClick={
              onSelect
                ? () => {
                    setOpenKey(key);
                    onSelect(r);
                  }
                : undefined
            }
            onKeyDown={onSelect ? (e) => handleKey(e, r) : undefined}
            tabIndex={onSelect ? 0 : undefined}
            className={`${CARD} ${onSelect ? "cursor-pointer" : ""}`}
            title={onSelect ? "Ver en inspector" : undefined}
          >
            <div className="flex justify-between">
              <code>{r.id}</code>
              <span className="font-bold text-accent-text">{r.score.toFixed(3)}</span>
            </div>
            <p>{r.text}</p>
            <span className="inline-block border-2 border-foreground bg-neon px-2 py-px font-tech text-xs text-accent-foreground">
              {r.namespace}
            </span>
          </li>
        );
      })}
    </ol>
  );
}