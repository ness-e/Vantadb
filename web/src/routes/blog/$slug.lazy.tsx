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
      <div className="page-wrapper">
        <div className="page-header-extended reveal">
          <span className="section-eyebrow">// Blog</span>
          <h1 className="section-title">Post not found</h1>
          <p className="section-sub">
            <Link to="/blog" className="nav-cta">
              ← Back to blog
            </Link>
          </p>
        </div>
      </div>
    );
  }

  return (
    <div className="page-wrapper">
      <main className="main-content">
        <article style={{ padding: "6rem 0" }}>
          <div className="reveal" style={{ marginBottom: "3rem" }}>
            <span className="section-eyebrow">// {post.date}</span>
            <h1
              className="section-title section-title--compact"
              style={{ fontSize: "var(--text-hero)", lineHeight: 1.05 }}
            >
              {post.title}
            </h1>
            <div
              style={{
                display: "flex",
                gap: "0.75rem",
                alignItems: "center",
                marginBottom: "1rem",
              }}
            >
              {post.author && (
                <span
                  style={{
                    fontFamily: "var(--font-mono)",
                    fontSize: "0.7rem",
                    color: "var(--amber)",
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
                    fontSize: "0.62rem",
                    color: "var(--steel)",
                    background: "var(--surface)",
                    padding: "0.15rem 0.5rem",
                  }}
                >
                  {t}
                </span>
              ))}
            </div>
            <p className="section-sub" style={{ maxWidth: 600 }}>
              {post.description}
            </p>
          </div>

          <div
            className="reveal article-body"
            dangerouslySetInnerHTML={{ __html: DOMPurify.sanitize(post.html) }}
          />
        </article>

        <nav className="bottom-nav">
          <Link to="/blog" className="back-link nav-cta">
            ← Back to blog
          </Link>
        </nav>
      </main>
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
      <div>Loading...</div>
    </div>
  );
}
