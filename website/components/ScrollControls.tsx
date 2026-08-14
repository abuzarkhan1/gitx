'use client';

import React, { useState, useEffect } from 'react';

export function ScrollControls() {
  const [scrollProgress, setScrollProgress] = useState(0);
  const [showBackToTop, setShowBackToTop] = useState(false);

  useEffect(() => {
    let ticking = false;

    const onScroll = () => {
      if (!ticking) {
        window.requestAnimationFrame(() => {
          const scrollY = window.scrollY;
          const docHeight = document.documentElement.scrollHeight - window.innerHeight;
          const progress = docHeight > 0 ? (scrollY / docHeight) * 100 : 0;
          setScrollProgress(progress);
          setShowBackToTop(scrollY > 400);
          ticking = false;
        });
        ticking = true;
      }
    };

    window.addEventListener('scroll', onScroll, { passive: true });
    return () => window.removeEventListener('scroll', onScroll);
  }, []);

  const scrollToTop = () => {
    window.scrollTo({ top: 0, behavior: 'smooth' });
  };

  return (
    <>
      {/* 2px Fixed Scroll Progress Bar */}
      <div
        role="progressbar"
        aria-valuenow={Math.round(scrollProgress)}
        aria-valuemin={0}
        aria-valuemax={100}
        aria-label="Page scroll progress"
        style={{
          position: 'fixed',
          top: 0,
          left: 0,
          zIndex: 90,
          height: '2px',
          background: '#ffffff',
          width: `${scrollProgress}%`,
          transition: 'width 0.05s linear',
          pointerEvents: 'none',
        }}
      />

      {/* Floating Back to Top Button */}
      <button
        onClick={scrollToTop}
        aria-label="Scroll back to top"
        style={{
          position: 'fixed',
          bottom: '2rem',
          right: '2rem',
          zIndex: 50,
          width: '42px',
          height: '42px',
          borderRadius: '50%',
          background: '#ffffff',
          color: '#000000',
          border: 'none',
          cursor: showBackToTop ? 'pointer' : 'default',
          boxShadow: '0 10px 25px rgba(0, 0, 0, 0.8)',
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'center',
          fontWeight: 'bold',
          opacity: showBackToTop ? 1 : 0,
          transform: showBackToTop ? 'translateY(0)' : 'translateY(12px)',
          pointerEvents: showBackToTop ? 'auto' : 'none',
          transition: 'opacity 0.25s ease, transform 0.25s ease',
        }}
      >
        ↑
      </button>
    </>
  );
}
