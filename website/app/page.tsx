import React from 'react';
import Link from 'next/link';
import { StarsCanvas } from '@/components/StarsCanvas';
import { InstallCmd } from '@/components/InstallCmd';
import { DownloadHub } from '@/components/DownloadHub';
import { FaqAccordion } from '@/components/FaqAccordion';
import {
  RustPulseRing,
  LineageForensicsVisual,
  HealthScorecardVisual,
  SqliteFts5Visual,
  DisasterRecoveryVisual,
  AirGapPrivacyVisual,
} from '@/components/CardVisuals';

const ARCHITECTURE_PILLARS = [
  {
    num: '01',
    title: 'Parallel Packfile Engine',
    tag: 'Rust & Rayon',
    desc: 'Bypasses slow shell-spawning wrappers by interfacing directly with raw Git object databases and packfiles via pure Rust and multithreaded Rayon pipelines.',
    specs: ['Direct packfile reader', 'Zero shell spawns', 'Sub-15ms parsing for 1.5k commits'],
  },
  {
    num: '02',
    title: 'Local SQLite FTS5 Inverted Index',
    tag: 'rusqlite & BM25',
    desc: 'Maintains an incremental on-disk database with BM25 full-text search triggers, enabling sub-millisecond keyword lookup across all commit messages, diffs, authors, and symbols.',
    specs: ['BM25 ranking algorithm', 'Sub-400µs query latency', 'Automatic schema migrations'],
  },
  {
    num: '03',
    title: 'Continuous File Lineage Forensics',
    tag: 'DAG Traversal',
    desc: 'Survives complex refactors, splits, and renames along the Git commit graph where standard git blame fails. Tracks code attribution across years with 100% confidence.',
    specs: ['Full history rename tracking', 'Line-level introduction history', 'Mainline traversal'],
  },
  {
    num: '04',
    title: 'Deterministic 6-Score Health & Risk',
    tag: 'Explainable Formulas',
    desc: 'Quantifies repository maintainability across 6 distinct sub-dimensions: Code Hotspots, Single-Maintainer Ownership, Branch Hygiene, Change Volatility, Architecture Stability, and Recovery Risk.',
    specs: ['0–100 weighted scale', 'Zero AI hallucinations', 'Deterministic JSON for CI/CD'],
  },
  {
    num: '05',
    title: '60 FPS Ratatui Terminal Interface',
    tag: 'Crossterm & TUI',
    desc: 'Full-screen, keyboard-driven terminal dashboard rendering 14 interactive diagnostic views with sub-frame response times, zero GPU bloat, and smooth navigation.',
    specs: ['14 interactive views', 'Vim keybindings (j/k/tab)', '60 FPS terminal rendering'],
  },
];

export default function HomePage() {
  return (
    <div style={{ position: 'relative', width: '100%', overflowX: 'hidden' }}>
      {/* Background Starfield Canvas */}
      <StarsCanvas />

      {/* ═══════════════════════════════
          HERO SECTION
          ═══════════════════════════════ */}
      <section
        style={{
          position: 'relative',
          paddingTop: '8.5rem',
          paddingBottom: '4.5rem',
          textAlign: 'center',
        }}
      >
        <div className="container" style={{ maxWidth: '960px' }}>
          {/* Dominant Hero Headline */}
          <h1
            className="vg-hero-heading"
            style={{
              fontSize: 'clamp(2.5rem, 6.2vw, 4.8rem)',
              color: '#ffffff',
              marginBottom: '2rem',
              letterSpacing: '-0.04em',
            }}
          >
            Terminal-Native Git Archaeology &amp;{' '}
            <em className="vg-serif vg-text-glow" style={{ color: '#ffffff', fontWeight: 400 }}>
              Repository Intelligence
            </em>
          </h1>

          {/* Primary Action Button Cluster */}
          <div
            style={{
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'center',
              gap: '1rem',
              flexWrap: 'wrap',
              marginBottom: '2rem',
            }}
          >
            <a href="#download" className="btn-primary">
              Get GitX on GitHub →
            </a>
            <a href="#architecture" className="btn-secondary">
              Explore 11 Crates
            </a>
          </div>

          {/* One-Line Install Chip */}
          <div style={{ maxWidth: '640px', margin: '0 auto' }}>
            <InstallCmd cmd="git clone https://github.com/abuzarkhan1/gitx.git && cd gitx && cargo build --release" label="Build from Source" />
          </div>
        </div>
      </section>

      {/* ═══════════════════════════════
          ARCHITECTURE (5 PILLARS)
          ═══════════════════════════════ */}
      <section
        id="architecture"
        style={{
          borderTop: '1px solid rgba(255, 255, 255, 0.08)',
          padding: 'var(--section-py-lg) 1.5rem',
        }}
      >
        <div className="container" style={{ maxWidth: '1000px' }}>
          <div style={{ textAlign: 'center', marginBottom: '4rem' }}>
            <h2
              style={{
                fontSize: 'clamp(2rem, 4vw, 3rem)',
                fontWeight: 800,
                color: '#ffffff',
                letterSpacing: '-0.04em',
                marginBottom: '1rem',
              }}
            >
              Modular systems design.{' '}
              <em className="vg-serif" style={{ color: '#ffffff', fontWeight: 400 }}>
                Every layer explainable.
              </em>
            </h2>
          </div>

          <div style={{ display: 'flex', flexDirection: 'column', gap: '2.5rem' }}>
            {ARCHITECTURE_PILLARS.map((pillar) => (
              <div
                key={pillar.num}
                className="bento-card"
                style={{ padding: '2.25rem' }}
              >
                <div className="shine-layer" />
                <div style={{ display: 'flex', alignItems: 'flex-start', justifyContent: 'space-between', flexWrap: 'wrap', gap: '1rem', marginBottom: '1rem' }}>
                  <div style={{ display: 'flex', alignItems: 'center', gap: '1.25rem' }}>
                    <div
                      style={{
                        width: '48px',
                        height: '48px',
                        borderRadius: '50%',
                        background: 'rgba(255, 255, 255, 0.06)',
                        border: '1px solid rgba(255, 255, 255, 0.15)',
                        display: 'flex',
                        alignItems: 'center',
                        justifyContent: 'center',
                        color: '#ffffff',
                        fontFamily: 'var(--font-mono)',
                        fontWeight: 900,
                        fontSize: '1rem',
                        flexShrink: 0,
                      }}
                    >
                      {pillar.num}
                    </div>
                    <div>
                      <h3 style={{ fontSize: '1.35rem', fontWeight: 800, color: '#ffffff' }}>
                        {pillar.title}
                      </h3>
                    </div>
                  </div>
                  <span
                    style={{
                      fontFamily: 'var(--font-mono)',
                      fontSize: '0.75rem',
                      background: 'rgba(255, 255, 255, 0.06)',
                      padding: '0.25rem 0.75rem',
                      borderRadius: '9999px',
                      color: '#ffffff',
                      border: '1px solid rgba(255, 255, 255, 0.1)',
                    }}
                  >
                    {pillar.tag}
                  </span>
                </div>

                <p style={{ color: '#a1a1aa', fontSize: '0.95rem', lineHeight: 1.6, marginBottom: '1.25rem' }}>
                  {pillar.desc}
                </p>

                <div style={{ display: 'flex', gap: '0.5rem', flexWrap: 'wrap' }}>
                  {pillar.specs.map((spec, si) => (
                    <span
                      key={si}
                      style={{
                        fontFamily: 'var(--font-mono)',
                        fontSize: '0.72rem',
                        color: '#d4d4d8',
                        background: 'rgba(0, 0, 0, 0.4)',
                        border: '1px solid rgba(255, 255, 255, 0.06)',
                        padding: '0.2rem 0.6rem',
                        borderRadius: '4px',
                      }}
                    >
                      ✓ {spec}
                    </span>
                  ))}
                </div>
              </div>
            ))}
          </div>
        </div>
      </section>

      {/* ═══════════════════════════════
          BENTO CAPABILITIES (6 CARDS)
          ═══════════════════════════════ */}
      <section
        id="features"
        style={{
          borderTop: '1px solid rgba(255, 255, 255, 0.08)',
          padding: 'var(--section-py-lg) 1.5rem',
        }}
      >
        <div className="container">
          <div style={{ textAlign: 'center', marginBottom: '4rem' }}>
            <h2
              style={{
                fontSize: 'clamp(2rem, 4vw, 3rem)',
                fontWeight: 800,
                color: '#ffffff',
                letterSpacing: '-0.04em',
                marginBottom: '1rem',
              }}
            >
              Zero walled gardens.{' '}
              <em className="vg-serif" style={{ color: '#ffffff', fontWeight: 400 }}>
                100% open intelligence.
              </em>
            </h2>
          </div>

          <div className="grid-3">
            {/* Card 1 */}
            <div className="bento-card" style={{ minHeight: '340px' }}>
              <div className="shine-layer" />
              <div>
                <h3 style={{ fontSize: '1.25rem', fontWeight: 800, color: '#ffffff', marginBottom: '0.5rem' }}>
                  11 Modular Rust Crates
                </h3>
                <p style={{ fontSize: '0.88rem', color: '#a1a1aa', lineHeight: 1.5, marginBottom: '1.25rem' }}>
                  Clean workspace division separating raw packfile parsing, database storage, DAG lineage, and Ratatui presentation.
                </p>
              </div>
              <RustPulseRing />
            </div>

            {/* Card 2 */}
            <div className="bento-card" style={{ minHeight: '340px' }}>
              <div className="shine-layer" />
              <div>
                <h3 style={{ fontSize: '1.25rem', fontWeight: 800, color: '#ffffff', marginBottom: '0.5rem' }}>
                  File Lineage &amp; Renames
                </h3>
                <p style={{ fontSize: '0.88rem', color: '#a1a1aa', lineHeight: 1.5, marginBottom: '1.25rem' }}>
                  Maintains continuous attribution across structural renames, directory moves, and multi-branch merges over years.
                </p>
              </div>
              <LineageForensicsVisual />
            </div>

            {/* Card 3 */}
            <div className="bento-card" style={{ minHeight: '340px' }}>
              <div className="shine-layer" />
              <div>
                <h3 style={{ fontSize: '1.25rem', fontWeight: 800, color: '#ffffff', marginBottom: '0.5rem' }}>
                  Deterministic 6-Score Health
                </h3>
                <p style={{ fontSize: '0.88rem', color: '#a1a1aa', lineHeight: 1.5, marginBottom: '1.25rem' }}>
                  Weighted transparent scoring across Hotspots, Ownership, Branch hygiene, Volatility, Architecture, and Recovery.
                </p>
              </div>
              <HealthScorecardVisual />
            </div>

            {/* Card 4 */}
            <div className="bento-card" style={{ minHeight: '340px' }}>
              <div className="shine-layer" />
              <div>
                <h3 style={{ fontSize: '1.25rem', fontWeight: 800, color: '#ffffff', marginBottom: '0.5rem' }}>
                  SQLite FTS5 BM25 Search
                </h3>
                <p style={{ fontSize: '0.88rem', color: '#a1a1aa', lineHeight: 1.5, marginBottom: '1.25rem' }}>
                  Sub-millisecond full-text queries over commit messages, authors, branches, and symbol definitions.
                </p>
              </div>
              <SqliteFts5Visual />
            </div>

            {/* Card 5 */}
            <div className="bento-card" style={{ minHeight: '340px' }}>
              <div className="shine-layer" />
              <div>
                <h3 style={{ fontSize: '1.25rem', fontWeight: 800, color: '#ffffff', marginBottom: '0.5rem' }}>
                  Reflog &amp; Dangling Objects
                </h3>
                <p style={{ fontSize: '0.88rem', color: '#a1a1aa', lineHeight: 1.5, marginBottom: '1.25rem' }}>
                  Resurrect orphaned commits and lost rebase experiments with read-only inspection and unified patch export.
                </p>
              </div>
              <DisasterRecoveryVisual />
            </div>

            {/* Card 6 */}
            <div className="bento-card" style={{ minHeight: '340px' }}>
              <div className="shine-layer" />
              <div>
                <h3 style={{ fontSize: '1.25rem', fontWeight: 800, color: '#ffffff', marginBottom: '0.5rem' }}>
                  100% Offline &amp; Private
                </h3>
                <p style={{ fontSize: '0.88rem', color: '#a1a1aa', lineHeight: 1.5, marginBottom: '1.25rem' }}>
                  Zero external network calls, zero telemetry, zero accounts. Your code and history stay strictly on your local machine.
                </p>
              </div>
              <AirGapPrivacyVisual />
            </div>
          </div>
        </div>
      </section>

      {/* ═══════════════════════════════
          CLI WORKFLOW SHOWCASE
          ═══════════════════════════════ */}
      <section
        id="cli"
        style={{
          borderTop: '1px solid rgba(255, 255, 255, 0.08)',
          padding: 'var(--section-py-lg) 1.5rem',
        }}
      >
        <div className="container" style={{ maxWidth: '800px', textAlign: 'center' }}>
          <h2
            style={{
              fontSize: 'clamp(2rem, 4vw, 3rem)',
              fontWeight: 800,
              color: '#ffffff',
              letterSpacing: '-0.04em',
              marginBottom: '2.5rem',
            }}
          >
            One binary.{' '}
            <em className="vg-serif" style={{ color: '#ffffff', fontWeight: 400 }}>
              Unlimited insights.
            </em>
          </h2>

          <div style={{ display: 'flex', flexDirection: 'column', gap: '0.85rem', textAlign: 'left' }}>
            <InstallCmd cmd="gitx scan" label="Build local SQLite index" />
            <InstallCmd cmd="gitx health" label="Emit 6-score health scorecard" />
            <InstallCmd cmd="gitx hotspots --limit 5" label="Rank high-risk files" />
            <InstallCmd cmd="gitx lineage src/engine.rs" label="Archaeology & rename lineage" />
            <InstallCmd cmd="gitx tui" label="Launch 60 FPS interactive dashboard" />
          </div>
        </div>
      </section>

      {/* ═══════════════════════════════
          TESTIMONIAL QUOTE
          ═══════════════════════════════ */}
      <section
        style={{
          borderTop: '1px solid rgba(255, 255, 255, 0.08)',
          padding: 'var(--section-py-lg) 1.5rem',
          textAlign: 'center',
        }}
      >
        <div className="container" style={{ maxWidth: '800px' }}>
          <blockquote
            style={{
              fontSize: 'clamp(1.2rem, 2.5vw, 1.6rem)',
              color: '#ffffff',
              fontStyle: 'italic',
              lineHeight: 1.6,
              marginBottom: '1.5rem',
            }}
            className="vg-serif"
          >
            &ldquo;Git has always stored the complete story of your codebase, but extracting answers with git log and git blame was painful. GitX turns the commit graph into an instantaneous, queryable knowledge engine.&rdquo;
          </blockquote>
          <div style={{ fontFamily: 'var(--font-mono)', fontSize: '0.85rem', color: '#a1a1aa' }}>
            <span style={{ color: '#ffffff', fontWeight: 700 }}>Abuzar Khan</span> · Lead Systems Engineer, GitX
          </div>
        </div>
      </section>

      {/* ═══════════════════════════════
          DOWNLOAD HUB
          ═══════════════════════════════ */}
      <section
        id="download"
        style={{
          borderTop: '1px solid rgba(255, 255, 255, 0.08)',
          padding: 'var(--section-py-lg) 1.5rem',
        }}
      >
        <div className="container" style={{ maxWidth: '960px' }}>
          <div style={{ textAlign: 'center', marginBottom: '3.5rem' }}>
            <h2
              style={{
                fontSize: 'clamp(2rem, 4vw, 3rem)',
                fontWeight: 800,
                color: '#ffffff',
                letterSpacing: '-0.04em',
                marginBottom: '1rem',
              }}
            >
              Get started in seconds.{' '}
              <em className="vg-serif" style={{ color: '#ffffff', fontWeight: 400 }}>
                100% Free.
              </em>
            </h2>
            <p style={{ color: '#a1a1aa', fontSize: '1rem', maxWidth: '600px', margin: '0 auto' }}>
              Download pre-built universal binaries for macOS, Linux, and Windows, or install directly via Cargo and Homebrew.
            </p>
          </div>

          <DownloadHub />
        </div>
      </section>

      {/* ═══════════════════════════════
          FAQ ACCORDION
          ═══════════════════════════════ */}
      <section
        id="faq"
        style={{
          borderTop: '1px solid rgba(255, 255, 255, 0.08)',
          padding: 'var(--section-py-lg) 1.5rem',
        }}
      >
        <div className="container" style={{ maxWidth: '800px' }}>
          <div style={{ textAlign: 'center', marginBottom: '3.5rem' }}>
            <h2
              style={{
                fontSize: 'clamp(2rem, 4vw, 3rem)',
                fontWeight: 800,
                color: '#ffffff',
                letterSpacing: '-0.04em',
                marginBottom: '1rem',
              }}
            >
              Frequently asked{' '}
              <em className="vg-serif" style={{ color: '#ffffff', fontWeight: 400 }}>
                questions.
              </em>
            </h2>
          </div>

          <FaqAccordion />
        </div>
      </section>
    </div>
  );
}
