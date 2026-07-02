import { Link, useLocation } from "@tanstack/react-router";
import VantaDBLogo from "./VantaDBLogo";
import { useState, useEffect } from "react";

export function Nav() {
  const [scrolled, setScrolled] = useState(false);
  const location = useLocation();

  useEffect(() => {
    const handleScroll = () => {
      setScrolled(window.scrollY > 20);
    };
    window.addEventListener("scroll", handleScroll);
    return () => window.removeEventListener("scroll", handleScroll);
  }, []);

  return (
    <nav style={{
      position: "fixed",
      top: 0,
      left: 0,
      right: 0,
      height: "64px",
      background: scrolled ? "var(--surface-glass, rgba(249,248,246,0.85))" : "transparent",
      backdropFilter: scrolled ? "blur(12px)" : "none",
      borderBottom: scrolled ? "1px solid var(--border)" : "1px solid transparent",
      zIndex: 100,
      transition: "all 200ms",
      display: "flex",
      alignItems: "center",
      padding: "0 24px"
    }}>
      <div style={{ 
        width: "100%", 
        maxWidth: "1200px", 
        margin: "0 auto", 
        display: "flex", 
        justifyContent: "space-between", 
        alignItems: "center" 
      }}>
        
        {/* Logo Izquierda */}
        <Link to="/" style={{ display: "flex", alignItems: "center", gap: "12px", textDecoration: "none" }}>
          <VantaDBLogo variant="full" size="sm" noAnimation />
        </Link>

        {/* Links Centro */}
        <div style={{ display: "none", gap: "32px" }} className="nav-desktop-links">
          {[
            { path: "/engine", label: "Core Engine" },
            { path: "/architecture", label: "Architecture" },
            { path: "/use-cases", label: "Use Cases" },
            { path: "/pricing", label: "Pricing" },
          ].map((item) => (
            <Link 
              key={item.path} 
              to={item.path} 
              className="nav-link"
              style={{
                fontFamily: "var(--font-mono)",
                fontSize: "var(--text-label)",
                fontWeight: 600,
                letterSpacing: "0.14em",
                textTransform: "uppercase",
                color: location.pathname === item.path ? "var(--foreground)" : "var(--steel)",
                textDecoration: "none",
                transition: "color 100ms"
              }}
            >
              {item.label}
            </Link>
          ))}
        </div>

        {/* CTA Derecha */}
        <div style={{ display: "flex", alignItems: "center", gap: "16px" }}>
          <Link 
            to="/docs" 
            className="nav-desktop-docs"
            style={{ 
              fontFamily: "var(--font-mono)", 
              fontSize: "var(--text-label)", 
              fontWeight: 600, 
              letterSpacing: "0.14em", 
              textTransform: "uppercase", 
              color: "var(--foreground)", 
              textDecoration: "none",
              display: "none"
            }}
          >
            Docs
          </Link>
          
          <a 
            href="https://github.com/ness-e/Vantadb" 
            target="_blank" 
            rel="noreferrer" 
            className="nav-github-btn"
            style={{
              background: "var(--foreground)",
              color: "var(--background)",
              padding: "8px 16px",
              fontFamily: "var(--font-mono)",
              fontSize: "var(--text-label)",
              fontWeight: 600,
              letterSpacing: "0.14em",
              textTransform: "uppercase",
              textDecoration: "none",
              transition: "background 150ms"
            }}
          >
            GitHub
          </a>
        </div>

      </div>

      <style>{`
        .nav-link:hover { color: var(--foreground) !important; }
        .nav-github-btn:hover { background: var(--amber) !important; }
        @media (min-width: 768px) {
          .nav-desktop-links { display: flex !important; }
          .nav-desktop-docs { display: block !important; }
        }
      `}</style>
    </nav>
  );
}
