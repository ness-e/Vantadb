import { createLazyRoute, Link } from "@tanstack/react-router";
import { useMemo } from "react";
import { NbSubpageHero } from "@/components/NbSubpageHero";
import { getAllPosts } from "../../lib/blog";

export const Route = createLazyRoute("/blog/")({
  component: BlogIndex,
});

function BlogIndex() {
  const posts = useMemo(() => getAllPosts(), []);

  return (
    <div className="nb-page">
      <NbSubpageHero
        num="00"
        eyebrow="Blog"
        title={
          <span>
            Deep dives.
            <br />
            Release notes.
          </span>
        }
        sub="Engineering blog about embedded vector databases, AI agents, local RAG, and the future of AI data infrastructure."
      />

      <section className="nb-section">
        <div className="nb-inner">
          <span className="nb-label nb-label--amber">Posts — {posts.length} articles</span>
          <div className="nb-divider" />

          {posts.length === 0 ? (
            <div
              className="nb-frame"
              data-frame-label="EMPTY"
              style={{ marginTop: "var(--space-xl)", padding: "var(--space-3xl)" }}
            >
              <span className="nb-label" style={{ textAlign: "center", marginBottom: 0 }}>
                No posts yet. Check back soon.
              </span>
            </div>
          ) : (
            <div className="nb-grid" style={{ marginTop: "var(--space-xl)" }}>
              {posts.map((post) => (
                <Link
                  key={post.slug}
                  to="/blog/$slug"
                  params={{ slug: post.slug }}
                  className="nb-cell"
                  style={{
                    display: "grid",
                    gridTemplateColumns: "140px 1fr auto",
                    gap: "var(--space-lg)",
                    alignItems: "start",
                    padding: "var(--space-lg) var(--space-xl)",
                    textDecoration: "none",
                    borderLeft: "2px solid transparent",
                    transition: "all 150ms var(--ease-brutal)",
                    background: "var(--background)",
                  }}
                >
                  <span className="nb-label" style={{ marginBottom: 0, paddingTop: "2px" }}>
                    {post.date}
                  </span>

                  <div
                    style={{ display: "flex", flexDirection: "column", gap: "var(--space-2xs)" }}
                  >
                    <h2
                      style={{
                        fontFamily: "var(--font-display)",
                        fontSize: "var(--text-title)",
                        fontWeight: 700,
                        letterSpacing: "var(--tracking-display)",
                        color: "var(--foreground)",
                        margin: 0,
                      }}
                    >
                      {post.title}
                    </h2>
                    {post.description && (
                      <p
                        style={{
                          fontFamily: "var(--font-sans)",
                          fontSize: "var(--text-code)",
                          color: "var(--muted)",
                          lineHeight: 1.5,
                          margin: 0,
                        }}
                      >
                        {post.description}
                      </p>
                    )}
                    <div
                      style={{
                        display: "flex",
                        gap: "var(--space-2xs)",
                        flexWrap: "wrap",
                        marginTop: "var(--space-3xs)",
                      }}
                    >
                      {post.author && (
                        <span
                          className="nb-pill-status nb-pill-status--amber"
                          style={{ fontSize: "var(--text-micro)" }}
                        >
                          {post.author}
                        </span>
                      )}
                      {post.tags?.map((t) => (
                        <span
                          key={t}
                          className="nb-pill-status"
                          style={{ fontSize: "var(--text-micro)" }}
                        >
                          {t}
                        </span>
                      ))}
                    </div>
                  </div>

                  <span className="nb-arrow" style={{ alignSelf: "center", marginTop: 0 }} />
                </Link>
              ))}
            </div>
          )}
        </div>
      </section>

      <style>{`
        .nb-cell:hover { border-left-color: var(--amber) !important; }
      `}</style>
    </div>
  );
}

export function PendingComponent() {
  return (
    <div
      style={{
        display: "flex",
        justifyContent: "center",
        alignItems: "center",
        height: "60vh",
        color: "var(--muted)",
      }}
    >
      <span className="nb-label" style={{ fontSize: "var(--text-label)", marginBottom: 0 }}>
        Loading...
      </span>
    </div>
  );
}
