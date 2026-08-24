"use client";

import { ArrowRight, Github, User } from "lucide-react";
import { PageHeader, PageSection } from "@/components/vanta/page-header";
import { Reveal } from "@/components/vanta/reveal";
import { TEAM_MEMBERS, VANTA } from "@/components/vanta/vanta-data";
import { useLanguage } from "@/lib/language-provider";

export default function TeamPage() {
  const { tt } = useLanguage();
  return (
    <div className="animate-rise">
      <PageHeader
        badge="§TEAM"
        title={tt("aboutTeamPage.title", "Team")}
        subtitle={tt("aboutTeamPage.subtitle", "A founder, a mascot, and a community. VantaDB is shaped by everyone who files an issue, opens a PR, or asks a question on Discord.")}
        tag={tt("aboutTeamPage.tag", "4 contributors · open source")}
      />

      <PageSection variant="cream">
        <Reveal direction="up">
          <div className="mb-8">
            <span className="inline-flex items-center gap-2 border-2 border-black bg-black px-2 py-0.5 font-tech text-[10px] font-bold uppercase tracking-[0.25em] text-[#FF5500] ">
              <span className="h-1.5 w-1.5 bg-[#FF5500]" />
              {tt("aboutTeam.peopleTag", "People")}
            </span>
            <h2 className="glitch-hover mt-3 font-display text-3xl uppercase leading-none text-black  sm:text-4xl">
              {tt("aboutTeam.peopleTitle", "Who builds VantaDB")}
            </h2>
            <p className="mt-2 max-w-2xl font-tech text-xs text-black/70 ">
              {tt("aboutTeam.peopleSubtitle", "Two humans, one cat, one license. The community card represents everyone else — and there is room for you in it.")}
            </p>
          </div>
        </Reveal>

        <div className="grid grid-cols-1 gap-5 sm:grid-cols-2">
          {TEAM_MEMBERS.map((member, i) => (
            <Reveal key={i} direction="up" delay={i * 80} as="article">
              <article className="press-lg group relative flex h-full flex-col border-4 border-black bg-[#F2EDE2] p-5 shadow-[6px_6px_0_0_#000] transition-transform hover:shadow-[6px_6px_0_0_#FF5500]   ">
                <span className="absolute -left-2 -top-3 rotate-[-6deg] border-2 border-black bg-black px-2 py-0.5 font-display text-xs uppercase text-[#FF5500]   ">
                  0{i + 1}
                </span>

                <div className="flex items-center gap-4">
                  {/* Avatar */}
                  <div className="shrink-0">
                    {member.avatar ? (
                      <div className="relative h-20 w-20 overflow-hidden border-4 border-black bg-[#FF5500] shadow-[3px_3px_0_0_#000]  ">
                        <img
                          src={member.avatar}
                          alt={`Avatar of ${member.name}`}
                          className="h-full w-full object-cover"
                          loading="lazy"
                        />
                        <div
                          className="pointer-events-none absolute inset-0 grid-tech opacity-30"
                          aria-hidden
                        />
                      </div>
                    ) : (
                      <div className="relative inline-flex h-20 w-20 items-center justify-center border-4 border-black bg-[#FF5500] text-black shadow-[3px_3px_0_0_#000]  ">
                        <User className="h-9 w-9" strokeWidth={2.5} aria-hidden />
                        <div
                          className="pointer-events-none absolute inset-0 grid-tech opacity-30"
                          aria-hidden
                        />
                      </div>
                    )}
                  </div>
                  <div className="min-w-0 flex-1">
                    <h3 className="glitch-hover font-display text-xl uppercase leading-none text-black  sm:text-2xl">
                      {member.name}
                    </h3>
                    <p className="mt-1 font-tech text-[10px] font-bold uppercase tracking-wider text-black">
                      {tt(`aboutTeam.members.${i}.role`, member.role)}
                    </p>
                  </div>
                </div>

                <p className="mt-4 flex-1 font-tech text-xs leading-relaxed text-black/80 ">
                  {tt(`aboutTeam.members.${i}.bio`, member.bio)}
                </p>

                <div className="mt-5 border-t-4 border-black pt-4 ">
                  {member.github ? (
                    <a
                      href={`https://github.com/${member.github}`}
                      target="_blank"
                      rel="noopener noreferrer"
                      className="press inline-flex w-full items-center justify-between gap-2 border-4 border-black bg-black px-3 py-2.5 font-tech text-xs font-bold uppercase tracking-wider text-[#FF5500] transition-colors hover:bg-[#FF5500] hover:text-black     "
                    >
                      <span className="inline-flex items-center gap-2">
                        <Github className="h-3.5 w-3.5" strokeWidth={2.5} aria-hidden />
                        @{member.github}
                      </span>
                      <ArrowRight className="h-3.5 w-3.5" strokeWidth={2.5} aria-hidden />
                    </a>
                  ) : (
                    <div className="flex items-center justify-center gap-2 border-4 border-dashed border-black/30 px-3 py-2.5 font-tech text-[10px] font-bold uppercase tracking-wider text-black/70  ">
                      <Github className="h-3.5 w-3.5" strokeWidth={2.5} aria-hidden />
                      {tt("aboutTeam.noGithub", "No GitHub — prefers nap time")}
                    </div>
                  )}
                </div>

                <div className="mt-4 h-1 w-full speed-lines opacity-30" aria-hidden />
              </article>
            </Reveal>
          ))}
        </div>

        {/* Join CTA */}
        <Reveal direction="up" delay={120}>
          <div className="mt-10 flex flex-col items-center justify-between gap-4 border-4 border-black bg-black p-6 text-[#FBF9F5] shadow-[6px_6px_0_0_#FF5500]  sm:flex-row">
            <div>
              <h3 className="font-display text-2xl uppercase leading-none text-[#FBF9F5]">
                {tt("aboutTeam.joinCtaTitle", "Join the team")}
              </h3>
              <p className="mt-1 font-tech text-xs text-[#FBF9F5]/60">
                {tt("aboutTeam.joinCtaBody", "Open an issue, send a PR, or just say hi on Discord. All contributions welcome.")}
              </p>
            </div>
            <a
              href={VANTA.discord}
              target="_blank"
              rel="noopener noreferrer"
              className="press-neon btn-neon-glow inline-flex shrink-0 items-center gap-2 border-4 border-[#FBF9F5] bg-[#FF5500] px-5 py-3 font-tech text-sm font-bold uppercase tracking-wider text-black"
            >
              {tt("aboutTeam.joinDiscord", "Join Discord")}
              <ArrowRight className="h-4 w-4" strokeWidth={2.5} aria-hidden />
            </a>
          </div>
        </Reveal>
      </PageSection>
    </div>
  );
}
