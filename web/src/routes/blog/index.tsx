import { createFileRoute, Link } from "@tanstack/react-router";
import { SwissSubpageHero } from "@/components/SwissSubpageHero";
import { getAllPosts } from "../../lib/blog";

export const Route = createFileRoute("/blog/")({
  head: () => ({
    meta: [
      { title: "VantaDB Blog" },
      {
        name: "description",
        content:
          "Engineering blog, release notes, and deep dives into embedded vector databases, AI agents, and local RAG.",
      },
    ],
  }),
  component: BlogIndex,
});

function BlogIndex() {
  const posts = getAllPosts();

  return (
    <div className="engine-page">
      <SwissSubpageHero
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

      <main className="engine-main">
        <section className="engine-section">
          <span className="swiss-eyebrow">Posts — {posts.length} articles</span>

          {posts.length === 0 ? (
            <div
              style={{
                border: "1px solid var(--border)",
                padding: "4rem",
                textAlign: "center",
                marginTop: "3rem",
              }}
            >
              <p
                style={{
                  fontFamily: "var(--font-mono)",
                  fontSize: "0.8rem",
                  color: "var(--steel)",
                  textTransform: "uppercase",
                  letterSpacing: "0.08em",
                  margin: 0,
                }}
              >
                No posts yet. Check back soon.
              </p>
            </div>
          ) : (
            <div
              style={{
                display: "grid",
                gridTemplateColumns: "1fr",
                gap: "1px",
                background: "var(--border)",
                border: "1px solid var(--border)",
                marginTop: "3rem",
              }}
            >
              {posts.map((post) => (
                <Link
                  key={post.slug}
                  to="/blog/$slug"
                  params={{ slug: post.slug }}
                  style={{
                    display: "grid",
                    gridTemplateColumns: "160px 1fr auto",
                    gap: "2rem",
                    alignItems: "start",
                    padding: "2rem 2.5rem",
                    background: "var(--background)",
                    textDecoration: "none",
                    borderLeft: "2px solid transparent",
                    transition: "all 150ms var(--ease-cut)",
                  }}
                  onMouseEnter={(e) => {
                    const el = e.currentTarget as HTMLAnchorElement;
                    el.style.background = "var(--surface-raised)";
                    el.style.borderLeftColor = "var(--amber)";
                  }}
                  onMouseLeave={(e) => {
                    const el = e.currentTarget as HTMLAnchorElement;
                    el.style.background = "var(--background)";
                    el.style.borderLeftColor = "transparent";
                  }}
                >
                  <span
                    style={{
                      fontFamily: "var(--font-mono)",
                      fontSize: "0.65rem",
                      color: "var(--steel)",
                      textTransform: "uppercase",
                      letterSpacing: "0.06em",
                      paddingTop: "3px",
                    }}
                  >
                    {post.date}
                  </span>

                  <div style={{ display: "flex", flexDirection: "column", gap: "0.4rem" }}>
                    <h2
                      style={{
                        fontFamily: "var(--font-display)",
                        fontSize: "1rem",
                        fontWeight: 700,
                        letterSpacing: "-0.02em",
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
                          fontSize: "0.8rem",
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
                        gap: "0.4rem",
                        flexWrap: "wrap",
                        marginTop: "0.25rem",
                      }}
                    >
                      {post.author && (
                        <span
                          style={{
                            fontFamily: "var(--font-mono)",
                            fontSize: "0.55rem",
                            fontWeight: 700,
                            textTransform: "uppercase",
                            letterSpacing: "0.08em",
                            color: "var(--amber)",
                            border: "1px solid rgba(255, 85, 0, 0.3)",
                            padding: "0.15rem 0.5rem",
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
                            fontSize: "0.55rem",
                            fontWeight: 700,
                            textTransform: "uppercase",
                            letterSpacing: "0.08em",
                            color: "var(--steel)",
                            border: "1px solid var(--border)",
                            padding: "0.15rem 0.5rem",
                          }}
                        >
                          {t}
                        </span>
                      ))}
                    </div>
                  </div>

                  <span
                    style={{
                      fontFamily: "var(--font-mono)",
                      fontSize: "0.65rem",
                      color: "var(--amber)",
                      paddingTop: "3px",
                    }}
                  >
                    →
                  </span>
                </Link>
              ))}
            </div>
          )}
        </section>
      </main>
    </div>
  );
}
