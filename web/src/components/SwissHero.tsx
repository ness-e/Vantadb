import { useRef, useState, useCallback, useEffect } from "react";
import { Link } from "@tanstack/react-router";
import { gsap, useGSAP } from "../lib/gsap";
import * as THREE from "three";

/* ─────────────────────────────────────────────────────────────────────────────
 * Three.js Scene — Wireframe Torus + Icosahedron
 * ──────────────────────────────────────────────────────────────────────────── */

function useHeroScene(canvasRef: React.RefObject<HTMLCanvasElement | null>) {
  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;

    const prefersReduced = window.matchMedia("(prefers-reduced-motion: reduce)").matches;

    const renderer = new THREE.WebGLRenderer({
      canvas,
      antialias: true,
      alpha: true,
    });
    renderer.setPixelRatio(Math.min(window.devicePixelRatio, 2));
    renderer.setClearColor(0x000000, 0);

    const scene = new THREE.Scene();
    const camera = new THREE.PerspectiveCamera(45, 1, 0.1, 100);
    camera.position.set(0, 0, 6);

    const torusGeo = new THREE.TorusGeometry(1.8, 0.15, 28, 80);
    const torusMat = new THREE.MeshBasicMaterial({
      color: 0x0a0a0a,
      wireframe: true,
      transparent: true,
      opacity: 0.7,
    });
    const torus = new THREE.Mesh(torusGeo, torusMat);
    torus.rotation.x = Math.PI * 0.35;
    torus.rotation.z = Math.PI * 0.08;
    torus.position.x = 1.5;
    scene.add(torus);

    const sphereGeo = new THREE.IcosahedronGeometry(0.7, 1);
    const sphereMat = new THREE.MeshBasicMaterial({
      color: 0xff5500,
      wireframe: true,
      transparent: true,
      opacity: 0.85,
    });
    const sphere = new THREE.Mesh(sphereGeo, sphereMat);
    sphere.position.x = 1.5;
    scene.add(sphere);

    const handleResize = () => {
      const rect = canvas.parentElement?.getBoundingClientRect();
      if (!rect) return;
      const w = rect.width;
      const h = rect.height;
      renderer.setSize(w, h);
      camera.aspect = w / h;
      camera.updateProjectionMatrix();
    };
    handleResize();
    window.addEventListener("resize", handleResize);

    let frameId: number;
    const baseSpeed = prefersReduced ? 0 : 0.001;

    const animate = () => {
      frameId = requestAnimationFrame(animate);
      torus.rotation.y += baseSpeed * 0.8;
      sphere.rotation.y -= baseSpeed * 1.2;
      sphere.rotation.x += baseSpeed * 0.5;
      renderer.render(scene, camera);
    };
    animate();

    return () => {
      cancelAnimationFrame(frameId);
      window.removeEventListener("resize", handleResize);
      scene.traverse((child) => {
        if (child instanceof THREE.Mesh) {
          child.geometry?.dispose();
          if (Array.isArray(child.material)) {
            child.material.forEach((m) => m.dispose());
          } else {
            child.material?.dispose();
          }
        }
      });
      renderer.dispose();
      renderer.forceContextLoss?.();
    };
  }, [canvasRef]);
}

/* ─────────────────────────────────────────────────────────────────────────────
 * SwissHero Component
 * ──────────────────────────────────────────────────────────────────────────── */

export function SwissHero() {
  const containerRef = useRef<HTMLElement>(null);
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const [copied, setCopied] = useState(false);

  // Three.js scene
  useHeroScene(canvasRef);

  // GSAP entry animations
  useGSAP(
    () => {
      const mm = gsap.matchMedia();

      mm.add("(prefers-reduced-motion: no-preference)", () => {
        // 1. Labels flash orange → white
        gsap.fromTo(
          ".swiss-hero-label span",
          { opacity: 0,           color: "var(--orange)" },
          {
            opacity: 1,
            color: "var(--foreground)",
            duration: 0.25,
            stagger: 0.08,
            ease: "cubic-bezier(0.25, 1, 0.5, 1)",
            delay: 0.3,
          },
        );

        // 2. Title reveal
        gsap.fromTo(
          ".swiss-hero-title-line", 
          { opacity: 0, y: 30 },
          { opacity: 1, y: 0, duration: 0.5, ease: "cubic-bezier(0.25, 1, 0.5, 1)", delay: 0.5 }
        );

        // 3. Description
        gsap.fromTo(
          ".swiss-hero-description",
          { opacity: 0, y: 15 },
          { opacity: 1, y: 0, duration: 0.25, ease: "cubic-bezier(0.25, 1, 0.5, 1)", delay: 1.1 },
        );

        // 4. CTAs
        gsap.fromTo(
          ".swiss-hero-actions",
          { opacity: 0, y: 15 },
          { opacity: 1, y: 0, duration: 0.25, ease: "cubic-bezier(0.25, 1, 0.5, 1)", delay: 1.3 },
        );

      });
    },
    { scope: containerRef },
  );

  const handleCopy = useCallback(async () => {
    try {
      await navigator.clipboard.writeText("pip install vantadb-py");
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    } catch (err) {
      console.error("Failed to copy command: ", err);
    }
  }, []);

  return (
    <section className="swiss-hero-section" ref={containerRef}>
      {/* Three.js Canvas */}
      <canvas ref={canvasRef} className="swiss-hero-canvas" aria-hidden="true" />

      {/* Content Overlay */}
      <div className="swiss-hero-overlay">
        <div className="swiss-hero-label">
          <span>[RUST-NATIVE]</span>
          <span>[IN-PROCESS]</span>
          <span>[ZERO-SERVERS]</span>
        </div>

        <h1 className="swiss-hero-title">
          <span className="swiss-hero-title-wrapper">
            <span className="swiss-hero-title-line">VantaDB — Embedded Vector Database for AI Agents</span>
          </span>
        </h1>

        <p className="swiss-hero-description">
          One pip install. Vector search (HNSW), BM25 full-text, and hybrid search (RRF) in a single Rust binary. Zero
          servers. Zero ops. Sub-millisecond.
        </p>

        <div className="swiss-hero-actions">
          <button
            onClick={handleCopy}
            className="btn-primary btn-primary--hero"
            aria-label="Copy pip install command"
          >
            <span>{copied ? "Copied!" : "pip install vantadb-py"}</span>
            <svg
              xmlns="http://www.w3.org/2000/svg"
              width="14"
              height="14"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              strokeWidth="2.5"
              strokeLinecap="round"
              strokeLinejoin="round"
              aria-hidden="true"
            >
              {copied ? (
                <polyline points="20 6 9 17 4 12" />
              ) : (
                <>
                  <rect x="9" y="9" width="13" height="13" rx="2" ry="2" />
                  <path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1" />
                </>
              )}
            </svg>
          </button>
          <Link to="/docs" className="btn-ghost btn-ghost--hero">
            Read Docs
          </Link>
        </div>
      </div>
    </section>
  );
}
