"use client";

import { ArrowRight, Github, MessageCircle, GitBranch, FileCheck, Scale } from "lucide-react";
import { PageHeader, PageSection } from "@/components/vanta/page-header";
import { Reveal } from "@/components/vanta/reveal";
import { VANTA } from "@/components/vanta/vanta-data";
import { useLanguage } from "@/lib/language-provider";

const CONTRIBUTE_STEPS = [
  {
    n: 1,
    titleKey: "aboutCommunity.steps.0.title",
    titleFallback: "Open an issue",
    bodyKey: "aboutCommunity.steps.0.body",
    bodyFallback:
      "Bug report, feature request, or benchmark result. Every issue is read. Link reproduction steps and Rust / Python version so we can triage fast.",
  },
  {
    n: 2,
    titleKey: "aboutCommunity.steps.1.title",
    titleFallback: "Open a PR",
    bodyKey: "aboutCommunity.steps.1.body",
    bodyFallback:
      "Fork the repo, branch off main, run cargo test && cargo clippy. Keep the diff scoped — small PRs land faster. Add a CHANGELOG entry under the right section.",
  },
  {
    n: 3,
    titleKey: "aboutCommunity.steps.2.title",
    titleFallback: "Discuss on Discord",
    bodyKey: "aboutCommunity.steps.2.body",
    bodyFallback:
      "Architecture questions, RFCs, and roadmap debates happen on Discord before they reach GitHub. Drop in, read the pinned messages, then ask away.",
  },
  {
    n: 4,
    titleKey: "aboutCommunity.steps.3.title",
    titleFallback: "Ship and credit",
    bodyKey: "aboutCommunity.steps.3.body",
    bodyFallback:
      "Merged PRs ship in the next release. Contributors are credited in CHANGELOG and the release notes. No CLA, no copyright transfer — Apache 2.0 stays Apache 2.0.",
  },
];

export default function CommunityPage() {
  const { t } = useLanguage();
  const tt = (key: string, fallback: string) => {
    const v = t(key);
    return v === key ? fallback : v;
  };
  return (
    <div className="animate-rise">
      <PageHeader
        badge="§COMMUNITY"
        title={tt("aboutCommunityPage.title", "Community")}
        subtitle={tt("aboutCommunityPage.subtitle", "VantaDB is built in the open. Discord for conversation, GitHub for code, Apache 2.0 for everything. Join in — your first PR is welcome.")}
        tag={tt("aboutCommunityPage.tag", "Discord · GitHub · Apache 2.0")}
      />

      {/* Channels */}
      <PageSection variant="cream">
        <Reveal direction="up">
          <div className="mb-8">
            <span className="inline-flex items-center gap-2 border-2 border-black bg-black px-2 py-0.5 font-tech text-[10px] font-bold uppercase tracking-[0.25em] text-[#FF5500] ">
              <span className="h-1.5 w-1.5 bg-[#FF5500]" />
              {tt("aboutCommunity.channelsTag", "Channels")}
            </span>
            <h2 className="glitch-hover mt-3 font-display text-3xl uppercase leading-none text-black  sm:text-4xl">
              {tt("aboutCommunity.channelsTitle", "Where to find us")}
            </h2>
          </div>
        </Reveal>

        <div className="grid grid-cols-1 gap-5 sm:grid-cols-2">
          <Reveal direction="right" as="article">
            <a
              href={VANTA.discord}
              target="_blank"
              rel="noopener noreferrer"
              className="press-lg group flex h-full flex-col border-4 border-black bg-[#F2EDE2] p-6 shadow-[6px_6px_0_0_#000] transition-transform hover:translate-x-[-3px] hover:translate-y-[-3px] hover:shadow-[9px_9px_0_0_#FF5500]   "
            >
              <div className="flex items-center gap-3">
                <span className="inline-flex h-14 w-14 items-center justify-center border-4 border-black bg-[#FF5500] text-black shadow-[3px_3px_0_0_#000] transition-transform group-hover:rotate-[-6deg]  ">
                  <MessageCircle className="h-7 w-7" strokeWidth={2.5} />
                </span>
                <div>
                  <h3 className="glitch-hover font-display text-2xl uppercase leading-none text-black ">
                    Discord
                  </h3>
                  <span className="font-tech text-[10px] uppercase tracking-wider text-[#FF5500]">
                    {tt("aboutCommunity.discordTag", "Live chat · RFCs · help")}
                  </span>
                </div>
              </div>
              <p className="mt-4 flex-1 font-tech text-xs leading-relaxed text-black/80 ">
                {tt(
                  "aboutCommunity.discordBody",
                  "The fastest place to get an answer. Architecture debates, edge cases, and benchmark results all land here first. Pinned messages have the FAQ."
                )}
              </p>
              <div className="mt-5 flex items-center justify-between border-t-4 border-black pt-4 font-tech text-xs font-bold uppercase tracking-wider text-black  ">
                <span>{tt("aboutCommunity.openDiscord", "Open Discord")}</span>
                <ArrowRight className="h-4 w-4 text-[#FF5500] transition-transform group-hover:translate-x-1" strokeWidth={2.5} aria-hidden />
              </div>
            </a>
          </Reveal>

          <Reveal direction="left" as="article">
            <a
              href={VANTA.repo}
              target="_blank"
              rel="noopener noreferrer"
              className="press-lg group flex h-full flex-col border-4 border-[#FF5500] bg-[#FBF9F5] p-6 shadow-[6px_6px_0_0_#FF5500] transition-transform hover:translate-x-[-3px] hover:translate-y-[-3px] hover:shadow-[9px_9px_0_0_#000] "
            >
              <div className="flex items-center gap-3">
                <span className="inline-flex h-14 w-14 items-center justify-center border-4 border-black bg-[#FF5500] text-black shadow-[3px_3px_0_0_#000] transition-transform group-hover:rotate-[-6deg]  ">
                  <Github className="h-7 w-7" strokeWidth={2.5} />
                </span>
                <div>
                  <h3 className="glitch-hover font-display text-2xl uppercase leading-none text-black ">
                    GitHub
                  </h3>
                  <span className="font-tech text-[10px] uppercase tracking-wider text-[#FF5500]">
                    {tt("aboutCommunity.githubTag", "Code · issues · PRs")}
                  </span>
                </div>
              </div>
              <p className="mt-4 flex-1 font-tech text-xs leading-relaxed text-black/80 ">
                {tt(
                  "aboutCommunity.githubBody",
                  "Source, issues, pull requests, releases. Star it, watch it, fork it. The repo is the single source of truth — Discord decisions get written back here."
                )}
              </p>
              <div className="mt-5 flex items-center justify-between border-t-4 border-black pt-4 font-tech text-xs font-bold uppercase tracking-wider text-black  ">
                <span>{tt("aboutCommunity.openGithub", "Open GitHub")}</span>
                <ArrowRight className="h-4 w-4 text-[#FF5500] transition-transform group-hover:translate-x-1" strokeWidth={2.5} aria-hidden />
              </div>
            </a>
          </Reveal>
        </div>
      </PageSection>

      {/* How to contribute */}
      <PageSection variant="paper">
        <Reveal direction="up">
          <div className="mb-8">
            <span className="inline-flex items-center gap-2 border-2 border-black bg-black px-2 py-0.5 font-tech text-[10px] font-bold uppercase tracking-[0.25em] text-[#FF5500] ">
              <GitBranch className="h-3 w-3" strokeWidth={3} aria-hidden />
              {tt("aboutCommunity.contributeTag", "Contribute")}
            </span>
            <h2 className="glitch-hover mt-3 font-display text-3xl uppercase leading-none text-black  sm:text-4xl">
              {tt("aboutCommunity.contributeTitle", "Four steps to your first PR")}
            </h2>
            <p className="mt-2 max-w-2xl font-tech text-xs text-black/70 ">
              {tt("aboutCommunity.contributeSubtitle", "No CLA. No telemetry. No gatekeeping. Small PRs welcome.")}
            </p>
          </div>
        </Reveal>

        <div className="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-4">
          {CONTRIBUTE_STEPS.map((step, i) => (
            <Reveal key={step.n} direction="up" delay={i * 80} as="article">
              <article className="press h-full border-4 border-black bg-[#FBF9F5] p-4  ">
                <span className="inline-flex h-8 w-8 items-center justify-center border-2 border-black bg-[#FF5500] font-display text-sm text-black ">
                  {step.n}
                </span>
                <h3 className="glitch-hover mt-3 font-display text-lg uppercase leading-none text-black ">
                  {tt(step.titleKey, step.titleFallback)}
                </h3>
                <p className="mt-2 font-tech text-xs leading-relaxed text-black/80 ">
                  {tt(step.bodyKey, step.bodyFallback)}
                </p>
                <div className="mt-3 h-1 w-full speed-lines opacity-30" aria-hidden />
              </article>
            </Reveal>
          ))}
        </div>

        {/* License */}
        <Reveal direction="up" delay={120}>
          <div className="mt-10 border-4 border-black bg-[#F2EDE2] p-6 shadow-[6px_6px_0_0_#000]   ">
            <div className="flex flex-col gap-4 sm:flex-row sm:items-start">
              <span className="inline-flex h-12 w-12 shrink-0 items-center justify-center border-4 border-black bg-black text-[#FF5500] shadow-[3px_3px_0_0_#FF5500] ">
                <Scale className="h-6 w-6" strokeWidth={2.5} />
              </span>
              <div>
                <span className="inline-flex items-center gap-2 font-tech text-[10px] font-bold uppercase tracking-[0.25em] text-[#FF5500]">
                  <FileCheck className="h-3 w-3" strokeWidth={3} aria-hidden />
                  {tt("aboutCommunity.licenseTag", "License")}
                </span>
                <h3 className="glitch-hover mt-2 font-display text-2xl uppercase leading-none text-black ">
                  {tt("aboutCommunity.licenseTitle", "Apache 2.0 — fork it, ship it")}
                </h3>
                <p className="mt-3 max-w-3xl font-tech text-sm leading-relaxed text-black/80 ">
                  {tt(
                    "aboutCommunity.licenseBody",
                    "The entire engine, CLI, Python bindings, and documentation are licensed under Apache 2.0. No CLA, no telemetry, no analytics, no phone-home. Audit every line. If we ever change direction, fork us — the license lets you."
                  )}
                </p>
              </div>
            </div>
          </div>
        </Reveal>
      </PageSection>
    </div>
  );
}
