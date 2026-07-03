import { Link, useLocation } from "@tanstack/react-router";
import VantaDBLogo from "./VantaDBLogo";
import { useState, useEffect, useCallback } from "react";

export function Nav() {
  const [scrolled, setScrolled] = useState(false);
  const [drawerOpen, setDrawerOpen] = useState(false);
  const location = useLocation();

  useEffect(() => {
    let ticking = false;
    const handleScroll = () => {
      if (!ticking) {
        requestAnimationFrame(() => {
          setScrolled(window.scrollY > 20);
          ticking = false;
        });
        ticking = true;
      }
    };
    window.addEventListener("scroll", handleScroll, { passive: true });
    return () => window.removeEventListener("scroll", handleScroll);
  }, []);

  useEffect(() => {
    if (drawerOpen) {
      document.body.style.overflow = "hidden";
    } else {
      document.body.style.overflow = "";
    }
    return () => {
      document.body.style.overflow = "";
    };
  }, [drawerOpen]);

  const closeDrawer = useCallback(() => setDrawerOpen(false), []);

  const isActive = (path: string) =>
    path === "/" ? location.pathname === "/" : location.pathname.startsWith(path);

  const navLinks = [
    { path: "/engine", label: "Core Engine" },
    { path: "/architecture", label: "Architecture" },
    { path: "/solutions/ai-agents", label: "AI Agents" },
    { path: "/solutions/local-rag", label: "Local RAG" },
    { path: "/solutions/ai-ide-tooling", label: "IDE Tooling" },
    { path: "/use-cases", label: "Use Cases" },
    { path: "/pricing", label: "Pricing" },
  ];

  return (
    <>
      <nav className={`vanta-nav${scrolled ? " vanta-nav--scrolled" : ""}`}>
        <Link to="/" className="vanta-logo">
          <VantaDBLogo variant="full" size="sm" />
        </Link>

        <div className="nav-desktop">
          {navLinks.map((item) => (
            <Link
              key={item.path}
              to={item.path}
              className={`nav-link${isActive(item.path) ? " active" : ""}`}
            >
              {item.label}
            </Link>
          ))}
        </div>

        <div className="nav-actions">
          <Link to="/docs" className="nav-cta">
            Docs
          </Link>
          <a
            href="https://github.com/ness-e/Vantadb"
            target="_blank"
            rel="noreferrer"
            className="nav-cta"
          >
            GitHub
          </a>
        </div>

        <button
          className={`nav-hamburger${drawerOpen ? " nav-hamburger--open" : ""}`}
          onClick={() => setDrawerOpen((v) => !v)}
          aria-label={drawerOpen ? "Close menu" : "Open menu"}
          aria-expanded={drawerOpen}
        >
          <span />
          <span />
          <span />
        </button>
      </nav>

      {drawerOpen && <div className="nav-overlay" onClick={closeDrawer} aria-hidden="true" />}

      <div
        className="nav-drawer"
        aria-hidden={!drawerOpen}
        role="dialog"
        aria-modal="true"
        aria-label="Navigation menu"
      >
        <div className="nav-drawer-header">
          <VantaDBLogo variant="full" size="sm" />
          <button className="nav-drawer-close" onClick={closeDrawer} aria-label="Close menu">
            <svg
              width="16"
              height="16"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              strokeWidth="2"
              strokeLinecap="round"
            >
              <line x1="18" y1="6" x2="6" y2="18" />
              <line x1="6" y1="6" x2="18" y2="18" />
            </svg>
          </button>
        </div>

        <div className="nav-drawer-body">
          {navLinks.concat({ path: "/docs", label: "Docs" }).map((item) => (
            <Link
              key={item.path}
              to={item.path}
              className={`nav-drawer-link${isActive(item.path) ? " active" : ""}`}
              onClick={closeDrawer}
            >
              {item.label}
            </Link>
          ))}
        </div>

        <div className="nav-drawer-footer">
          <a
            href="https://github.com/ness-e/Vantadb"
            target="_blank"
            rel="noreferrer"
            className="nav-drawer-cta nav-drawer-cta--primary"
          >
            <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor" aria-hidden="true">
              <path d="M12 0C5.37 0 0 5.37 0 12c0 5.31 3.435 9.795 8.205 11.385.6.105.825-.255.825-.57 0-.285-.015-1.23-.015-2.235-3.015.555-3.795-.735-4.035-1.41-.135-.345-.72-1.41-1.23-1.695-.42-.225-1.02-.78-.015-.795.945-.015 1.62.87 1.845 1.23 1.08 1.815 2.805 1.305 3.495.99.105-.78.42-1.305.765-1.605-2.67-.3-5.46-1.335-5.46-5.925 0-1.305.465-2.385 1.23-3.225-.12-.3-.54-1.53.12-3.18 0 0 1.005-.315 3.3 1.23.96-.27 1.98-.405 3-.405s2.04.135 3 .405c2.295-1.56 3.3-1.23 3.3-1.23.66 1.65.24 2.88.12 3.18.765.84 1.23 1.905 1.23 3.225 0 4.605-2.805 5.625-5.475 5.925.435.375.81 1.095.81 2.22 0 1.605-.015 2.895-.015 3.3 0 .315.225.69.825.57A12.02 12.02 0 0 0 24 12c0-6.63-5.37-12-12-12z" />
            </svg>
            GitHub
          </a>
        </div>
      </div>
    </>
  );
}
