import { createFileRoute } from "@tanstack/react-router";
import { SwissSubpageHero } from "@/components/SwissSubpageHero";

export const Route = createFileRoute("/about/contact")({
  head: () => ({
    meta: [
      { title: "VantaDB — Contact" },
      {
        name: "description",
        content:
          "Get in touch with the VantaDB team. Enterprise inquiries, partnerships, security reports, or general questions.",
      },
    ],
  }),
  component: ContactPage,
});

// ── Data ─────────────────────────────────────────────────────────────────────
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
    <div className="engine-page">
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

      <main className="engine-main">
        {/* Contact channels */}
        <section className="engine-section engine-section--bordered">
          <span className="swiss-eyebrow">01 / 02 — Contact Channels</span>
          <div
            style={{
              display: "grid",
              gridTemplateColumns: "repeat(3, 1fr)",
              gap: "1px",
              background: "var(--border)",
              border: "1px solid var(--border)",
              marginTop: "3rem",
            }}
          >
            {CONTACTS.map((c) => (
              <div
                key={c.channel}
                style={{
                  background: "var(--background)",
                  padding: "2.5rem",
                  display: "flex",
                  flexDirection: "column",
                  gap: "0.5rem",
                  transition: "background-color 150ms var(--ease-cut)",
                }}
                onMouseEnter={(e) =>
                  ((e.currentTarget as HTMLDivElement).style.background = "var(--surface-raised)")
                }
                onMouseLeave={(e) =>
                  ((e.currentTarget as HTMLDivElement).style.background = "var(--background)")
                }
              >
                <span
                  style={{
                    fontFamily: "var(--font-mono)",
                    fontSize: "0.55rem",
                    color: c.type === "email" ? "var(--amber)" : "var(--steel)",
                    textTransform: "uppercase",
                    letterSpacing: "0.08em",
                  }}
                >
                  {c.type === "email" ? "EMAIL" : "LINK"}
                </span>
                <h3
                  style={{
                    fontFamily: "var(--font-display)",
                    fontSize: "1.1rem",
                    fontWeight: 700,
                    letterSpacing: "-0.03em",
                    color: "var(--foreground)",
                    margin: 0,
                  }}
                >
                  {c.channel}
                </h3>
                <span
                  style={{
                    fontFamily: "var(--font-mono)",
                    fontSize: "0.72rem",
                    color: "var(--amber)",
                    letterSpacing: "0.02em",
                  }}
                >
                  {c.detail}
                </span>
                <p
                  style={{
                    fontFamily: "var(--font-sans)",
                    fontSize: "0.75rem",
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
        </section>

        {/* Security disclosure */}
        <section className="engine-section">
          <span className="swiss-eyebrow">02 / 02 — Security</span>
          <div className="swiss-grid-12" style={{ alignItems: "start", marginTop: "3rem" }}>
            <div className="col-span-4">
              <h2
                style={{
                  fontFamily: "var(--font-display)",
                  fontSize: "1.5rem",
                  fontWeight: 800,
                  letterSpacing: "-0.04em",
                  color: "var(--foreground)",
                  lineHeight: 1.1,
                }}
              >
                Responsible disclosure
              </h2>
            </div>
            <div className="col-span-8">
              <p
                style={{
                  fontFamily: "var(--font-sans)",
                  fontSize: "0.85rem",
                  color: "var(--muted)",
                  lineHeight: 1.75,
                  margin: 0,
                }}
              >
                Found a security vulnerability? Email{" "}
                <span style={{ color: "var(--amber)", fontFamily: "var(--font-mono)" }}>
                  security@vantadb.dev
                </span>
                . We practice responsible disclosure and will work with you to validate, fix, and
                release a patch before public disclosure. We don't have a formal bug bounty program
                yet, but we'll credit you in the release notes.
              </p>
              <p
                style={{
                  fontFamily: "var(--font-sans)",
                  fontSize: "0.85rem",
                  color: "var(--muted)",
                  lineHeight: 1.75,
                  margin: "1rem 0 0",
                }}
              >
                Response time: &lt;48h for critical, &lt;72h for high severity. We follow a 90-day
                disclosure timeline from first contact.
              </p>
            </div>
          </div>
        </section>
      </main>
    </div>
  );
}
