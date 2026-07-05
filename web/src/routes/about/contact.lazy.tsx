import { createLazyRoute, Link } from "@tanstack/react-router";
import { SwissSubpageHero } from "@/components/SwissSubpageHero";

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
      <SwissSubpageHero
        num="06"
        eyebrow="About — Contact"
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
          <div className="nb-telemetry" style={{ marginBottom: "var(--space-md)" }}>
            <span>Home</span>
            <span>Contact</span>
          </div>

          <span className="nb-label nb-label--amber">01 / 02 — Contact Channels</span>
          <div className="nb-divider" />

          <div className="nb-grid nb-grid--cols-3" style={{ marginTop: "var(--space-xl)" }}>
            {CONTACTS.map((c) => (
              <div key={c.channel} className="nb-cell" style={{ padding: "var(--space-xl)" }}>
                <span
                  className="nb-label"
                  style={{
                    color: c.type === "email" ? "var(--amber)" : "var(--steel)",
                    marginBottom: "var(--space-2xs)",
                  }}
                >
                  {c.type === "email" ? "EMAIL" : "LINK"}
                </span>
                <h3
                  style={{
                    fontFamily: "var(--font-display)",
                    fontSize: "var(--text-title)",
                    fontWeight: 700,
                    letterSpacing: "var(--tracking-display)",
                    color: "var(--foreground)",
                    margin: "0 0 var(--space-3xs)",
                  }}
                >
                  {c.channel}
                </h3>
                <span
                  className="nb-label nb-label--amber"
                  style={{ marginBottom: "var(--space-2xs)" }}
                >
                  {c.detail}
                </span>
                <p
                  style={{
                    fontFamily: "var(--font-sans)",
                    fontSize: "var(--text-code)",
                    color: "var(--muted)",
                    lineHeight: 1.5,
                    margin: 0,
                  }}
                >
                  {c.sub}
                </p>
              </div>
            ))}
          </div>
        </div>
      </section>

      <section className="nb-section nb-bg-cross--faint">
        <div className="nb-inner">
          <span className="nb-label nb-label--amber">02 / 02 — Security</span>
          <div className="nb-divider" />

          <div className="nb-split-7-5" style={{ marginTop: "var(--space-xl)" }}>
            <h2
              style={{
                fontFamily: "var(--font-display)",
                fontSize: "var(--text-display)",
                fontWeight: 800,
                letterSpacing: "var(--tracking-display)",
                color: "var(--foreground)",
                lineHeight: 1.1,
                margin: 0,
              }}
            >
              Responsible disclosure
            </h2>
            <div>
              <p
                style={{
                  fontFamily: "var(--font-sans)",
                  fontSize: "var(--text-body)",
                  color: "var(--muted)",
                  lineHeight: 1.75,
                  margin: 0,
                }}
              >
                Found a security vulnerability? Email{" "}
                <span
                  className="nb-label nb-label--amber"
                  style={{ display: "inline", marginBottom: 0 }}
                >
                  security@vantadb.dev
                </span>
                . We practice responsible disclosure and will work with you to validate, fix, and
                release a patch before public disclosure. We don't have a formal bug bounty program
                yet, but we'll credit you in the release notes.
              </p>
              <p
                style={{
                  fontFamily: "var(--font-sans)",
                  fontSize: "var(--text-body)",
                  color: "var(--muted)",
                  lineHeight: 1.75,
                  margin: "var(--space-sm) 0 0",
                }}
              >
                Response time: &lt;48h for critical, &lt;72h for high severity. We follow a 90-day
                disclosure timeline from first contact.
              </p>
            </div>
          </div>
        </div>
      </section>

      <section className="nb-section">
        <div className="nb-inner">
          <div className="nb-block-amber" style={{ textAlign: "center" }}>
            <span className="nb-label" style={{ color: "var(--text-on-amber)" }}>
              GET STARTED
            </span>
            <p
              style={{
                fontFamily: "var(--font-sans)",
                fontSize: "var(--text-body)",
                color: "var(--text-on-amber)",
                margin: "var(--space-2xs) 0",
                opacity: 0.85,
              }}
            >
              VantaDB is free and open source. Start building today.
            </p>
            <Link
              to="/docs"
              className="btn-ghost"
              style={{
                borderColor: "var(--text-on-amber)",
                color: "var(--text-on-amber)",
                boxShadow: "var(--shadow-brutal)",
              }}
            >
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
    <div
      style={{
        display: "flex",
        justifyContent: "center",
        alignItems: "center",
        height: "60vh",
        color: "var(--muted)",
      }}
    >
      <span className="nb-label" style={{ fontSize: "var(--text-label)", marginBottom: 0 }}>
        Loading...
      </span>
    </div>
  );
}
