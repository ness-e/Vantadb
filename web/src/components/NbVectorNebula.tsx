import { Link } from "@tanstack/react-router";
import { memo, useCallback, useEffect, useRef } from "react";
import { gsap, useGSAP } from "../lib/gsap";
import "../styles/vector-nebula.css";

interface Particle {
  x: number;
  y: number;
  vx: number;
  vy: number;
  radius: number;
  alpha: number;
  hue: number;
  pulse: number;
  pulseSpeed: number;
}

const CONNECTION_DIST = 140;
const MOUSE_RADIUS = 180;
const PARTICLE_COUNT = 120;
const COLORS = [
  { h: 16, s: "100%", l: "55%" },
  { h: 20, s: "90%", l: "50%" },
  { h: 12, s: "100%", l: "60%" },
  { h: 25, s: "80%", l: "45%" },
  { h: 360, s: "0%", l: "70%" },
];

export const NbVectorNebula = memo(function NbVectorNebula() {
  const sectionRef = useRef<HTMLElement>(null);
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const wrapperRef = useRef<HTMLDivElement>(null);
  const particlesRef = useRef<Particle[]>([]);
  const mouseRef = useRef({ x: -1e6, y: -1e6 });
  const frameRef = useRef(0);
  const dimsRef = useRef({ w: 0, h: 0 });

  const initParticles = useCallback((w: number, h: number) => {
    const p: Particle[] = [];
    for (let i = 0; i < PARTICLE_COUNT; i++) {
      const color = COLORS[i % COLORS.length];
      p.push({
        x: Math.random() * w,
        y: Math.random() * h,
        vx: (Math.random() - 0.5) * 0.4,
        vy: (Math.random() - 0.5) * 0.4,
        radius: 1 + Math.random() * 2.5,
        alpha: 0.3 + Math.random() * 0.7,
        hue: color.h,
        pulse: Math.random() * Math.PI * 2,
        pulseSpeed: 0.01 + Math.random() * 0.03,
      });
    }
    particlesRef.current = p;
  }, []);

  const draw = useCallback(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;
    const ctx = canvas.getContext("2d");
    if (!ctx) return;

    const { w, h } = dimsRef.current;
    const particles = particlesRef.current;
    const mouse = mouseRef.current;

    ctx.clearRect(0, 0, w, h);

    for (let i = 0; i < particles.length; i++) {
      const p = particles[i];

      p.pulse += p.pulseSpeed;
      const pulseAlpha = p.alpha * (0.6 + 0.4 * Math.sin(p.pulse));

      const dx = mouse.x - p.x;
      const dy = mouse.y - p.y;
      const dist = Math.hypot(dx, dy);
      if (dist < MOUSE_RADIUS && dist > 1) {
        const force = (MOUSE_RADIUS - dist) / MOUSE_RADIUS;
        p.vx -= (dx / dist) * force * 0.15;
        p.vy -= (dy / dist) * force * 0.15;
      }

      p.vx += (Math.random() - 0.5) * 0.02;
      p.vy += (Math.random() - 0.5) * 0.02;
      p.vx *= 0.98;
      p.vy *= 0.98;

      p.x += p.vx;
      p.y += p.vy;

      if (p.x < -20) p.x = w + 20;
      if (p.x > w + 20) p.x = -20;
      if (p.y < -20) p.y = h + 20;
      if (p.y > h + 20) p.y = -20;

      const gradient = ctx.createRadialGradient(p.x, p.y, 0, p.x, p.y, p.radius * 4);
      gradient.addColorStop(
        0,
        `hsla(${p.hue}, ${COLORS[i % COLORS.length].s}, ${COLORS[i % COLORS.length].l}, ${pulseAlpha})`,
      );
      gradient.addColorStop(
        0.3,
        `hsla(${p.hue}, ${COLORS[i % COLORS.length].s}, ${COLORS[i % COLORS.length].l}, ${pulseAlpha * 0.3})`,
      );
      gradient.addColorStop(
        1,
        `hsla(${p.hue}, ${COLORS[i % COLORS.length].s}, ${COLORS[i % COLORS.length].l}, 0)`,
      );

      ctx.beginPath();
      ctx.arc(p.x, p.y, p.radius * 4, 0, Math.PI * 2);
      ctx.fillStyle = gradient;
      ctx.fill();

      ctx.beginPath();
      ctx.arc(p.x, p.y, p.radius, 0, Math.PI * 2);
      ctx.fillStyle = `hsla(${p.hue}, ${COLORS[i % COLORS.length].s}, ${COLORS[i % COLORS.length].l}, ${pulseAlpha})`;
      ctx.fill();
    }

    for (let i = 0; i < particles.length; i++) {
      for (let j = i + 1; j < particles.length; j++) {
        const a = particles[i];
        const b = particles[j];
        const dx = a.x - b.x;
        const dy = a.y - b.y;
        const dist = Math.hypot(dx, dy);

        if (dist < CONNECTION_DIST) {
          const alpha = (1 - dist / CONNECTION_DIST) * 0.15;
          ctx.beginPath();
          ctx.moveTo(a.x, a.y);
          ctx.lineTo(b.x, b.y);
          ctx.strokeStyle = `hsla(${(a.hue + b.hue) / 2}, 80%, 60%, ${alpha})`;
          ctx.lineWidth = 0.5;
          ctx.stroke();
        }
      }
    }

    frameRef.current = requestAnimationFrame(draw);
  }, []);

  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;

    const resize = () => {
      const wrapper = wrapperRef.current;
      if (!wrapper) return;
      const w = wrapper.clientWidth;
      const h = wrapper.clientHeight;
      const dpr = window.devicePixelRatio || 1;
      canvas.width = w * dpr;
      canvas.height = h * dpr;
      canvas.style.width = `${w}px`;
      canvas.style.height = `${h}px`;
      dimsRef.current = { w, h };
      const ctx = canvas.getContext("2d");
      if (ctx) ctx.scale(dpr, dpr);
      initParticles(w, h);
    };

    resize();

    const onMouse = (e: MouseEvent) => {
      const wrapper = wrapperRef.current;
      if (!wrapper) return;
      const rect = wrapper.getBoundingClientRect();
      mouseRef.current = {
        x: e.clientX - rect.left,
        y: e.clientY - rect.top,
      };
    };

    const onLeave = () => {
      mouseRef.current = { x: -1e6, y: -1e6 };
    };

    window.addEventListener("resize", resize);
    window.addEventListener("mousemove", onMouse);
    window.addEventListener("mouseleave", onLeave);

    frameRef.current = requestAnimationFrame(draw);

    return () => {
      window.removeEventListener("resize", resize);
      window.removeEventListener("mousemove", onMouse);
      window.removeEventListener("mouseleave", onLeave);
      cancelAnimationFrame(frameRef.current);
    };
  }, [draw, initParticles]);

  useGSAP(
    () => {
      const mm = gsap.matchMedia();
      mm.add("(prefers-reduced-motion: no-preference)", () => {
        const tl = gsap.timeline({
          scrollTrigger: { trigger: sectionRef.current, start: "top 75%" },
        });
        tl.fromTo(
          ".nb-nebula-eyebrow",
          { opacity: 0, y: 20 },
          { opacity: 1, y: 0, duration: 0.4, ease: "var(--ease-swiss)" },
        );
        tl.fromTo(
          ".nb-nebula-title",
          { opacity: 0, y: 24 },
          { opacity: 1, y: 0, duration: 0.5, ease: "var(--ease-swiss)" },
          "-=0.2",
        );
        tl.fromTo(".nb-nebula-sub", { opacity: 0 }, { opacity: 1, duration: 0.3 }, "-=0.15");
        tl.fromTo(
          ".nb-nebula-actions",
          { opacity: 0, y: 12 },
          { opacity: 1, y: 0, duration: 0.3 },
          "-=0.1",
        );
      });
    },
    { scope: sectionRef },
  );

  return (
    <section ref={sectionRef} className="nb-nebula" aria-label="Closing statement">
      <div ref={wrapperRef} className="nb-nebula-canvas-wrap">
        <canvas ref={canvasRef} aria-hidden="true" />
      </div>

      <div className="nb-nebula-content">
        <div className="nb-nebula-inner">
          <p className="nb-nebula-eyebrow">[VECTOR SPACE]</p>
          <h2 className="nb-nebula-title">
            Build the future
            <br />
            <span className="nb-nebula-accent">of AI memory.</span>
          </h2>
          <p className="nb-nebula-sub">
            VantaDB is open source, Apache-2.0, and ready to ship.
            <br />
            One binary. Zero servers.
          </p>
          <div className="nb-nebula-actions">
            <Link to="/docs" className="nb-btn">
              GET STARTED
            </Link>
            <a
              href="https://github.com/vantadb/vantadb"
              target="_blank"
              rel="noopener noreferrer"
              className="nb-btn nb-btn--ghost"
            >
              STAR ON GITHUB
            </a>
          </div>
        </div>
      </div>

      <div className="nb-nebula-credit">
        <span className="nb-nebula-credit-line" />
        <span className="nb-nebula-credit-text">
          v0.1.5 &mdash; Apache 2.0 &mdash; Rust + Python
        </span>
        <span className="nb-nebula-credit-line" />
      </div>
    </section>
  );
});
