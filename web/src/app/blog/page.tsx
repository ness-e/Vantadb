"use client";

import { ArrowRight, Clock, User } from "lucide-react";
import { PageHeader, PageSection } from "@/components/vanta/page-header";
import { Reveal } from "@/components/vanta/reveal";
import { BLOG_POSTS } from "@/components/vanta/vanta-data";
import { cn } from "@/lib/utils";
import { useLanguage } from "@/lib/language-provider";

const TAG_STYLES: Record<string, string> = {
  ink: "bg-black text-[#FF5500] border-black   ",
  neon: "bg-[#FF5500] text-black border-black",
  muted:
    "bg-[#F2EDE2] text-black/70 border-black   ",
};

export default function BlogPage() {
  const { tt } = useLanguage();
  return (
    <div className="animate-rise">
      <PageHeader
        badge="§BLOG"
        title={tt("blogPage.title", "Blog")}
        subtitle={tt("blogPage.subtitle", "Engineering deep-dives, architecture notes, and stories from building an embedded Rust retrieval engine. Local-first, written in plain text.")}
        tag={tt("blogPage.tag", "Engineering · Story")}
      />

      <PageSection variant="cream">
        <Reveal direction="up">
          <div className="mb-8 flex flex-col gap-3 sm:flex-row sm:items-end sm:justify-between">
            <div>
              <span className="inline-flex items-center gap-2 border-2 border-black bg-black px-2 py-0.5 font-tech text-[10px] font-bold uppercase tracking-[0.25em] text-[#FF5500] ">
                <span className="h-1.5 w-1.5 bg-[#FF5500]" />
                {tt("blogPage.postsTag", "Posts")}
              </span>
              <h2 className="glitch-hover mt-3 font-display text-3xl uppercase leading-none text-black  sm:text-4xl">
                {tt("blogPage.latestTitle", "Latest writing")}
              </h2>
              <p className="mt-2 max-w-2xl font-tech text-xs text-black/70 ">
                {tt("blogPage.latestSubtitle", "{count} posts on hybrid search, agent memory, and the local-first thesis. No tracking, no paywall.").replace("{count}", String(BLOG_POSTS.length))}
              </p>
            </div>
          </div>
        </Reveal>

        <div className="grid grid-cols-1 gap-5 sm:grid-cols-2">
          {BLOG_POSTS.map((post, i) => (
            <Reveal key={post.slug} direction="up" delay={i * 60} as="article">
              <a
                href={`/blog/${post.slug}`}
                className="press-lg group relative flex h-full flex-col border-4 border-black bg-[#F2EDE2] p-5 shadow-[6px_6px_0_0_#000] transition-transform hover:translate-x-[-3px] hover:translate-y-[-3px] hover:shadow-[9px_9px_0_0_#FF5500]   "
              >
                <span
                  className={cn(
                    "absolute -left-2 -top-3 rotate-[-6deg] self-start border-2 border-black px-2 py-0.5 font-tech text-[9px] font-bold uppercase tracking-[0.2em] ",
                    TAG_STYLES[post.tagColor] ?? TAG_STYLES.muted
                  )}
                >
                  {tt(`blogPost.data.${i}.tag`, post.tag)}
                </span>

                <div className="mt-2 flex items-center gap-3 font-tech text-[10px] uppercase tracking-wider text-black/60 ">
                  <span className="inline-flex items-center gap-1">
                    <User className="h-3 w-3" strokeWidth={2.5} aria-hidden />
                    {tt(`blogPost.data.${i}.author`, post.author)}
                  </span>
                  <span aria-hidden>·</span>
                  <span>{post.date}</span>
                  <span aria-hidden>·</span>
                  <span className="inline-flex items-center gap-1">
                    <Clock className="h-3 w-3" strokeWidth={2.5} aria-hidden />
                    {post.readTime}
                  </span>
                </div>

                <h3 className="glitch-hover mt-3 font-display text-2xl uppercase leading-none text-black  sm:text-3xl">
                  {post.title}
                </h3>

                <p className="mt-3 flex-1 font-tech text-xs leading-relaxed text-black/80 ">
                  {tt(`blogPost.data.${i}.excerpt`, post.excerpt)}
                </p>

                <div className="mt-5 flex items-center justify-between border-t-4 border-black pt-4 ">
                  <span className="font-tech text-[10px] font-bold uppercase tracking-[0.2em] text-black/60 ">
                    {tt("blogPage.readPost", "Read post")}
                  </span>
                  <span className="inline-flex h-9 w-9 items-center justify-center border-2 border-black bg-[#FF5500] text-black transition-transform group-hover:translate-x-1 ">
                    <ArrowRight className="h-4 w-4" strokeWidth={2.5} aria-hidden />
                  </span>
                </div>

                <div className="mt-4 h-1 w-full speed-lines opacity-30" aria-hidden />
              </a>
            </Reveal>
          ))}
        </div>
      </PageSection>
    </div>
  );
}
