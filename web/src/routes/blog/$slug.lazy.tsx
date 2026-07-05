import { createLazyRoute, Link } from "@tanstack/react-router";
import { useRef } from "react";
import { getPostBySlug } from "../../lib/blog";
import { useScrollReveal } from "@/hooks/useScrollReveal";
import { NbSection, NbSectionHeader } from "@/components/nb";
import { gsap } from "@/lib/gsap";
import { useAnimationSafe } from "@/hooks/useAnimationSafe";
import { fadeUp, scrollTriggerConfig } from "@/lib/gsap-utils";
import DOMPurify from "dompurify";
import "../../styles/blog.css";

export const Route = createLazyRoute("/blog/$slug")({
  component: BlogPost,
});

function BlogPost() {
  const { slug } = Route.useParams();
  const post = getPostBySlug(slug);
  const postRef = useRef<HTMLElement>(null);

  useScrollReveal();

  useAnimationSafe(() => {
    const parts = gsap.utils.toArray<HTMLElement>(".nb-engine-part");
    if (!parts.length) return;
    const tl = gsap.timeline({ scrollTrigger: scrollTriggerConfig(postRef.current, 60) });
    parts.forEach((part) => tl.add(fadeUp(part, { stagger: 0 }), "-=0.15"));
  }, postRef);

  if (!post) {
    return (
      <div className="nb-page">
        <NbSection ariaLabel="Post not found">
          <div className="nb-frame blog-slug-not-found-frame nb-engine-part">
            <span className="nb-card-frame-title">Post not found</span>
            <br />
            <Link to="/blog" className="nb-arrow">
              Back to blog
            </Link>
          </div>
        </NbSection>
      </div>
    );
  }

  return (
    <div className="nb-page">
      <NbSection ref={postRef} ariaLabel="Blog post">
        <div className="blog-slug-breadcrumb">
          <span>Blog</span>
          <span>{post.slug}</span>
        </div>

        <NbSectionHeader
          monoLabel={post.date}
          headline={post.title}
          sub={post.description || undefined}
        />

        <div className="blog-slug-meta">
          {post.author && <span className="blog-slug-author">By {post.author}</span>}
          {post.tags?.map((t) => (
            <span key={t} className="blog-slug-tag">
              {t}
            </span>
          ))}
        </div>

        <div className="nb-divider" />

        <div
          className="article-body blog-slug-body nb-engine-part"
          dangerouslySetInnerHTML={{ __html: DOMPurify.sanitize(post.html) }}
        />

        <div className="nb-divider blog-slug-divider" />

        <nav className="blog-slug-nav">
          <Link to="/blog" className="nb-arrow">
            Back to blog
          </Link>
        </nav>
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
