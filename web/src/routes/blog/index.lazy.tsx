import { createLazyRoute, Link } from "@tanstack/react-router";
import { useRef, useMemo } from "react";
import { NbSubpageHero } from "@/components/NbSubpageHero";
import { NbSection, NbSectionHeader } from "@/components/nb";
import { gsap } from "@/lib/gsap";
import { useAnimationSafe } from "@/hooks/useAnimationSafe";
import { fadeUp, scrollTriggerConfig } from "@/lib/gsap-utils";
import { getAllPosts } from "../../lib/blog";
import "../../styles/blog.css";

export const Route = createLazyRoute("/blog/")({
  component: BlogIndex,
});

function BlogIndex() {
  const posts = useMemo(() => getAllPosts(), []);
  const postsRef = useRef<HTMLElement>(null);

  useAnimationSafe(() => {
    const parts = gsap.utils.toArray<HTMLElement>(".nb-engine-part");
    if (!parts.length) return;
    const tl = gsap.timeline({ scrollTrigger: scrollTriggerConfig(postsRef.current, 60) });
    parts.forEach((part) => tl.add(fadeUp(part, { stagger: 0 }), "-=0.15"));
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
          <div className="nb-frame blog-frame-empty nb-engine-part">
            <span className="nb-section-sub">No posts yet. Check back soon.</span>
          </div>
        ) : (
          <div className="blog-post-list nb-engine-part">
            {posts.map((post) => (
              <Link
                key={post.slug}
                to="/blog/$slug"
                params={{ slug: post.slug }}
                className="nb-cell blog-post-link"
              >
                <span className="blog-post-date">{post.date}</span>

                <div className="blog-post-body">
                  <h2 className="nb-card-frame-title">{post.title}</h2>
                  {post.description && <p className="nb-card-frame-desc">{post.description}</p>}
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
      </NbSection>
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
