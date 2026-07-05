import { createLazyRoute, Link } from "@tanstack/react-router";
import { useMemo } from "react";
import { NbSubpageHero } from "@/components/NbSubpageHero";
import { getAllPosts } from "../../lib/blog";
import "../../styles/blog.css";

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
          <h2 className="blog-heading">Posts — {posts.length} articles</h2>
          <div className="nb-divider" />

          {posts.length === 0 ? (
            <div className="nb-frame blog-frame-empty">
              <span className="blog-empty-text">No posts yet. Check back soon.</span>
            </div>
          ) : (
            <div className="nb-grid blog-post-list">
              {posts.map((post) => (
                <Link
                  key={post.slug}
                  to="/blog/$slug"
                  params={{ slug: post.slug }}
                  className="nb-cell blog-post-link"
                >
                  <span className="blog-post-date">{post.date}</span>

                  <div className="blog-post-body">
                    <h2 className="blog-post-title">{post.title}</h2>
                    {post.description && <p className="blog-post-desc">{post.description}</p>}
                    <div className="blog-post-tags">
                      {post.author && <span className="blog-tag-author">{post.author}</span>}
                      {post.tags?.map((t) => (
                        <span key={t} className="blog-tag">
                          {t}
                        </span>
                      ))}
                    </div>
                  </div>

                  <span className="nb-arrow blog-arrow" />
                </Link>
              ))}
            </div>
          )}
        </div>
      </section>
    </div>
  );
}

export function PendingComponent() {
  return (
    <div className="blog-pending">
      <span className="blog-pending-text">Loading...</span>
    </div>
  );
}
