import { useRef, useState, useCallback, useEffect } from "react";
import { Link } from "@tanstack/react-router";
import { gsap, useGSAP } from "../lib/gsap";
import * as THREE from "three";

/* ─────────────────────────────────────────────────────────────────────────────
 * Three.js Scene — Wireframe Torus + Network Nodes
 * ──────────────────────────────────────────────────────────────────────────── */

function useHeroScene(canvasRef: React.RefObject<HTMLCanvasElement | null>) {
  const mouseRef = useRef({ x: 0, y: 0 });

  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;

    // Check reduced motion
    const prefersReduced = window.matchMedia(
      "(prefers-reduced-motion: reduce)"
    ).matches;

    // ── Renderer ──
    const renderer = new THREE.WebGLRenderer({
      canvas,
      antialias: true,
      alpha: true,
    });
    renderer.setPixelRatio(Math.min(window.devicePixelRatio, 2));
    renderer.setClearColor(0x000000, 0);

    // ── Scene & Camera ──
    const scene = new THREE.Scene();
    const camera = new THREE.PerspectiveCamera(45, 1, 0.1, 100);
    camera.position.set(0, 0, 6);

    // ── Torus Wireframe ──
    const torusGeo = new THREE.TorusGeometry(1.6, 0.6, 24, 48);
    const torusMat = new THREE.MeshBasicMaterial({
      color: 0xff5500,
      wireframe: true,
      transparent: true,
      opacity: 0.55,
    });
    const torus = new THREE.Mesh(torusGeo, torusMat);
    torus.rotation.x = Math.PI * 0.3;
    torus.rotation.z = Math.PI * 0.1;
    scene.add(torus);

    // ── Inner sphere wireframe ──
    const sphereGeo = new THREE.IcosahedronGeometry(0.8, 2);
    const sphereMat = new THREE.MeshBasicMaterial({
      color: 0x000000,
      wireframe: true,
      transparent: true,
      opacity: 0.08,
    });
    const sphere = new THREE.Mesh(sphereGeo, sphereMat);
    scene.add(sphere);

    // ── Network nodes (floating points) ──
    const nodeCount = 60;
    const nodePositions = new Float32Array(nodeCount * 3);
    for (let i = 0; i < nodeCount; i++) {
      const theta = Math.random() * Math.PI * 2;
      const phi = Math.acos(2 * Math.random() - 1);
      const r = 2.2 + Math.random() * 1.2;
      nodePositions[i * 3] = r * Math.sin(phi) * Math.cos(theta);
      nodePositions[i * 3 + 1] = r * Math.sin(phi) * Math.sin(theta);
      nodePositions[i * 3 + 2] = r * Math.cos(phi);
    }
    const nodeGeo = new THREE.BufferGeometry();
    nodeGeo.setAttribute("position", new THREE.BufferAttribute(nodePositions, 3));
    const nodeMat = new THREE.PointsMaterial({
      color: 0xff5500,
      size: 0.05,
      transparent: true,
      opacity: 0.85,
      sizeAttenuation: true,
    });
    const nodes = new THREE.Points(nodeGeo, nodeMat);
    scene.add(nodes);

    // ── Connection lines between nearby nodes ──
    const linePositions: number[] = [];
    for (let i = 0; i < nodeCount; i++) {
      for (let j = i + 1; j < nodeCount; j++) {
        const dx = nodePositions[i * 3] - nodePositions[j * 3];
        const dy = nodePositions[i * 3 + 1] - nodePositions[j * 3 + 1];
        const dz = nodePositions[i * 3 + 2] - nodePositions[j * 3 + 2];
        const dist = Math.sqrt(dx * dx + dy * dy + dz * dz);
        if (dist < 1.2) {
          linePositions.push(
            nodePositions[i * 3], nodePositions[i * 3 + 1], nodePositions[i * 3 + 2],
            nodePositions[j * 3], nodePositions[j * 3 + 1], nodePositions[j * 3 + 2]
          );
        }
      }
    }
    const lineGeo = new THREE.BufferGeometry();
    lineGeo.setAttribute(
      "position",
      new THREE.Float32BufferAttribute(linePositions, 3)
    );
    const lineMat = new THREE.LineBasicMaterial({
      color: 0xff5500,
      transparent: true,
      opacity: 0.12,
    });
    const lines = new THREE.LineSegments(lineGeo, lineMat);
    scene.add(lines);

    // ── Group for mouse interaction ──
    const group = new THREE.Group();
    group.add(torus, sphere, nodes, lines);
    scene.add(group);

    // Position to the right side of the viewport
    group.position.x = 1.5;

    // ── Resize ──
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

    // ── Mouse tracking ──
    const handleMouse = (e: MouseEvent) => {
      mouseRef.current.x = (e.clientX / window.innerWidth) * 2 - 1;
      mouseRef.current.y = -(e.clientY / window.innerHeight) * 2 + 1;
    };
    window.addEventListener("mousemove", handleMouse);

    // ── Animation loop ──
    let frameId: number;
    const baseSpeed = prefersReduced ? 0 : 0.001;

    const animate = () => {
      frameId = requestAnimationFrame(animate);

      // Slow auto-rotation
      torus.rotation.y += baseSpeed * 0.8;
      sphere.rotation.y -= baseSpeed * 1.2;
      sphere.rotation.x += baseSpeed * 0.5;
      nodes.rotation.y += baseSpeed * 0.3;
      lines.rotation.y += baseSpeed * 0.3;

      // Mouse-driven rotation (15° snap as per spec, but smoothed slightly)
      const targetRx = mouseRef.current.y * 0.3;
      const targetRy = mouseRef.current.x * 0.4;
      group.rotation.x += (targetRx - group.rotation.x) * 0.03;
      group.rotation.y += (targetRy - group.rotation.y) * 0.03;

      renderer.render(scene, camera);
    };
    animate();

    return () => {
      cancelAnimationFrame(frameId);
      window.removeEventListener("resize", handleResize);
      window.removeEventListener("mousemove", handleMouse);
      renderer.dispose();
      torusGeo.dispose();
      torusMat.dispose();
      sphereGeo.dispose();
      sphereMat.dispose();
      nodeGeo.dispose();
      nodeMat.dispose();
      lineGeo.dispose();
      lineMat.dispose();
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
          { opacity: 0, color: "#ff5500" },
          {
            opacity: 1,
            color: "var(--foreground)",
            duration: 0.25,
            stagger: 0.08,
            ease: "power1.in",
            delay: 0.3,
          }
        );

        // 2. Title mask reveal
        gsap.fromTo(
          ".swiss-hero-title-line",
          { y: "110%", clipPath: "inset(0% 0% 100% 0%)" },
          {
            y: "0%",
            clipPath: "inset(0% 0% 0% 0%)",
            duration: 0.8,
            ease: "power3.out",
            delay: 0.5,
          }
        );

        // 3. Tagline
        gsap.fromTo(
          ".swiss-hero-tagline",
          { opacity: 0, y: 20 },
          { opacity: 1, y: 0, duration: 0.6, ease: "power2.out", delay: 0.9 }
        );

        // 4. Description
        gsap.fromTo(
          ".swiss-hero-description",
          { opacity: 0, y: 15 },
          { opacity: 1, y: 0, duration: 0.5, ease: "power2.out", delay: 1.1 }
        );

        // 5. CTAs
        gsap.fromTo(
          ".swiss-hero-actions",
          { opacity: 0, y: 15 },
          { opacity: 1, y: 0, duration: 0.5, ease: "power2.out", delay: 1.3 }
        );

        // 6. Scanline glow
        gsap.fromTo(
          ".swiss-hero-scanline",
          { scaleX: 0 },
          {
            scaleX: 1,
            duration: 1.2,
            ease: "power2.inOut",
            delay: 0.8,
          }
        );
      });
    },
    { scope: containerRef }
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
            <span className="swiss-hero-title-line">VantaDB</span>
          </span>
        </h1>

        <h2 className="swiss-hero-tagline">
          The database that thinks with you.
        </h2>

        <p className="swiss-hero-description">
          One pip install. Vector search, SQL, and full-text search in a single
          binary. Zero servers. Zero ops. Sub-millisecond.
        </p>

        <div className="swiss-hero-actions">
          <button
            onClick={handleCopy}
            className="swiss-button-primary"
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
          <Link to="/docs" className="swiss-button-ghost">
            Read Docs
          </Link>
        </div>
      </div>

      {/* Bottom scanline */}
      <div className="swiss-hero-scanline" />
    </section>
  );
}
