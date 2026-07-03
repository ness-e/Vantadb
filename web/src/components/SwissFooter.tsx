import { Link } from "@tanstack/react-router";
import { memo } from "react";

export const SwissFooter = memo(function SwissFooter() {
  return (
    <footer style={{ 
      background: "var(--block-dark-bg, #0a0a0a)", 
      borderTop: "1px solid rgba(255,255,255,0.08)",
      padding: "80px 24px 40px 24px",
      color: "var(--block-dark-text, #f0f0f0)"
    }}>
      <div style={{ maxWidth: "1200px", margin: "0 auto" }}>
        
        <div style={{ 
          display: "grid", 
          gridTemplateColumns: "repeat(auto-fit, minmax(200px, 1fr))", 
          gap: "48px",
          marginBottom: "80px"
        }}>
          
          {/* Logo Column */}
          <div style={{ display: "flex", flexDirection: "column", gap: "16px" }}>
            <span style={{ fontFamily: "var(--font-display)", fontSize: "1.5rem", fontWeight: 700, letterSpacing: "-0.04em", color: "#ffffff" }}>
              VantaDB
            </span>
            <span style={{ fontFamily: "var(--font-mono)", fontSize: "0.75rem", color: "var(--block-dark-muted, #808080)", letterSpacing: "0.1em" }}>
              Embedded Cognitive Memory.
            </span>
          </div>

          {/* Links Columns */}
          <div style={{ display: "flex", flexDirection: "column", gap: "16px" }}>
            <span style={{ fontFamily: "var(--font-mono)", fontSize: "0.75rem", color: "#ffffff", letterSpacing: "0.1em" }}>PRODUCT</span>
            <Link to="/engine" className="footer-link">Core Engine</Link>
            <Link to="/architecture" className="footer-link">Architecture</Link>
            <Link to="/integrations" className="footer-link">Integrations</Link>
            <Link to="/pricing" className="footer-link">Pricing</Link>
          </div>

          <div style={{ display: "flex", flexDirection: "column", gap: "16px" }}>
            <span style={{ fontFamily: "var(--font-mono)", fontSize: "0.75rem", color: "#ffffff", letterSpacing: "0.1em" }}>RESOURCES</span>
            <Link to="/docs" className="footer-link">Documentation</Link>
            <Link to="/use-cases" className="footer-link">Use Cases</Link>
            <Link to="/blog" className="footer-link">Blog</Link>
          </div>

          <div style={{ display: "flex", flexDirection: "column", gap: "16px" }}>
            <span style={{ fontFamily: "var(--font-mono)", fontSize: "0.75rem", color: "#ffffff", letterSpacing: "0.1em" }}>COMPANY</span>
            <Link to="/about" className="footer-link">About</Link>
            <Link to="/about/community" className="footer-link">Community</Link>
            <Link to="/about/contact" className="footer-link">Contact</Link>
          </div>

        </div>

        {/* Bottom bar */}
        <div style={{ 
          borderTop: "1px solid rgba(255,255,255,0.08)", 
          paddingTop: "32px",
          display: "flex",
          justifyContent: "space-between",
          alignItems: "center",
          flexWrap: "wrap",
          gap: "16px"
        }}>
          <span style={{ fontFamily: "var(--font-mono)", fontSize: "0.75rem", color: "var(--block-dark-muted, #808080)", letterSpacing: "0.1em", textTransform: "uppercase" }}>
            © {new Date().getFullYear()} VantaDB. All rights reserved.
          </span>
          
          <div style={{ display: "flex", gap: "24px" }}>
            <a href="https://github.com/ness-e/Vantadb" target="_blank" rel="noreferrer" className="footer-link" style={{ fontSize: "0.75rem", textTransform: "uppercase" }}>GitHub</a>
            <a href="https://twitter.com/vantadb" target="_blank" rel="noreferrer" className="footer-link" style={{ fontSize: "0.75rem", textTransform: "uppercase" }}>Twitter</a>
            <a href="https://discord.gg/vantadb" target="_blank" rel="noreferrer" className="footer-link" style={{ fontSize: "0.75rem", textTransform: "uppercase" }}>Discord</a>
          </div>
        </div>

      </div>

      <style>{`
        .footer-link {
          font-family: var(--font-sans);
          font-size: 0.95rem;
          color: var(--block-dark-muted, #808080);
          text-decoration: none;
          transition: color 150ms;
        }
        .footer-link:hover {
          color: #ffffff;
        }
      `}</style>
    </footer>
  );
});
