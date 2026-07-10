import { createLazyRoute, Link } from "@tanstack/react-router";
import { useRef } from "react";
import { NbSubpageHero } from "@/components/NbSubpageHero";
import { NbBlockAmber, NbSection, NbSectionHeader } from "@/components/nb";
import { useAnimationSafe } from "@/hooks/useAnimationSafe";
import { fadeUp } from "@/lib/motion-utils";
import "../../styles/about.css";

export const Route = createLazyRoute("/about/contact")({
  component: ContactPage,
});

const CONTACTS = [
  {
    channel: "Enterprise",
    detail: "enterprise@vantadb.dev",
    sub: "Licensing, SLA, custom deployments",
    type: "email",
  },
  {
    channel: "Security",
    detail: "security@vantadb.dev",
    sub: "Vulnerability reports, responsible disclosure",
    type: "email",
  },
  {
    channel: "Partnerships",
    detail: "partners@vantadb.dev",
    sub: "Integrations, co-marketing, ecosystems",
    type: "email",
  },
  {
    channel: "GitHub Issues",
    detail: "github.com/ness-e/Vantadb",
    sub: "Bug reports and feature requests",
    type: "link",
  },
  {
    channel: "Discord",
    detail: "Join community",
    sub: "Real-time support and Q&A",
    type: "link",
  },
  {
    channel: "X / Twitter",
    detail: "@vantadb",
    sub: "Announcements and updates",
    type: "link",
  },
];

function ContactPage() {
  const contactsRef = useRef<HTMLElement>(null);
  const securityRef = useRef<HTMLElement>(null);

  useAnimationSafe(() => {
    const parts = contactsRef.current?.querySelectorAll<HTMLElement>(".nc-ab-part");
    if (!parts?.length) return;
    fadeUp(parts, { stagger: 0.2 });
  }, contactsRef);

  useAnimationSafe(() => {
    const parts = securityRef.current?.querySelectorAll<HTMLElement>(".nc-ab-part");
    if (!parts?.length) return;
    fadeUp(parts, { stagger: 0.2 });
  }, securityRef);

  return (
    <div className="nb-page">
      <NbSubpageHero
        pattern="p16"
        title={
          <span>
            Get in touch.
            <br />
            We read everything.
          </span>
        }
        sub="Whether you're evaluating VantaDB for your enterprise, interested in a partnership, or just want to say hello — we'd love to hear from you."
      />

      <NbSection ref={contactsRef} ariaLabel="Contact channels">
        <NbSectionHeader
          monoLabel="[CONTACT]"
          headline="Contact channels."
          sub="The right channel for every conversation."
        />

        <div className="nc-ab-part">
          <div className="nc-ab-contacts">
            {CONTACTS.map((c) => (
              <div key={c.channel} className="nc-ab-contact-card">
                <span className="nc-ab-contact-type">{c.type === "email" ? "EMAIL" : "LINK"}</span>
                <h3 className="nc-ab-contact-title">{c.channel}</h3>
                <span className="nc-ab-contact-detail">{c.detail}</span>
                <p className="nc-ab-contact-sub">{c.sub}</p>
              </div>
            ))}
          </div>
        </div>
      </NbSection>

      <NbSection ref={securityRef} variant="dark" ariaLabel="Security">
        <NbSectionHeader
          monoLabel="[SECURITY]"
          headline="Responsible disclosure."
          sub="How we handle security vulnerabilities."
        />

        <div className="nc-ab-part">
          <div className="nb-split-7-5">
            <div>
              <p>
                Found a security vulnerability? Email{" "}
                <span className="nb-mono-label">security@vantadb.dev</span>. We practice responsible
                disclosure and will work with you to validate, fix, and release a patch before
                public disclosure. We don't have a formal bug bounty program yet, but we'll credit
                you in the release notes.
              </p>
              <p>
                Response time: &lt;48h for critical, &lt;72h for high severity. We follow a 90-day
                disclosure timeline from first contact.
              </p>
            </div>
          </div>
        </div>
      </NbSection>

      <NbSection ariaLabel="Get started">
        <NbBlockAmber>
          <div className="nb-text-center">
            <span className="nb-mono-label">GET STARTED</span>
            <p className="nb-section-sub">VantaDB is free and open source. Start building today.</p>
            <Link to="/docs" className="nb-btn nb-btn--ghost">
              DOCS
            </Link>
          </div>
        </NbBlockAmber>
      </NbSection>
    </div>
  );
}

export function PendingComponent() {
  return (
    <div className="nb-pending">
      <span>Loading...</span>
    </div>
  );
}
