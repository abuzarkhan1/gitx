'use client';

import React, { useState, useEffect, useRef } from 'react';
import Link from 'next/link';
import { usePathname } from 'next/navigation';

const NAV_LINKS = [
  { label: 'Architecture', href: '/#architecture' },
  { label: 'Features', href: '/#features' },
  { label: 'CLI', href: '/#cli' },
  { label: 'Benchmarks', href: '/#benchmarks' },
  { label: 'Download', href: '/#download' },
  { label: 'About', href: '/about' },
  { label: 'Contact', href: '/contact' },
];

function GitHubIcon({ size = 16 }: { size?: number }) {
  return (
    <svg width={size} height={size} viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" aria-hidden="true">
      <path d="M15 22v-4a4.8 4.8 0 0 0-1-3.5c3 0 6-2 6-5.5.08-1.25-.27-2.48-1-3.5.28-1.15.28-2.35 0-3.5 0 0-1 0-3 1.5-2.64-.5-5.36-.5-8 0C6 2 5 2 5 2c-.3 1.15-.3 2.35 0 3.5A5.403 5.403 0 0 0 4 9c0 3.5 3 5.5 6 5.5-.39.49-.68 1.05-.85 1.65-.17.6-.22 1.23-.15 1.85v4" />
      <path d="M9 18c-4.51 2-5-2-7-2" />
    </svg>
  );
}

export function Navbar() {
  const pathname = usePathname();
  const [scrolled, setScrolled] = useState(false);
  const [bannerVisible, setBannerVisible] = useState(true);
  const [menuOpen, setMenuOpen] = useState(false);

  useEffect(() => {
    const onScroll = () => setScrolled(window.scrollY > 15);
    window.addEventListener('scroll', onScroll, { passive: true });
    return () => window.removeEventListener('scroll', onScroll);
  }, []);

  useEffect(() => {
    document.body.style.overflow = menuOpen ? 'hidden' : '';
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === 'Escape' && menuOpen) setMenuOpen(false);
    };
    window.addEventListener('keydown', handleKeyDown);
    return () => {
      document.body.style.overflow = '';
      window.removeEventListener('keydown', handleKeyDown);
    };
  }, [menuOpen]);

  return (
    <>
      {/* Accessible Skip Link */}
      <a
        href="#main-content"
        style={{
          position: 'absolute',
          top: '-9999px',
          left: '1rem',
          zIndex: 100,
          background: '#ffffff',
          color: '#000000',
          padding: '0.5rem 1rem',
          borderRadius: '0.35rem',
          fontWeight: 800,
          fontFamily: 'var(--font-space)',
        }}
        onFocus={(e) => (e.currentTarget.style.top = '1rem')}
        onBlur={(e) => (e.currentTarget.style.top = '-9999px')}
      >
        Skip to main content
      </a>

      <header
        role="banner"
        style={{
          position: 'fixed',
          top: 0,
          left: 0,
          right: 0,
          zIndex: 50,
          transition: 'background 0.3s ease, border-color 0.3s ease',
          background: scrolled ? 'rgba(8, 8, 10, 0.95)' : 'rgba(8, 8, 10, 0.8)',
          backdropFilter: 'blur(20px)',
          WebkitBackdropFilter: 'blur(20px)',
          borderBottom: scrolled ? '1px solid rgba(255, 255, 255, 0.08)' : '1px solid transparent',
        }}
      >
        {/* Integrated Announcement Banner */}
        {bannerVisible && (
          <div
            style={{
              background: 'linear-gradient(90deg, rgba(255,255,255,0.06), rgba(255,255,255,0.02))',
              borderBottom: '1px solid rgba(255, 255, 255, 0.06)',
              padding: '0.45rem 1.5rem',
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'center',
              gap: '0.75rem',
              fontSize: '0.78rem',
              fontFamily: 'var(--font-mono)',
              color: '#d4d4d8',
              position: 'relative',
            }}
          >
            <span style={{ color: '#ffffff', fontWeight: 700 }}>⚡ GitX v0.1.0 Released:</span>
            <span>11 native Rust crates &amp; local SQLite FTS5 archaeology.</span>
            <Link
              href="/#download"
              style={{
                color: '#ffffff',
                textDecoration: 'underline',
                fontWeight: 700,
                marginLeft: '0.25rem',
              }}
            >
              Get Started →
            </Link>
            <button
              onClick={() => setBannerVisible(false)}
              aria-label="Dismiss banner"
              style={{
                position: 'absolute',
                right: '1rem',
                top: '50%',
                transform: 'translateY(-50%)',
                background: 'transparent',
                border: 'none',
                color: '#71717a',
                cursor: 'pointer',
                fontSize: '0.85rem',
                padding: '0.25rem',
                lineHeight: 1,
              }}
            >
              ✕
            </button>
          </div>
        )}

        {/* Main Navbar Bar */}
        <div
          style={{
            maxWidth: '1200px',
            margin: '0 auto',
            padding: '0 1.5rem',
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'space-between',
            height: '60px',
          }}
        >
          {/* Wordmark */}
          <Link
            href="/"
            style={{
              fontSize: '1.75rem',
              fontWeight: 900,
              color: '#ffffff',
              letterSpacing: '-0.04em',
              userSelect: 'none',
              display: 'flex',
              alignItems: 'center',
              gap: '2px',
            }}
          >
            <span>Git</span>
            <em className="vg-serif" style={{ fontStyle: 'italic', fontWeight: 700, color: '#ffffff', fontSize: '1.85rem' }}>
              X
            </em>
          </Link>

          {/* Desktop Nav */}
          <nav aria-label="Main navigation" style={{ display: 'none', alignItems: 'center', gap: '2rem' }} className="desktop-nav">
            {NAV_LINKS.map((link) => {
              const isActive = pathname === link.href || (pathname === '/' && link.href.startsWith('/#'));
              return (
                <Link
                  key={link.label}
                  href={link.href}
                  style={{
                    fontSize: '0.92rem',
                    fontWeight: 800,
                    letterSpacing: '-0.02em',
                    color: pathname === link.href ? '#ffffff' : '#a1a1aa',
                    transition: 'color 0.15s ease',
                  }}
                  className="hover-text-white"
                >
                  {link.label}
                </Link>
              );
            })}
          </nav>

          {/* Desktop Actions */}
          <div style={{ display: 'none', alignItems: 'center', gap: '1.25rem' }} className="desktop-actions">
            <a
              href="https://github.com/abuzarkhan1/gitx"
              target="_blank"
              rel="noopener noreferrer"
              style={{
                color: '#a1a1aa',
                display: 'flex',
                alignItems: 'center',
                gap: '0.5rem',
                fontSize: '0.9rem',
                fontWeight: 700,
              }}
              className="hover-text-white"
            >
              <GitHubIcon size={16} />
              <span>GitHub</span>
            </a>
            <Link
              href="/#download"
              className="btn-primary"
              style={{
                height: '38px',
                padding: '0 1.25rem',
                fontSize: '0.85rem',
              }}
            >
              Download
            </Link>
          </div>

          {/* Mobile Hamburger */}
          <button
            onClick={() => setMenuOpen((v) => !v)}
            aria-label={menuOpen ? 'Close menu' : 'Open menu'}
            aria-expanded={menuOpen}
            style={{
              display: 'flex',
              flexDirection: 'column',
              justifyContent: 'center',
              alignItems: 'center',
              width: '40px',
              height: '40px',
              background: 'transparent',
              border: 'none',
              cursor: 'pointer',
              gap: '5px',
            }}
            className="mobile-hamburger"
          >
            <span
              style={{
                display: 'block',
                height: '2px',
                width: '22px',
                background: '#ffffff',
                transition: 'all 0.2s ease',
                transform: menuOpen ? 'rotate(45deg) translateY(7px)' : 'none',
              }}
            />
            <span
              style={{
                display: 'block',
                height: '2px',
                width: '22px',
                background: '#ffffff',
                transition: 'all 0.2s ease',
                opacity: menuOpen ? 0 : 1,
              }}
            />
            <span
              style={{
                display: 'block',
                height: '2px',
                width: '22px',
                background: '#ffffff',
                transition: 'all 0.2s ease',
                transform: menuOpen ? 'rotate(-45deg) translateY(-7px)' : 'none',
              }}
            />
          </button>
        </div>
      </header>

      {/* Mobile Drawer */}
      {menuOpen && (
        <nav
          aria-label="Mobile navigation"
          style={{
            position: 'fixed',
            inset: 0,
            zIndex: 48,
            background: 'rgba(8, 8, 10, 0.98)',
            backdropFilter: 'blur(30px)',
            WebkitBackdropFilter: 'blur(30px)',
            display: 'flex',
            flexDirection: 'column',
            justifyContent: 'flex-start',
            alignItems: 'center',
            gap: '1.75rem',
            padding: '6rem 2rem 3rem',
            overflowY: 'auto',
          }}
        >
          {NAV_LINKS.map((link) => (
            <Link
              key={link.label}
              href={link.href}
              onClick={() => setMenuOpen(false)}
              style={{
                fontSize: '1.5rem',
                fontWeight: 900,
                color: '#ffffff',
                letterSpacing: '-0.03em',
              }}
            >
              {link.label}
            </Link>
          ))}
          <Link
            href="/#download"
            onClick={() => setMenuOpen(false)}
            className="btn-primary"
            style={{ width: '100%', maxWidth: '280px', textAlign: 'center', marginTop: '1rem' }}
          >
            Download GitX
          </Link>
        </nav>
      )}

      <style jsx>{`
        @media (min-width: 768px) {
          .desktop-nav {
            display: flex !important;
          }
          .desktop-actions {
            display: flex !important;
          }
          .mobile-hamburger {
            display: none !important;
          }
        }
      `}</style>
    </>
  );
}
