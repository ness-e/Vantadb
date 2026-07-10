import { createLazyRoute, Link } from "@tanstack/react-router";
import { useMemo, useRef } from "react";
import { NbSubpageHero } from "@/components/NbSubpageHero";
import { NbSection, NbSectionHeader } from "@/components/nb";
import { useAnimationSafe } from "@/hooks/useAnimationSafe";
import { fadeUp } from "@/lib/motion-utils";
import { getAllPosts } from "../../lib/blog";
import "../../styles/blog.css";

export const Route = createLazyRoute("/blog/")({
  component: BlogIndex,
});

function BlogIndex() {
  const posts = useMemo(() => getAllPosts(), []);
  const postsRef = useRef<HTMLElement>(null);

  useAnimationSafe(() => {
    const parts = postsRef.current?.querySelectorAll<HTMLElement>(".nc-bl-part");
    if (!parts?.length) return;
    fadeUp(parts, { stagger: 0.2 });
  }, postsRef);

  return (
    <div className="nb-page">
      <NbSubpageHero
        pattern="p13"
        title={
          <span>
            Deep dives.
            <br />
            Release notes.
          </span>
        }
        sub="Engineering blog about embedded vector databases, AI agents, local RAG, and the future of AI data infrastructure."
      />

      <NbSection ref={postsRef} ariaLabel="Blog posts">
        <NbSectionHeader
          monoLabel="[BLOG]"
          headline={`${posts.length} articles.`}
          sub="Latest posts from the VantaDB engineering team."
        />

        {posts.length === 0 ? (
          <div className="nb-frame nc-bl-frame nc-bl-frame--empty nc-bl-part">
            <span className="nb-section-sub">No posts yet. Check back soon.</span>
          </div>
        ) : (
          <div className="nc-bl-list nc-bl-part">
            {posts.map((post) => (
              <Link
                key={post.slug}
                to="/blog/$slug"
                params={{ slug: post.slug }}
                className="nb-cell nc-bl-link"
              >
                <span className="nc-bl-date">{post.date}</span>

                <div className="nc-bl-body">
                  <h2 className="nb-card-frame-title">{post.title}</h2>
                  {post.description && <p className="nb-card-frame-desc">{post.description}</p>}
                  <div className="nc-bl-tags">
                    {post.author && (
                      <span className="nc-bl-tag nc-bl-tag--author">{post.author}</span>
                    )}
                    {post.tags?.map((t) => (
                      <span key={t} className="nc-bl-tag">
                        {t}
                      </span>
                    ))}
                  </div>
                </div>

                <span className="nb-arrow nc-bl-arrow" />
              </Link>
            ))}
          </div>
        )}
      </NbSection>
    </div>
  );
}

export function PendingComponent() {
  return (
    <div className="nc-bl-pending">
      <span className="nc-bl-pending-text">Loading...</span>
    </div>
  );
}
