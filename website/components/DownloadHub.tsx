'use client';

import React from 'react';
import { InstallCmd } from './InstallCmd';

const INSTALL_OPTIONS = [
  {
    id: 'git-build',
    label: 'Build from Source (Recommended)',
    cmd: 'git clone https://github.com/abuzarkhan1/gitx.git && cd gitx && cargo build --release',
    desc: 'Compiles all 11 workspace crates locally with release optimizations',
  },
  {
    id: 'cargo-git',
    label: 'Install via Cargo (Git)',
    cmd: 'cargo install --git https://github.com/abuzarkhan1/gitx.git gitx-cli',
    desc: 'Directly builds and installs the gitx binary to $HOME/.cargo/bin',
  },
  {
    id: 'cargo-tui',
    label: 'Launch Interactive TUI',
    cmd: 'cargo run --release -p gitx-tui',
    desc: 'Spins up the 60 FPS Ratatui dashboard directly from repository root',
  },
];

export function DownloadHub() {
  return (
    <div style={{ display: 'flex', flexDirection: 'column', gap: '2rem' }}>
      {/* Platform & Build Overview Cards */}
      <div className="grid-2">
        {/* macOS & Linux Card */}
        <div className="bento-card" style={{ padding: '2.25rem' }}>
          <div className="shine-layer" />
          <div style={{ marginBottom: '1.5rem' }}>
            <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', marginBottom: '0.5rem' }}>
              <h3 style={{ fontSize: '1.5rem', fontWeight: 800, color: '#ffffff' }}>macOS &amp; Linux</h3>
              <span style={{ fontSize: '0.75rem', fontFamily: 'var(--font-mono)', background: 'rgba(255,255,255,0.08)', padding: '0.2rem 0.5rem', borderRadius: '4px' }}>
                Native Rust
              </span>
            </div>
            <p style={{ fontFamily: 'var(--font-mono)', fontSize: '0.78rem', color: '#a1a1aa' }}>
              Apple Silicon (ARM64), Intel (x86_64), and Linux distributions. Compiles with standard Cargo toolchain.
            </p>
          </div>

          <div style={{ display: 'flex', flexDirection: 'column', gap: '0.75rem' }}>
            <a
              href="https://github.com/abuzarkhan1/gitx"
              target="_blank"
              rel="noreferrer"
              className="btn-primary"
              style={{ width: '100%', textAlign: 'center' }}
            >
              View Repository on GitHub →
            </a>
          </div>
        </div>

        {/* Windows & Cross-Platform Card */}
        <div className="bento-card" style={{ padding: '2.25rem' }}>
          <div className="shine-layer" />
          <div style={{ marginBottom: '1.5rem' }}>
            <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', marginBottom: '0.5rem' }}>
              <h3 style={{ fontSize: '1.5rem', fontWeight: 800, color: '#ffffff' }}>Windows &amp; Cross-Platform</h3>
              <span style={{ fontSize: '0.75rem', fontFamily: 'var(--font-mono)', background: 'rgba(255,255,255,0.08)', padding: '0.2rem 0.5rem', borderRadius: '4px' }}>
                MSVC / GNU
              </span>
            </div>
            <p style={{ fontFamily: 'var(--font-mono)', fontSize: '0.78rem', color: '#a1a1aa' }}>
              Full support for Windows Terminal, PowerShell, and CI/CD pipelines via native MSVC compilation.
            </p>
          </div>

          <div style={{ display: 'flex', flexDirection: 'column', gap: '0.75rem' }}>
            <a
              href="https://github.com/abuzarkhan1/gitx/releases"
              target="_blank"
              rel="noreferrer"
              className="btn-secondary"
              style={{ width: '100%', textAlign: 'center' }}
            >
              GitHub Releases &amp; Changelogs →
            </a>
          </div>
        </div>
      </div>

      {/* Build & Installation Commands */}
      <div className="bento-card" style={{ padding: '2rem' }}>
        <div className="shine-layer" />
        <div style={{ marginBottom: '1.25rem', display: 'flex', alignItems: 'center', justifyContent: 'space-between', flexWrap: 'wrap', gap: '1rem' }}>
          <div>
            <h4 style={{ fontSize: '1.15rem', fontWeight: 800, color: '#ffffff', marginBottom: '0.25rem' }}>
              Source &amp; Cargo Installation
            </h4>
          </div>

          <a
            href="https://github.com/abuzarkhan1/gitx"
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
            GitHub Repository →
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

export default DownloadHub;
