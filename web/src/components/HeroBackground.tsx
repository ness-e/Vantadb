import { useRef, useEffect } from "react";

const AMBER = { r: 255, g: 85, b: 0 };
const PARTICLE_COUNT = 35;
const CONNECT_DIST = 140;
const GLOW_RADIUS = 60;
const MAX_SPEED = 0.4;
const DRIFT = 0.015;

export function HeroBackground() {
  const canvasRef = useRef<HTMLCanvasElement>(null);

  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;
    const ctx = canvas.getContext("2d");
    if (!ctx) return;

    const reduceMotion = window.matchMedia("(prefers-reduced-motion: reduce)").matches;
    let animId: number;

    interface Particle {
      x: number;
      y: number;
      vx: number;
      vy: number;
      r: number;
      a: number;
    }

    let particles: Particle[] = [];
    let w = 0;
    let h = 0;

    function resize() {
      const dpr = window.devicePixelRatio || 1;
      w = window.innerWidth;
      h = window.innerHeight;
      canvas!.width = w * dpr;
      canvas!.height = h * dpr;
      canvas!.style.width = w + "px";
      canvas!.style.height = h + "px";
      ctx!.setTransform(dpr, 0, 0, dpr, 0, 0);
    }

    function init() {
      particles = Array.from({ length: PARTICLE_COUNT }, () => ({
        x: Math.random() * w,
        y: Math.random() * h,
        vx: (Math.random() - 0.5) * MAX_SPEED * 2,
        vy: (Math.random() - 0.5) * MAX_SPEED * 2,
        r: 1.5 + Math.random() * 2.5,
        a: 0.15 + Math.random() * 0.35,
      }));
    }

    function draw() {
      ctx!.clearRect(0, 0, w, h);

      for (const p of particles) {
        p.x += p.vx;
        p.y += p.vy;
        if (p.x < 0 || p.x > w) p.vx *= -1;
        if (p.y < 0 || p.y > h) p.vy *= -1;
        p.vx += (Math.random() - 0.5) * DRIFT;
        p.vy += (Math.random() - 0.5) * DRIFT;
        const spd = Math.hypot(p.vx, p.vy);
        if (spd > MAX_SPEED) {
          p.vx = (p.vx / spd) * MAX_SPEED;
          p.vy = (p.vy / spd) * MAX_SPEED;
        }
      }

      for (let i = 0; i < particles.length; i++) {
        for (let j = i + 1; j < particles.length; j++) {
          const a = particles[i];
          const b = particles[j];
          const dx = a.x - b.x;
          const dy = a.y - b.y;
          const dist = Math.hypot(dx, dy);
          if (dist < CONNECT_DIST) {
            const alpha = (1 - dist / CONNECT_DIST) * 0.12;
            ctx!.beginPath();
            ctx!.moveTo(a.x, a.y);
            ctx!.lineTo(b.x, b.y);
            ctx!.strokeStyle = `rgba(${AMBER.r},${AMBER.g},${AMBER.b},${alpha})`;
            ctx!.lineWidth = 0.5;
            ctx!.stroke();
          }
        }
      }

      for (const p of particles) {
        const glow = ctx!.createRadialGradient(p.x, p.y, 0, p.x, p.y, GLOW_RADIUS * (p.r / 3));
        glow.addColorStop(0, `rgba(${AMBER.r},${AMBER.g},${AMBER.b},${p.a * 0.12})`);
        glow.addColorStop(1, `rgba(${AMBER.r},${AMBER.g},${AMBER.b},0)`);
        ctx!.fillStyle = glow;
        ctx!.beginPath();
        ctx!.arc(p.x, p.y, GLOW_RADIUS * (p.r / 3), 0, Math.PI * 2);
        ctx!.fill();

        ctx!.beginPath();
        ctx!.arc(p.x, p.y, p.r, 0, Math.PI * 2);
        ctx!.fillStyle = `rgba(${AMBER.r},${AMBER.g},${AMBER.b},${p.a})`;
        ctx!.fill();
      }

      animId = requestAnimationFrame(draw);
    }

    function drawStatic() {
      for (const p of particles) {
        ctx!.beginPath();
        ctx!.arc(p.x, p.y, p.r, 0, Math.PI * 2);
        ctx!.fillStyle = `rgba(${AMBER.r},${AMBER.g},${AMBER.b},${p.a})`;
        ctx!.fill();
      }
    }

    resize();
    init();
    if (!reduceMotion) draw();
    else drawStatic();

    const onResize = () => {
      resize();
    };
    window.addEventListener("resize", onResize);
    return () => {
      cancelAnimationFrame(animId);
      window.removeEventListener("resize", onResize);
    };
  }, []);

  return <canvas ref={canvasRef} className="hero-canvas-bg" aria-hidden="true" />;
}
