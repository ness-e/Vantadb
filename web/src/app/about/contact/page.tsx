"use client";

import {
  ArrowRight,
  Github,
  MessageCircle,
  Mail,
  Bug,
  Lightbulb,
  type LucideIcon,
} from "lucide-react";
import { PageHeader, PageSection } from "@/components/vanta/page-header";
import { Reveal } from "@/components/vanta/reveal";
import { VANTA } from "@/components/vanta/vanta-data";
import { useLanguage } from "@/lib/language-provider";

type Channel = {
  icon: LucideIcon;
  label: string;
  labelKey: string;
  labelFallback: string;
  value: string;
  href: string;
  description: string;
  descriptionKey: string;
  descriptionFallback: string;
  external: boolean;
};

const CHANNELS: Channel[] = [
  {
    icon: MessageCircle,
    labelKey: "aboutContact.channel.0.label",
    labelFallback: "Discord",
    label: "Discord",
    value: "VantaDB Discord",
    href: VANTA.discord,
    descriptionKey: "aboutContact.channel.0.description",
    descriptionFallback:
      "Fastest response. Architecture questions, RFCs, help getting unstuck. Pinned messages hold the FAQ.",
    description:
      "Fastest response. Architecture questions, RFCs, help getting unstuck. Pinned messages hold the FAQ.",
    external: true,
  },
  {
    icon: Bug,
    labelKey: "aboutContact.channel.1.label",
    labelFallback: "Bug reports",
    label: "Bug reports",
    value: "GitHub Issues",
    href: `${VANTA.repo}/issues/new?template=bug_report.md`,
    descriptionKey: "aboutContact.channel.1.description",
    descriptionFallback:
      "Found a crash, a correctness bug, or a perf regression? File an issue with reproduction steps and Rust / Python version.",
    description:
      "Found a crash, a correctness bug, or a perf regression? File an issue with reproduction steps and Rust / Python version.",
    external: true,
  },
  {
    icon: Lightbulb,
    labelKey: "aboutContact.channel.2.label",
    labelFallback: "Feature requests",
    label: "Feature requests",
    value: "GitHub Discussions",
    href: `${VANTA.repo}/discussions`,
    descriptionKey: "aboutContact.channel.2.description",
    descriptionFallback:
      "Have an idea? Open a discussion first — we surface design feedback before code lands. Smaller PRs land faster.",
    description:
      "Have an idea? Open a discussion first — we surface design feedback before code lands. Smaller PRs land faster.",
    external: true,
  },
  {
    icon: Mail,
    labelKey: "aboutContact.channel.3.label",
    labelFallback: "Email",
    label: "Email",
    value: "maintainers@vantadb.dev",
    href: "mailto:maintainers@vantadb.dev",
    descriptionKey: "aboutContact.channel.3.description",
    descriptionFallback:
      "Security disclosures, enterprise inquiries, or anything you'd rather not put in a public issue. PGP key on request.",
    description:
      "Security disclosures, enterprise inquiries, or anything you'd rather not put in a public issue. PGP key on request.",
    external: false,
  },
];

export default function ContactPage() {
  const { t } = useLanguage();
  const tt = (key: string, fallback: string) => {
    const v = t(key);
    return v === key ? fallback : v;
  };
  return (
    <div className="animate-rise">
      <PageHeader
        badge="§CONTACT"
        title={tt("aboutContactPage.title", "Contact")}
        subtitle={tt("aboutContactPage.subtitle", "No contact form — we keep it direct. Discord for chat, GitHub for code, email for the rest. Every channel is read by a maintainer.")}
        tag={tt("aboutContactPage.tag", "4 channels · maintainer-read")}
      />

      <PageSection variant="cream">
        <Reveal direction="up">
          <div className="mb-8">
            <span className="inline-flex items-center gap-2 border-2 border-black bg-black px-2 py-0.5 font-tech text-[10px] font-bold uppercase tracking-[0.25em] text-[#FF5500] ">
              <span className="h-1.5 w-1.5 bg-[#FF5500]" />
              {tt("aboutContact.channelsTag", "Channels")}
            </span>
            <h2 className="glitch-hover mt-3 font-display text-3xl uppercase leading-none text-black  sm:text-4xl">
              {tt("aboutContact.channelsTitle", "Pick the right door")}
            </h2>
            <p className="mt-2 max-w-2xl font-tech text-xs text-black/70 ">
              {tt("aboutContact.channelsSubtitle", "Different doors for different things. Choosing the right one shortens your time-to-answer by days.")}
            </p>
          </div>
        </Reveal>

        <div className="grid grid-cols-1 gap-5 sm:grid-cols-2">
          {CHANNELS.map((channel, i) => {
            const Icon = channel.icon;
            return (
              <Reveal key={channel.label} direction={i % 2 === 0 ? "right" : "left"} delay={i * 60} as="article">
                <a
                  href={channel.href}
                  target={channel.external ? "_blank" : undefined}
                  rel={channel.external ? "noopener noreferrer" : undefined}
                  className="press-lg group flex h-full flex-col border-4 border-black bg-[#F2EDE2] p-6 shadow-[6px_6px_0_0_#000] transition-transform hover:translate-x-[-3px] hover:translate-y-[-3px] hover:shadow-[9px_9px_0_0_#FF5500]   "
                >
                  <div className="flex items-center justify-between">
                    <span className="inline-flex h-14 w-14 items-center justify-center border-4 border-black bg-[#FF5500] text-black shadow-[3px_3px_0_0_#000] transition-transform group-hover:rotate-[-6deg]  ">
                      <Icon className="h-7 w-7" strokeWidth={2.5} />
                    </span>
                    <span className="border-2 border-black bg-[#FBF9F5] px-2 py-0.5 font-tech text-[9px] font-bold uppercase tracking-[0.2em] text-black/70   ">
                      {tt(channel.labelKey, channel.labelFallback)}
                    </span>
                  </div>

                  <h3 className="glitch-hover mt-4 font-display text-xl uppercase leading-none text-black  sm:text-2xl">
                    {channel.value}
                  </h3>

                  <p className="mt-3 flex-1 font-tech text-xs leading-relaxed text-black/80 ">
                    {tt(channel.descriptionKey, channel.descriptionFallback)}
                  </p>

                  <div className="mt-5 flex items-center justify-between border-t-4 border-black pt-4 ">
                    <span className="font-tech text-[10px] font-bold uppercase tracking-[0.2em] text-black/60 ">
                      {tt("aboutContact.openLabel", "Open")} {tt(channel.labelKey, channel.labelFallback).toLowerCase()}
                    </span>
                    <span className="inline-flex h-9 w-9 items-center justify-center border-2 border-black bg-[#FF5500] text-black transition-transform group-hover:translate-x-1 ">
                      <ArrowRight className="h-4 w-4" strokeWidth={2.5} aria-hidden />
                    </span>
                  </div>

                  <div className="mt-4 h-1 w-full speed-lines opacity-30" aria-hidden />
                </a>
              </Reveal>
            );
          })}
        </div>

        {/* No form note */}
        <Reveal direction="up" delay={120}>
          <div className="mt-10 border-4 border-dashed border-black/30 p-5 text-center ">
            <p className="font-tech text-xs leading-relaxed text-black/60 ">
              {tt("aboutContact.noFormNote", "We deliberately don't run a contact form. Forms route to nowhere. The four links above route to maintainers. Use them — your message will be read.")}
            </p>
          </div>
        </Reveal>

        {/* Quick CTA */}
        <Reveal direction="up" delay={150}>
          <div className="mt-8 flex flex-col items-center justify-between gap-4 border-4 border-black bg-black p-6 text-[#FBF9F5] shadow-[6px_6px_0_0_#FF5500]  sm:flex-row">
            <div className="flex items-center gap-3">
              <Github className="h-6 w-6 text-[#FF5500]" strokeWidth={2.5} aria-hidden />
              <div>
                <h3 className="font-display text-xl uppercase leading-none text-[#FBF9F5]">
                  {tt("aboutContact.codeCtaTitle", "Just want the code?")}
                </h3>
                <p className="mt-1 font-tech text-xs text-[#FBF9F5]/60">
                  {tt("aboutContact.codeCtaBody", "Skip the channels — clone the repo, read the source.")}
                </p>
              </div>
            </div>
            <a
              href={VANTA.repo}
              target="_blank"
              rel="noopener noreferrer"
              className="press-neon btn-neon-glow inline-flex shrink-0 items-center gap-2 border-4 border-[#FBF9F5] bg-[#FF5500] px-5 py-3 font-tech text-sm font-bold uppercase tracking-wider text-black"
            >
              {tt("aboutContact.viewOnGithub", "View on GitHub")}
              <ArrowRight className="h-4 w-4" strokeWidth={2.5} aria-hidden />
            </a>
          </div>
        </Reveal>
      </PageSection>
    </div>
  );
}
