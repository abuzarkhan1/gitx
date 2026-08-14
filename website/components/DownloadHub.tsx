'use client';

import React from 'react';
import { InstallCmd } from './InstallCmd';

const INSTALL_OPTIONS = [
  { id: 'curl', label: 'One-Line Curl', cmd: 'curl -fsSL https://gitx.sh/install.sh | sh', desc: 'Detects OS and CPU arch automatically' },
  { id: 'cargo', label: 'Cargo (crates.io)', cmd: 'cargo install gitx-cli --locked', desc: 'Builds from crates.io with locked dependencies' },
  { id: 'brew', label: 'Homebrew (macOS / Linux)', cmd: 'brew install abuzarkhan1/tap/gitx', desc: 'Native formula for macOS & Linuxbrew' },
  { id: 'git', label: 'Build from Source (Git)', cmd: 'git clone https://github.com/abuzarkhan1/gitx.git && cd gitx && cargo build --release', desc: 'Compile the 11 crates locally with optimizations' },
];

export function DownloadHub() {
  return (
    <div style={{ display: 'flex', flexDirection: 'column', gap: '2rem' }}>
      {/* Platform Binary Cards */}
      <div className="grid-2">
        {/* macOS Card */}
        <div className="bento-card" style={{ padding: '2.25rem' }}>
          <div className="shine-layer" />
          <div style={{ marginBottom: '1.5rem' }}>
            <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', marginBottom: '0.5rem' }}>
              <h3 style={{ fontSize: '1.5rem', fontWeight: 800, color: '#ffffff' }}>macOS</h3>
              <span style={{ fontSize: '0.75rem', fontFamily: 'var(--font-mono)', background: 'rgba(255,255,255,0.08)', padding: '0.2rem 0.5rem', borderRadius: '4px' }}>
                Universal Binary
              </span>
            </div>
            <p style={{ fontFamily: 'var(--font-mono)', fontSize: '0.78rem', color: '#a1a1aa' }}>
              Apple Silicon (M1/M2/M3/M4) &amp; Intel x86_64
            </p>
          </div>

          <div style={{ display: 'flex', flexDirection: 'column', gap: '0.75rem' }}>
            <a
              href="https://github.com/abuzarkhan1/gitx/releases/latest"
              target="_blank"
              rel="noreferrer"
              className="btn-primary"
              style={{ width: '100%' }}
            >
              Download for Apple Silicon (ARM64)
            </a>
            <a
              href="https://github.com/abuzarkhan1/gitx/releases/latest"
              target="_blank"
              rel="noreferrer"
              className="btn-secondary"
              style={{ width: '100%' }}
            >
              Download for Intel (x86_64)
            </a>
          </div>
        </div>

        {/* Linux & Windows Card */}
        <div className="bento-card" style={{ padding: '2.25rem' }}>
          <div className="shine-layer" />
          <div style={{ marginBottom: '1.5rem' }}>
            <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', marginBottom: '0.5rem' }}>
              <h3 style={{ fontSize: '1.5rem', fontWeight: 800, color: '#ffffff' }}>Linux &amp; Windows</h3>
              <span style={{ fontSize: '0.75rem', fontFamily: 'var(--font-mono)', background: 'rgba(255,255,255,0.08)', padding: '0.2rem 0.5rem', borderRadius: '4px' }}>
                Multi-Arch
              </span>
            </div>
            <p style={{ fontFamily: 'var(--font-mono)', fontSize: '0.78rem', color: '#a1a1aa' }}>
              Debian/Ubuntu .deb, Tarballs, and Windows x64 .exe
            </p>
          </div>

          <div style={{ display: 'flex', flexDirection: 'column', gap: '0.75rem' }}>
            <a
              href="https://github.com/abuzarkhan1/gitx/releases/latest"
              target="_blank"
              rel="noreferrer"
              className="btn-primary"
              style={{ width: '100%' }}
            >
              Download Linux (.deb / .tar.gz)
            </a>
            <a
              href="https://github.com/abuzarkhan1/gitx/releases/latest"
              target="_blank"
              rel="noreferrer"
              className="btn-secondary"
              style={{ width: '100%' }}
            >
              Download Windows (x64 .exe)
            </a>
          </div>
        </div>
      </div>

      {/* Package Manager / Command-Line Installation Methods */}
      <div className="bento-card" style={{ padding: '2rem' }}>
        <div className="shine-layer" />
        <div style={{ marginBottom: '1.25rem', display: 'flex', alignItems: 'center', justifyContent: 'space-between', flexWrap: 'wrap', gap: '1rem' }}>
          <div>
            <h4 style={{ fontSize: '1.15rem', fontWeight: 800, color: '#ffffff', marginBottom: '0.25rem' }}>
              Package Managers &amp; CLI Installation
            </h4>
            <p style={{ fontSize: '0.85rem', color: '#a1a1aa' }}>
              Install GitX directly in your terminal with your preferred package manager.
            </p>
          </div>

          <a
            href="https://github.com/abuzarkhan1/gitx/releases"
            target="_blank"
            rel="noreferrer"
            style={{
              display: 'inline-flex',
              alignItems: 'center',
              gap: '0.4rem',
              fontSize: '0.82rem',
              color: '#ffffff',
              fontFamily: 'var(--font-mono)',
              textDecoration: 'underline',
            }}
          >
            All GitHub Releases →
          </a>
        </div>

        <div style={{ display: 'flex', flexDirection: 'column', gap: '0.85rem' }}>
          {INSTALL_OPTIONS.map((opt) => (
            <div key={opt.id}>
              <div style={{ display: 'flex', justifyContent: 'space-between', marginBottom: '0.35rem', fontSize: '0.75rem', fontFamily: 'var(--font-mono)' }}>
                <span style={{ color: '#ffffff', fontWeight: 700 }}>{opt.label}</span>
                <span style={{ color: '#71717a' }}>{opt.desc}</span>
              </div>
              <InstallCmd cmd={opt.cmd} />
            </div>
          ))}
        </div>
      </div>
    </div>
  );
}
