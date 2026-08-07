import { FormEvent, useState } from "react";
import { SearchResult, vantaErrorMessage } from "../vanta";
import { search } from "../vanta";
import ResultsList from "./ResultsList";

interface Props {
  busy: boolean;
  runError: (msg: string) => void;
}

export default function SearchBar({ busy, runError }: Props) {
  const [query, setQuery] = useState("");
  const [topK, setTopK] = useState(5);
  const [results, setResults] = useState<SearchResult[] | null>(null);
  const [searching, setSearching] = useState(false);

  async function handleSubmit(e: FormEvent) {
    e.preventDefault();
    setSearching(true);
    try {
      const r = await search({ query, top_k: topK });
      setResults(r);
    } catch (err) {
      runError(vantaErrorMessage(err));
    } finally {
      setSearching(false);
    }
  }

  return (
    <section className="panel">
      <h2>Search</h2>
      <form className="row" onSubmit={handleSubmit}>
        <input
          value={query}
          onChange={(e) => setQuery(e.target.value)}
          placeholder="Semantic query"
          aria-label="Query"
          required
        />
        <input
          type="number"
          min={1}
          max={100}
          value={topK}
          onChange={(e) => setTopK(Number(e.target.value))}
          className="narrow"
          aria-label="Top K"
        />
        <button type="submit" disabled={busy || searching || !query.trim()}>
          {searching ? "Searching…" : "Search"}
        </button>
      </form>
      <ResultsList results={results} />
    </section>
  );
}