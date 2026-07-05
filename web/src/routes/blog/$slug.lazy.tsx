import { createLazyRoute, Link } from "@tanstack/react-router";
import { getPostBySlug } from "../../lib/blog";
import { useScrollReveal } from "@/hooks/useScrollReveal";
import DOMPurify from "dompurify";

export const Route = createLazyRoute("/blog/$slug")({
  component: BlogPost,
});

function BlogPost() {
  const { slug } = Route.useParams();
  const post = getPostBySlug(slug);

  useScrollReveal();

  if (!post) {
    return (
      <div className="nb-page">
        <section className="nb-section">
          <div className="nb-inner">
            <div className="nb-frame" style={{ padding: "var(--space-3xl)", textAlign: "center" }}>
              <span
                style={{
                  fontFamily: "var(--font-display)",
                  fontSize: "var(--text-display)",
                  color: "var(--amber)",
                  fontWeight: 700,
                  marginBottom: "var(--space-md)",
                  display: "block",
                }}
              >
                Post not found
              </span>
              <br />
              <Link to="/blog" className="nb-arrow">
                Back to blog
              </Link>
            </div>
          </div>
        </section>
      </div>
    );
  }

  return (
    <div className="nb-page">
      <section className="nb-section">
        <div className="nb-inner">
          <div style={{ marginBottom: "var(--space-md)" }}>
            <span>Blog</span>
            <span>{post.slug}</span>
          </div>

          <div style={{ marginBottom: "var(--space-xl)" }}>
            <span
              style={{
                fontFamily: "var(--font-mono)",
                fontSize: "var(--text-label)",
                color: "var(--amber)",
                fontWeight: 700,
                marginBottom: 0,
              }}
            >
              {post.date}
            </span>
            <h1
              style={{
                fontFamily: "var(--font-display)",
                fontSize: "var(--text-hero)",
                fontWeight: 700,
                letterSpacing: "var(--tracking-tight)",
                lineHeight: 1.05,
                color: "var(--foreground)",
                margin: "var(--space-sm) 0",
              }}
            >
              {post.title}
            </h1>
            <div
              style={{
                display: "flex",
                gap: "var(--space-sm)",
                alignItems: "center",
                marginBottom: "var(--space-md)",
              }}
            >
              {post.author && (
                <span
                  style={{
                    fontFamily: "var(--font-mono)",
                    fontSize: "var(--text-label)",
                    color: "var(--amber)",
                    fontWeight: 700,
                    marginBottom: 0,
                  }}
                >
                  By {post.author}
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
            {post.description && (
              <p
                style={{
                  fontFamily: "var(--font-sans)",
                  fontSize: "var(--text-lead)",
                  color: "var(--muted)",
                  lineHeight: 1.6,
                  margin: 0,
                  maxWidth: "65ch",
                }}
              >
                {post.description}
              </p>
            )}
          </div>

          <div className="nb-divider" />

          <div
            className="article-body"
            style={{
              fontFamily: "var(--font-sans)",
              fontSize: "var(--text-body)",
              lineHeight: 1.8,
              color: "var(--foreground)",
              marginTop: "var(--space-xl)",
              maxWidth: "75ch",
            }}
            dangerouslySetInnerHTML={{ __html: DOMPurify.sanitize(post.html) }}
          />

          <div className="nb-divider" style={{ marginTop: "var(--space-xl)" }} />

          <nav style={{ marginTop: "var(--space-lg)" }}>
            <Link to="/blog" className="nb-arrow">
              Back to blog
            </Link>
          </nav>
        </div>
      </section>

      <style>{`
        .article-body h2 { font-family: var(--font-display); font-size: var(--text-display); font-weight: 700; letter-spacing: var(--tracking-display); margin-top: var(--space-2xl); margin-bottom: var(--space-sm); }
        .article-body h3 { font-family: var(--font-display); font-size: var(--text-title); font-weight: 700; letter-spacing: var(--tracking-display); margin-top: var(--space-xl); margin-bottom: var(--space-sm); }
        .article-body p { margin-bottom: var(--space-sm); }
        .article-body code { font-family: var(--font-mono); font-size: var(--text-code); background: var(--surface); padding: 2px 6px; border: 1px solid var(--border-visible); }
        .article-body pre { background: var(--surface); border: 2px solid var(--border-visible); padding: var(--space-md); overflow-x: auto; margin: var(--space-md) 0; }
        .article-body pre code { background: none; border: none; padding: 0; }
        .article-body a { color: var(--amber); text-decoration: underline; text-underline-offset: 3px; }
        .article-body blockquote { border-left: 3px solid var(--amber); margin: var(--space-md) 0; padding: var(--space-sm) var(--space-md); background: var(--surface); font-style: italic; }
        .article-body ul, .article-body ol { padding-left: var(--space-lg); margin-bottom: var(--space-sm); }
        .article-body li { margin-bottom: var(--space-2xs); }
        .article-body img { border: 2px solid var(--border-visible); max-width: 100%; height: auto; margin: var(--space-md) 0; }
        .article-body hr { border: none; border-top: 2px solid var(--border-visible); margin: var(--space-xl) 0; }
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
