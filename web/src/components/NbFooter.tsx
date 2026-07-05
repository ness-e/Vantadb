import { Link } from "@tanstack/react-router";
import { memo } from "react";
import VantaDBLogo from "./VantaDBLogo";

export const NbFooter = memo(function NbFooter() {
  return (
    <footer className="nb-section nb-section--dark" data-frame-label="FOOTER">
      <div className="nb-inner">
        <div className="footer-grid nb-frame">
          <div className="footer-col footer-col--brand">
            <VantaDBLogo variant="full" size="sm" />
            <span className="footer-tagline">Embedded Cognitive Memory.</span>
            <div className="footer-meta">
              <span className="nb-pill">v0.1.0</span>
              <span className="nb-pill">MIT</span>
            </div>
          </div>

          <div className="footer-col">
            <span className="nb-label">Product</span>
            <Link to="/engine" className="footer-link">
              &gt; Core Engine
            </Link>
            <Link to="/architecture" className="footer-link">
              &gt; Architecture
            </Link>
            <Link to="/integrations" className="footer-link">
              &gt; Integrations
            </Link>
            <Link to="/solutions/ai-agents" className="footer-link">
              &gt; AI Agents
            </Link>
            <Link to="/solutions/local-rag" className="footer-link">
              &gt; Local RAG
            </Link>
            <Link to="/pricing" className="footer-link">
              &gt; Pricing
            </Link>
          </div>

          <div className="footer-col">
            <span className="nb-label">Resources</span>
            <Link to="/docs" className="footer-link">
              &gt; Documentation
            </Link>
            <Link to="/use-cases" className="footer-link">
              &gt; Use Cases
            </Link>
            <Link to="/solutions/security" className="footer-link">
              &gt; Security
            </Link>
            <Link to="/changelog" className="footer-link">
              &gt; Changelog
            </Link>
            <Link to="/product/benchmarks" className="footer-link">
              &gt; Benchmarks
            </Link>
            <Link to="/blog" className="footer-link">
              &gt; Blog
            </Link>
          </div>

          <div className="footer-col">
            <span className="nb-label">Company</span>
            <Link to="/about" className="footer-link">
              &gt; About
            </Link>
            <Link to="/about/community" className="footer-link">
              &gt; Community
            </Link>
            <Link to="/about/contact" className="footer-link">
              &gt; Contact
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
              &gt; GitHub
            </a>
            <a
              href="https://twitter.com/vantadb"
              target="_blank"
              rel="noreferrer"
              className="footer-link footer-link--sm"
            >
              &gt; Twitter
            </a>
            <a
              href="https://discord.gg/vantadb"
              target="_blank"
              rel="noreferrer"
              className="footer-link footer-link--sm"
            >
              &gt; Discord
            </a>
          </div>
        </div>
      </div>
    </footer>
  );
});
