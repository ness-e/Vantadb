"use client";

import { useEffect, useState } from "react";
import Markdown from "react-markdown";

/**
 * /docs/guides — renders the repository guides (markdown) inside the site.
 * Files live in public/docs/ (copied from docs/ at build-prep time).
 * Light-only by design decision (FIND-26): marketing site has no dark theme.
 */

const GUIDES = [
  { file: "QUICKSTART.md", title: "Quickstart", desc: "Install and run your first hybrid query in 5 minutes." },
  { file: "VERSIONING.md", title: "Versioning & Stability", desc: "What stability we promise 0.x consumers." },
  { file: "UPGRADE.md", title: "Upgrade Guide", desc: "What changes between versions, step by step." },
  { file: "BACKUP_RESTORE.md", title: "Backup & Restore", desc: "How to back up your embedded database safely." },
] as const;

export default function DocsGuidesPage() {
  const [selected, setSelected] = useState<string>(GUIDES[0].file);
  const [content, setContent] = useState<string>("");
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    let cancelled = false;
    setLoading(true);
    setError(null);
    fetch(`/docs/${selected}`)
      .then((r) => {
        if (!r.ok) throw new Error(`HTTP ${r.status}`);
        return r.text();
      })
      .then((text) => {
        if (!cancelled) {
          setContent(text);
          setLoading(false);
        }
      })
      .catch((e: unknown) => {
        if (!cancelled) {
          setError(e instanceof Error ? e.message : "Failed to load");
          setLoading(false);
        }
      });
    return () => {
      cancelled = true;
    };
  }, [selected]);

  return (
    <div className="min-h-screen bg-[#FBF9F5] text-black">
      <div className="mx-auto max-w-5xl px-4 py-10">
        <h1 className="font-[family-name:var(--font-anton)] text-4xl uppercase tracking-wide">
          Guides
        </h1>
        <p className="mt-2 text-sm text-[#3A3A3A]">
          Official documentation, rendered from the repository sources.
        </p>
        <div className="mt-6 flex flex-wrap gap-2">
          {GUIDES.map((g) => (
            <button
              key={g.file}
              onClick={() => setSelected(g.file)}
              className={`rounded-none border-2 border-black px-3 py-1.5 text-sm font-medium transition-colors ${
                selected === g.file
                  ? "bg-[#FF5500] text-black"
                  : "bg-white hover:bg-[#F2EDE2]"
              }`}
            >
              {g.title}
            </button>
          ))}
        </div>
        <ul className="mt-4 space-y-1 text-xs text-[#3A3A3A]">
          <li>{GUIDES.find((g) => g.file === selected)?.desc}</li>
        </ul>
        <article className="prose prose-sm mt-8 max-w-none border-4 border-black bg-white p-6 shadow-[6px_6px_0_0_#000]">
          {loading && <p className="text-sm">Loading…</p>}
          {error && (
            <p className="text-sm text-red-700">
              Failed to load guide: {error}
            </p>
          )}
          {!loading && !error && <Markdown>{content}</Markdown>}
        </article>
      </div>
    </div>
  );
}
