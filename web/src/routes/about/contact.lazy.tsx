import { createLazyRoute, Link } from "@tanstack/react-router";
import { NbSubpageHero } from "@/components/NbSubpageHero";

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
  return (
    <div className="nb-page">
      <NbSubpageHero
        num="06"
        title={
          <span>
            Get in touch.
            <br />
            We read everything.
          </span>
        }
        sub="Whether you're evaluating VantaDB for your enterprise, interested in a partnership, or just want to say hello — we'd love to hear from you."
      />

      <section className="nb-section">
        <div className="nb-inner">
          <h2 className="about-contact-section-title">Contact Channels</h2>
          <div className="nb-divider" />

          <div className="nb-grid nb-grid--cols-3 about-contact-grid-top">
            {CONTACTS.map((c) => (
              <div key={c.channel} className="nb-cell about-contact-channel-card">
                <span
                  style={{
                    fontFamily: "var(--font-mono)",
                    fontSize: "var(--text-micro)",
                    color: c.type === "email" ? "var(--amber)" : "var(--steel)",
                    marginBottom: "var(--space-2xs)",
                    display: "block",
                  }}
                >
                  {c.type === "email" ? "EMAIL" : "LINK"}
                </span>
                <h3 className="about-contact-channel-name">{c.channel}</h3>
                <span className="about-contact-channel-detail">{c.detail}</span>
                <p className="about-contact-channel-sub">{c.sub}</p>
              </div>
            ))}
          </div>
        </div>
      </section>

      <section className="nb-section nb-bg-cross--faint">
        <div className="nb-inner">
          <h2 className="about-contact-section-title">Security</h2>
          <div className="nb-divider" />

          <div className="nb-split-7-5 about-contact-split-top">
            <h2 className="about-contact-security-title">Responsible disclosure</h2>
            <div>
              <p className="about-contact-security-desc">
                Found a security vulnerability? Email{" "}
                <span className="about-contact-security-email">security@vantadb.dev</span>. We
                practice responsible disclosure and will work with you to validate, fix, and release
                a patch before public disclosure. We don't have a formal bug bounty program yet, but
                we'll credit you in the release notes.
              </p>
              <p className="about-contact-security-desc-p2">
                Response time: &lt;48h for critical, &lt;72h for high severity. We follow a 90-day
                disclosure timeline from first contact.
              </p>
            </div>
          </div>
        </div>
      </section>

      <section className="nb-section">
        <div className="nb-inner">
          <div className="nb-block-amber about-contact-cta-block">
            <span className="about-contact-cta-label">GET STARTED</span>
            <p className="about-contact-cta-desc">
              VantaDB is free and open source. Start building today.
            </p>
            <Link to="/docs" className="nb-btn nb-btn--ghost about-contact-cta-btn">
              DOCS
            </Link>
          </div>
        </div>
      </section>
    </div>
  );
}

export function PendingComponent() {
  return (
    <div className="about-pending-container">
      <span className="about-pending-text">Loading...</span>
    </div>
  );
}
