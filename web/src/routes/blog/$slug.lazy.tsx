import { createLazyRoute, Link } from "@tanstack/react-router";
import { getPostBySlug } from "../../lib/blog";
import { useScrollReveal } from "@/hooks/useScrollReveal";
import DOMPurify from "dompurify";
import "../../styles/blog.css";

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
            <div className="nb-frame blog-slug-not-found-frame">
              <span className="blog-slug-not-found-text">
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
          <div className="blog-slug-breadcrumb">
            <span>Blog</span>
            <span>{post.slug}</span>
          </div>

          <div className="blog-slug-header">
            <span className="blog-slug-date">
              {post.date}
            </span>
            <h1 className="blog-slug-title">
              {post.title}
            </h1>
            <div className="blog-slug-meta">
              {post.author && (
                <span className="blog-slug-author">
                  By {post.author}
                </span>
              )}
              {post.tags?.map((t) => (
                <span key={t} className="blog-slug-tag">
                  {t}
                </span>
              ))}
            </div>
            {post.description && (
              <p className="blog-slug-desc">
                {post.description}
              </p>
            )}
          </div>

          <div className="nb-divider" />

          <div
            className="article-body blog-slug-body"
            dangerouslySetInnerHTML={{ __html: DOMPurify.sanitize(post.html) }}
          />

          <div className="nb-divider blog-slug-divider" />

          <nav className="blog-slug-nav">
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
    <div className="blog-pending">
      <span className="blog-pending-text">
        Loading...
      </span>
    </div>
  );
}
