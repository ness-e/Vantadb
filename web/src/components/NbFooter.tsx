import { Link } from "@tanstack/react-router";
import { memo } from "react";
import VantaDBLogo from "./VantaDBLogo";

export const NbFooter = memo(function NbFooter() {
  return (
    <footer className="nb-section nb-section--dark">
      <div className="nb-inner">
        <div className="footer-grid">
          <div className="footer-col footer-col--brand">
            <VantaDBLogo variant="full" size="sm" />
            <span className="footer-tagline">Embedded Cognitive Memory.</span>
          </div>

          <div className="footer-col">
            <h3
              style={{
                fontFamily: "var(--font-mono)",
                fontSize: "0.65rem",
                letterSpacing: "0.12em",
                color: "var(--amber)",
                textTransform: "uppercase",
                marginBottom: "var(--space-sm)",
              }}
            >
              Product
            </h3>
            <Link to="/engine" className="footer-link">
              Core Engine
            </Link>
            <Link to="/architecture" className="footer-link">
              Architecture
            </Link>
            <Link to="/integrations" className="footer-link">
              Integrations
            </Link>
            <Link to="/solutions/ai-agents" className="footer-link">
              AI Agents
            </Link>
            <Link to="/solutions/local-rag" className="footer-link">
              Local RAG
            </Link>
            <Link to="/pricing" className="footer-link">
              Pricing
            </Link>
          </div>

          <div className="footer-col">
            <h3
              style={{
                fontFamily: "var(--font-mono)",
                fontSize: "0.65rem",
                letterSpacing: "0.12em",
                color: "var(--amber)",
                textTransform: "uppercase",
                marginBottom: "var(--space-sm)",
              }}
            >
              Resources
            </h3>
            <Link to="/docs" className="footer-link">
              Documentation
            </Link>
            <Link to="/use-cases" className="footer-link">
              Use Cases
            </Link>
            <Link to="/solutions/security" className="footer-link">
              Security
            </Link>
            <Link to="/changelog" className="footer-link">
              Changelog
            </Link>
            <Link to="/product/benchmarks" className="footer-link">
              Benchmarks
            </Link>
            <Link to="/blog" className="footer-link">
              Blog
            </Link>
          </div>

          <div className="footer-col">
            <h3
              style={{
                fontFamily: "var(--font-mono)",
                fontSize: "0.65rem",
                letterSpacing: "0.12em",
                color: "var(--amber)",
                textTransform: "uppercase",
                marginBottom: "var(--space-sm)",
              }}
            >
              Company
            </h3>
            <Link to="/about" className="footer-link">
              About
            </Link>
            <Link to="/about/community" className="footer-link">
              Community
            </Link>
            <Link to="/about/contact" className="footer-link">
              Contact
            </Link>
          </div>
        </div>

        <div className="footer-bottom">
          <span className="footer-copyright">
            &copy; {new Date().getFullYear()} VantaDB &mdash; MIT License
          </span>
          <div className="footer-social">
            <a
              href="https://github.com/ness-e/Vantadb"
              target="_blank"
              rel="noreferrer"
              className="footer-link footer-link--sm"
            >
              GitHub
            </a>
            <a
              href="https://twitter.com/vantadb"
              target="_blank"
              rel="noreferrer"
              className="footer-link footer-link--sm"
            >
              Twitter
            </a>
            <a
              href="https://discord.gg/vantadb"
              target="_blank"
              rel="noreferrer"
              className="footer-link footer-link--sm"
            >
              Discord
            </a>
          </div>
        </div>
      </div>
    </footer>
  );
});
