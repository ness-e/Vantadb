"use client";

import { ArrowLeft, Clock, User, Calendar, ArrowRight } from "lucide-react";
import { useParams, useRouter } from "next/navigation";
import { PageHeader, PageSection } from "@/components/vanta/page-header";
import { Reveal } from "@/components/vanta/reveal";
import { BLOG_POSTS, VANTA } from "@/components/vanta/vanta-data";
import { cn } from "@/lib/utils";
import { useLanguage } from "@/lib/language-provider";

const TAG_STYLES: Record<string, string> = {
  ink: "bg-black text-[#FF5500] border-black   ",
  neon: "bg-[#FF5500] text-black border-black",
  muted:
    "bg-[#F2EDE2] text-black/70 border-black   ",
};

export default function BlogPostPage() {
  const params = useParams();
  const router = useRouter();
  const { t } = useLanguage();
  const tt = (key: string, fallback: string) => {
    const v = t(key);
    return v === key ? fallback : v;
  };
  const slug = params.slug as string;
  const postIdx = BLOG_POSTS.findIndex((p) => p.slug === slug);
  const post = postIdx >= 0 ? BLOG_POSTS[postIdx] : undefined;

  if (!post) {
    return (
      <div className="animate-rise">
        <PageHeader
          badge="§404"
          title={tt("blogPostPage.title", "Post Not Found")}
          subtitle={tt("blogPostPage.subtitle", "This post doesn't exist or hasn't been published yet.")}
          tag={tt("blogPostPage.tag", "Blog · missing")}
        />
        <PageSection variant="cream">
          <div className="mx-auto max-w-2xl">
            <button
              onClick={() => router.push("/blog")}
              className="press inline-flex items-center gap-2 border-4 border-black bg-[#FF5500] px-5 py-3 font-tech text-xs font-bold uppercase tracking-wider text-black shadow-[6px_6px_0_0_#000] transition-colors hover:bg-black hover:text-[#FF5500]  "
            >
              <ArrowLeft className="h-4 w-4" strokeWidth={2.5} aria-hidden />
              {tt("blogPost.backToList", "Back to Blog")}
            </button>
          </div>
        </PageSection>
      </div>
    );
  }

  // Suggested next post (any other post)
  const nextPostIdx = BLOG_POSTS.findIndex((p) => p.slug !== post.slug);
  const nextPost = nextPostIdx >= 0 ? BLOG_POSTS[nextPostIdx] : null;

  return (
    <div className="animate-rise">
      <PageHeader
        badge={`§${tt(`blogPost.data.${postIdx}.tag`, post.tag).toUpperCase()}`}
        title={post.title}
        subtitle={tt(`blogPost.data.${postIdx}.excerpt`, post.excerpt)}
        tag={`${post.date} · ${post.readTime}`}
      >
        <div className="flex flex-wrap items-center gap-3 font-tech text-[11px] uppercase tracking-wider text-black/70 ">
          <span className="inline-flex items-center gap-1.5 border-2 border-black bg-[#FBF9F5] px-2.5 py-1 text-black   ">
            <User className="h-3 w-3 text-[#FF5500]" strokeWidth={2.5} aria-hidden />
            {tt(`blogPost.data.${postIdx}.author`, post.author)}
          </span>
          <span className="inline-flex items-center gap-1.5 border-2 border-black bg-[#FBF9F5] px-2.5 py-1 text-black   ">
            <Calendar className="h-3 w-3 text-[#FF5500]" strokeWidth={2.5} aria-hidden />
            {post.date}
          </span>
          <span className="inline-flex items-center gap-1.5 border-2 border-black bg-[#FBF9F5] px-2.5 py-1 text-black   ">
            <Clock className="h-3 w-3 text-[#FF5500]" strokeWidth={2.5} aria-hidden />
            {post.readTime}
          </span>
          <span
            className={cn(
              "border-2 px-2.5 py-1 font-bold uppercase tracking-[0.2em]",
              TAG_STYLES[post.tagColor] ?? TAG_STYLES.muted
            )}
          >
            {tt(`blogPost.data.${postIdx}.tag`, post.tag)}
          </span>
        </div>
      </PageHeader>

      <PageSection variant="paper">
        <article className="mx-auto max-w-3xl">
          <Reveal direction="up">
            <div className="border-4 border-black bg-[#FBF9F5] p-6 shadow-[6px_6px_0_0_#000]    sm:p-8">
              {post.content.map((block, i) => {
                if (block.type === "h2") {
                  return (
                    <h2
                      key={i}
                      className="glitch-hover mt-8 font-display text-2xl uppercase leading-none text-black  sm:text-3xl"
                    >
                      <span className="mr-2 text-[#FF5500]" aria-hidden>
                        §
                      </span>
                      {tt(`blogPost.data.${postIdx}.content.${i}`, block.text)}
                    </h2>
                  );
                }
                return (
                  <p
                    key={i}
                    className="mt-4 font-tech text-sm leading-relaxed text-black/80 first:mt-0 "
                  >
                    {tt(`blogPost.data.${postIdx}.content.${i}`, block.text)}
                  </p>
                );
              })}

              <div className="mt-10 flex flex-col gap-3 border-t-4 border-black pt-6 sm:flex-row sm:items-center sm:justify-between ">
                <button
                  onClick={() => router.push("/blog")}
                  className="press inline-flex items-center justify-center gap-2 border-4 border-black bg-black px-5 py-3 font-tech text-xs font-bold uppercase tracking-wider text-[#FF5500] transition-colors hover:bg-[#FF5500] hover:text-black     "
                >
                  <ArrowLeft className="h-4 w-4" strokeWidth={2.5} aria-hidden />
                  {tt("blogPost.backToList", "Back to Blog")}
                </button>

                <a
                  href={VANTA.repo}
                  target="_blank"
                  rel="noopener noreferrer"
                  className="press inline-flex items-center justify-center gap-2 border-4 border-dashed border-black/40 px-5 py-3 font-tech text-[11px] font-bold uppercase tracking-wider text-black/60 transition-colors hover:border-[#FF5500] hover:text-[#FF5500]  "
                >
                  {tt("blogPost.discussOnGithub", "Discuss on GitHub")}
                  <ArrowRight className="h-3.5 w-3.5" strokeWidth={2.5} aria-hidden />
                </a>
              </div>
            </div>
          </Reveal>
        </article>
      </PageSection>

      {/* Suggested next read */}
      {nextPost && (
        <PageSection variant="cream">
          <Reveal direction="up">
            <div className="mb-6 flex items-center gap-3">
              <span className="inline-flex h-2 w-2 animate-flicker bg-[#FF5500]" aria-hidden />
              <span className="font-tech text-[10px] font-bold uppercase tracking-[0.25em] text-black/60 ">
                {tt("blogPost.keepReading", "Keep reading")}
              </span>
            </div>
          </Reveal>
          <Reveal direction="up" delay={80}>
            <a
              href={`/blog/${nextPost.slug}`}
              className="press-lg group flex flex-col gap-3 border-4 border-black bg-[#F2EDE2] p-5 shadow-[6px_6px_0_0_#000] transition-transform hover:translate-x-[-3px] hover:translate-y-[-3px] hover:shadow-[9px_9px_0_0_#FF5500]    sm:flex-row sm:items-center sm:justify-between"
            >
              <div className="flex-1">
                <span className="font-tech text-[10px] uppercase tracking-wider text-[#FF5500]">
                  {tt(`blogPost.data.${nextPostIdx}.tag`, nextPost.tag)} · {nextPost.readTime}
                </span>
                <h3 className="glitch-hover mt-1 font-display text-xl uppercase leading-none text-black  sm:text-2xl">
                  {nextPost.title}
                </h3>
                <p className="mt-2 line-clamp-2 font-tech text-xs text-black/70 ">
                  {tt(`blogPost.data.${nextPostIdx}.excerpt`, nextPost.excerpt)}
                </p>
              </div>
              <span className="inline-flex h-10 w-10 shrink-0 items-center justify-center border-2 border-black bg-[#FF5500] text-black transition-transform group-hover:translate-x-1 ">
                <ArrowRight className="h-4 w-4" strokeWidth={2.5} aria-hidden />
              </span>
            </a>
          </Reveal>
        </PageSection>
      )}
    </div>
  );
}
