import { Link } from "@tanstack/react-router";
import { memo } from "react";
import VantaDBLogo from "./VantaDBLogo";

export const SwissFooter = memo(function SwissFooter() {
  return (
    <footer className="swiss-footer">
      <div className="footer-grid">
        <div className="footer-col">
          <VantaDBLogo variant="full" size="sm" />
          <span className="footer-tagline">Embedded Cognitive Memory.</span>
        </div>

        <div className="footer-col">
          <span className="footer-col-label">PRODUCT</span>
          <Link to="/engine" className="footer-link">Core Engine</Link>
          <Link to="/architecture" className="footer-link">Architecture</Link>
          <Link to="/integrations" className="footer-link">Integrations</Link>
          <Link to="/solutions/ai-agents" className="footer-link">AI Agents</Link>
          <Link to="/solutions/local-rag" className="footer-link">Local RAG</Link>
          <Link to="/pricing" className="footer-link">Pricing</Link>
        </div>

        <div className="footer-col">
          <span className="footer-col-label">RESOURCES</span>
          <Link to="/docs" className="footer-link">Documentation</Link>
          <Link to="/use-cases" className="footer-link">Use Cases</Link>
          <Link to="/solutions/security" className="footer-link">Security</Link>
          <Link to="/changelog" className="footer-link">Changelog</Link>
          <Link to="/product/benchmarks" className="footer-link">Benchmarks</Link>
          <Link to="/blog" className="footer-link">Blog</Link>
        </div>

        <div className="footer-col">
          <span className="footer-col-label">COMPANY</span>
          <Link to="/about" className="footer-link">About</Link>
          <Link to="/about/community" className="footer-link">Community</Link>
          <Link to="/about/contact" className="footer-link">Contact</Link>
        </div>
      </div>

      <div className="footer-bottom">
        <span className="footer-copyright">© {new Date().getFullYear()} VantaDB.</span>
        <div className="footer-social">
          <a href="https://github.com/ness-e/Vantadb" target="_blank" rel="noreferrer" className="footer-link footer-link--sm">GitHub</a>
          <a href="https://twitter.com/vantadb" target="_blank" rel="noreferrer" className="footer-link footer-link--sm">Twitter</a>
          <a href="https://discord.gg/vantadb" target="_blank" rel="noreferrer" className="footer-link footer-link--sm">Discord</a>
        </div>
      </div>
    </footer>
  );
});
