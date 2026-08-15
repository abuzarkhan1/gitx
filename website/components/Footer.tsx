'use client';

import React, { useState, useEffect, useRef } from 'react';
import Link from 'next/link';

function GitHubIcon({ size = 16 }: { size?: number }) {
  return (
    <svg width={size} height={size} viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" aria-hidden="true">
      <path d="M15 22v-4a4.8 4.8 0 0 0-1-3.5c3 0 6-2 6-5.5.08-1.25-.27-2.48-1-3.5.28-1.15.28-2.35 0-3.5 0 0-1 0-3 1.5-2.64-.5-5.36-.5-8 0C6 2 5 2 5 2c-.3 1.15-.3 2.35 0 3.5A5.403 5.403 0 0 0 4 9c0 3.5 3 5.5 6 5.5-.39.49-.68 1.05-.85 1.65-.17.6-.22 1.23-.15 1.85v4" />
      <path d="M9 18c-4.51 2-5-2-7-2" />
    </svg>
  );
}

export function Footer() {
  const footerRef = useRef<HTMLElement>(null);
  const [mousePos, setMousePos] = useState({ x: 50, y: 50 });

  useEffect(() => {
    const handleMouseMove = (e: MouseEvent) => {
      if (!footerRef.current) return;
      const rect = footerRef.current.getBoundingClientRect();
      const x = ((e.clientX - rect.left) / rect.width) * 100;
      const y = ((e.clientY - rect.top) / rect.height) * 100;
      setMousePos({ x: Math.max(0, Math.min(100, x)), y: Math.max(0, Math.min(100, y)) });
    };

    window.addEventListener('mousemove', handleMouseMove);
    return () => window.removeEventListener('mousemove', handleMouseMove);
  }, []);

  return (
    <footer
      ref={footerRef}
      style={{
        position: 'relative',
        borderTop: '1px solid rgba(255, 255, 255, 0.08)',
        background: '#08080a',
        paddingTop: '4rem',
        paddingBottom: '3rem',
        color: '#ffffff',
        overflow: 'hidden',
        userSelect: 'none',
      }}
    >
      <div style={{ maxWidth: '1200px', margin: '0 auto', padding: '0 1.5rem', position: 'relative', zIndex: 10 }}>
        {/* Giant Mouse-following Radial Glow Brand Mark (Signature Brand Header) */}
        <div
          style={{
            position: 'relative',
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
            margin: '1.5rem 0',
            padding: '2rem 0',
            overflow: 'hidden',
            pointerEvents: 'none',
            minHeight: '160px',
          }}
        >
          <div
            style={{
              fontSize: 'clamp(5rem, 16vw, 13rem)',
              fontWeight: 900,
              letterSpacing: '-0.06em',
              color: 'transparent',
              WebkitBackgroundClip: 'text',
              lineHeight: 1,
              transition: 'all 0.3s ease',
              textAlign: 'center',
              backgroundImage: `radial-gradient(circle at ${mousePos.x}% ${mousePos.y}%, rgba(255,255,255,1) 0%, rgba(255,255,255,0.35) 55%, rgba(255,255,255,0.08) 100%)`,
              textShadow: '0 0 50px rgba(255,255,255,0.15)',
            }}
          >
            GITX
          </div>
        </div>

        {/* Sleek Bottom Bar */}
        <div
          style={{
            display: 'flex',
            flexWrap: 'wrap',
            alignItems: 'center',
            justifyContent: 'space-between',
            gap: '1.5rem',
            paddingTop: '2rem',
            borderTop: '1px solid rgba(255, 255, 255, 0.08)',
          }}
        >
          <div style={{ display: 'flex', alignItems: 'center', gap: '1.25rem' }}>
            <div style={{ fontFamily: 'var(--font-mono)', fontSize: '0.75rem', fontWeight: 700, textTransform: 'uppercase', letterSpacing: '0.1em', color: '#a1a1aa' }}>
              © 2026 Abuzar Khan
            </div>
          </div>

          <div style={{ display: 'flex', alignItems: 'center', gap: '1.5rem' }}>
            <a
              href="https://github.com/abuzarkhan1/gitx"
              target="_blank"
              rel="noreferrer"
              style={{
                display: 'flex',
                alignItems: 'center',
                gap: '0.5rem',
                fontWeight: 800,
                fontSize: '1rem',
                color: '#ffffff',
                letterSpacing: '-0.02em',
              }}
              className="hover-text-white"
            >
              <GitHubIcon size={16} />
              <span>GitHub</span>
            </a>
          </div>

          <div style={{ display: 'flex', alignItems: 'center', gap: '0.5rem', fontFamily: 'var(--font-mono)', fontSize: '0.75rem', fontWeight: 700, textTransform: 'uppercase', letterSpacing: '0.1em', color: '#a1a1aa' }}>
            <span style={{ width: '8px', height: '8px', borderRadius: '50%', background: '#ffffff', animation: 'ping 1.5s cubic-bezier(0,0,0.2,1) infinite' }} />
            <span>Systems Operational</span>
          </div>
        </div>
      </div>
    </footer>
  );
}
