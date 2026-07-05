import { memo } from "react";
import "../styles/trust-bar.css";

const INTEGRATIONS = ["RUST", "PYTHON", "DOCKER", "GITHUB", "VSCODE"];

const TICKER_ITEMS = [
  "INSERT 10k vectors... DONE (1.2ms)",
  "QUERY 1k results... DONE (0.8ms)",
  "HYBRID SEARCH 0.998 R@10",
  "BUILD TIME 47ms",
];

const BOX_STYLE = {
  display: "flex",
  alignItems: "center",
  justifyContent: "center",
  width: "6ch",
  height: "2.4em",
  border: "1px solid var(--border-visible)",
  fontFamily: "var(--font-mono)",
  fontSize: "var(--text-micro)",
  color: "var(--steel)",
  letterSpacing: "0.14em",
  textTransform: "uppercase",
  fontWeight: 600,
  flexShrink: 0,
} as const;

const ROW_STYLE = {
  display: "flex",
  justifyContent: "center",
  gap: "var(--space-sm)",
  flexWrap: "wrap",
} as const;

export const NbTrustBar = memo(function NbTrustBar() {
  return (
    <section className="nb-section nb-section--sm" aria-label="Trust indicators">
      <div className="nb-inner">
        <div
          style={{
            display: "flex",
            alignItems: "center",
            justifyContent: "center",
            gap: "var(--space-lg)",
            paddingBottom: "var(--space-md)",
            flexWrap: "wrap",
          }}
        >
          <span
            style={{
              fontFamily: "var(--font-mono)",
              fontSize: "var(--text-metric)",
              fontWeight: 700,
              color: "var(--amber)",
              lineHeight: 1,
              letterSpacing: "-0.04em",
            }}
          >
            2,847 ★
          </span>
          <span
            style={{
              fontFamily: "var(--font-mono)",
              fontSize: "var(--text-code)",
              fontWeight: 600,
              letterSpacing: "0.14em",
              color: "var(--steel)",
              textTransform: "uppercase",
            }}
          >
            USED BY
          </span>
        </div>

        <div
          style={{
            ...ROW_STYLE,
            paddingBottom: "var(--space-lg)",
          }}
        >
          {INTEGRATIONS.map((name) => (
            <div key={name} style={BOX_STYLE} aria-hidden="true">
              {name}
            </div>
          ))}
        </div>

        <div
          className="nb-trust-marquee"
          style={{
            overflow: "hidden",
            width: "100%",
            position: "relative",
            borderTop: "1px solid var(--border)",
            paddingTop: "var(--space-md)",
          }}
        >
          <div className="nb-trust-marquee-track">
            <span>{TICKER_ITEMS.join(" /// ")}</span>
            <span>{TICKER_ITEMS.join(" /// ")}</span>
          </div>
        </div>
      </div>
    </section>
  );
});
