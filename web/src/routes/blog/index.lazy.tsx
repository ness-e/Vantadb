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
          <h2
            style={{
              fontFamily: "var(--font-display)",
              fontSize: "var(--text-title)",
              fontWeight: 700,
              color: "var(--foreground)",
              margin: "0 0 var(--space-md)",
            }}
          >
            Posts — {posts.length} articles
          </h2>
          <div className="nb-divider" />

          {posts.length === 0 ? (
            <div
              className="nb-frame"
              style={{ marginTop: "var(--space-xl)", padding: "var(--space-3xl)" }}
            >
              <span
                style={{
                  fontFamily: "var(--font-mono)",
                  fontSize: "var(--text-micro)",
                  color: "var(--muted)",
                  marginBottom: 0,
                  display: "block",
                  textAlign: "center",
                }}
              >
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
                  <span
                    style={{
                      fontFamily: "var(--font-mono)",
                      fontSize: "var(--text-micro)",
                      color: "var(--muted)",
                      marginBottom: 0,
                      paddingTop: "2px",
                      display: "block",
                    }}
                  >
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
                          style={{
                            fontFamily: "var(--font-mono)",
                            fontSize: "var(--text-micro)",
                            color: "var(--amber)",
                            fontWeight: 700,
                            border: "1px solid var(--amber)",
                            padding: "2px 6px",
                          }}
                        >
                          {post.author}
                        </span>
                      )}
                      {post.tags?.map((t) => (
                        <span
                          key={t}
                          style={{
                            fontFamily: "var(--font-mono)",
                            fontSize: "var(--text-micro)",
                            color: "var(--muted)",
                            border: "1px solid var(--border)",
                            padding: "2px 6px",
                          }}
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
      <span
        style={{
          fontFamily: "var(--font-mono)",
          fontSize: "var(--text-label)",
          color: "var(--muted)",
          marginBottom: 0,
        }}
      >
        Loading...
      </span>
    </div>
  );
}
