"use client";

import React, { useEffect, useRef } from "react";

export function ObservatoryGridCanvas() {
  const canvasRef = useRef<HTMLCanvasElement>(null);

  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;

    const ctx = canvas.getContext("2d", { alpha: true });
    if (!ctx) return;

    let animationFrameId: number;
    let width = (canvas.width = canvas.offsetWidth);
    let height = (canvas.height = canvas.offsetHeight);

    const handleResize = () => {
      if (!canvas) return;
      width = canvas.width = canvas.offsetWidth;
      height = canvas.height = canvas.offsetHeight;
    };

    window.addEventListener("resize", handleResize);

    // Particle beacons
    const pulses = Array.from({ length: 6 }, () => ({
      x: Math.random() * width,
      y: Math.random() * height,
      radius: Math.random() * 2 + 1,
      speedX: (Math.random() - 0.5) * 0.3,
      speedY: (Math.random() - 0.5) * 0.3,
      alpha: Math.random() * 0.5 + 0.2,
      maxAlpha: Math.random() * 0.6 + 0.3,
      pulseSpeed: Math.random() * 0.02 + 0.008,
      growing: true,
    }));

    let mouseX = width / 2;
    let mouseY = height / 2;
    let targetMouseX = mouseX;
    let targetMouseY = mouseY;

    const handleMouseMove = (e: MouseEvent) => {
      const rect = canvas.getBoundingClientRect();
      targetMouseX = e.clientX - rect.left;
      targetMouseY = e.clientY - rect.top;
    };

    window.addEventListener("mousemove", handleMouseMove, { passive: true });

    const gridSize = 48;

    const render = () => {
      ctx.clearRect(0, 0, width, height);

      // Smooth mouse interpolation
      mouseX += (targetMouseX - mouseX) * 0.05;
      mouseY += (targetMouseY - mouseY) * 0.05;

      // Draw subtle hairline grid dots & intersections
      ctx.fillStyle = "rgba(32, 32, 32, 0.06)";
      for (let x = 0; x < width; x += gridSize) {
        for (let y = 0; y < height; y += gridSize) {
          const dist = Math.hypot(x - mouseX, y - mouseY);
          const proximity = Math.max(0, 1 - dist / 280);

          if (proximity > 0) {
            ctx.fillStyle = `rgba(255, 104, 44, ${proximity * 0.35})`;
            ctx.fillRect(x - 1, y - 1, 2, 2);
          } else {
            ctx.fillStyle = "rgba(32, 32, 32, 0.05)";
            ctx.fillRect(x - 0.5, y - 0.5, 1, 1);
          }
        }
      }

      // Draw animated drifting beacons
      pulses.forEach((p) => {
        p.x += p.speedX;
        p.y += p.speedY;

        if (p.x < 0) p.x = width;
        if (p.x > width) p.x = 0;
        if (p.y < 0) p.y = height;
        if (p.y > height) p.y = 0;

        if (p.growing) {
          p.alpha += p.pulseSpeed;
          if (p.alpha >= p.maxAlpha) p.growing = false;
        } else {
          p.alpha -= p.pulseSpeed;
          if (p.alpha <= 0.1) p.growing = true;
        }

        // Draw soft glow
        const gradient = ctx.createRadialGradient(p.x, p.y, 0, p.x, p.y, 24);
        gradient.addColorStop(0, `rgba(255, 104, 44, ${p.alpha * 0.5})`);
        gradient.addColorStop(1, "rgba(255, 104, 44, 0)");

        ctx.fillStyle = gradient;
        ctx.beginPath();
        ctx.arc(p.x, p.y, 24, 0, Math.PI * 2);
        ctx.fill();

        // Core dot
        ctx.fillStyle = `rgba(255, 104, 44, ${p.alpha})`;
        ctx.beginPath();
        ctx.arc(p.x, p.y, p.radius, 0, Math.PI * 2);
        ctx.fill();
      });

      animationFrameId = requestAnimationFrame(render);
    };

    render();

    return () => {
      window.removeEventListener("resize", handleResize);
      window.removeEventListener("mousemove", handleMouseMove);
      cancelAnimationFrame(animationFrameId);
    };
  }, []);

  return (
    <canvas
      ref={canvasRef}
      className="absolute inset-0 w-full h-full pointer-events-none -z-10 opacity-70"
      aria-hidden="true"
    />
  );
}
