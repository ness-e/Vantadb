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
    const parts = gsap.utils.toArray<HTMLElement>(".nc-bl-part");
    if (!parts.length) return;
    const tl = gsap.timeline({ scrollTrigger: scrollTriggerConfig(postRef.current, 60) });
    parts.forEach((part) => tl.add(fadeUp(part, { stagger: 0 }), "-=0.15"));
  }, postRef);

  if (!post) {
    return (
      <div className="nb-page">
        <NbSection ariaLabel="Post not found">
          <div className="nb-frame nc-bl-frame nc-bl-part">
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
        <div className="nc-bl-breadcrumb">
          <span>Blog</span>
          <span>{post.slug}</span>
        </div>

        <NbSectionHeader
          monoLabel={post.date}
          headline={post.title}
          sub={post.description || undefined}
        />

        <div className="nc-bl-meta">
          {post.author && <span className="nc-bl-author">By {post.author}</span>}
          {post.tags?.map((t) => (
            <span key={t} className="nc-bl-meta-tag">
              {t}
            </span>
          ))}
        </div>

        <div className="nb-divider" />

        <div
          className="article-body nc-bl-body-text nc-bl-part"
          dangerouslySetInnerHTML={{ __html: DOMPurify.sanitize(post.html) }}
        />

        <div className="nb-divider nc-bl-divider" />

        <nav className="nc-bl-nav">
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
    <div className="nc-bl-pending">
      <span className="nc-bl-pending-text">Loading...</span>
    </div>
  );
}
