import React from 'react';
import Link from 'next/link';
import { StarsCanvas } from '@/components/StarsCanvas';
import { InstallCmd } from '@/components/InstallCmd';
import { TerminalMockup } from '@/components/TerminalMockup';
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

const REAL_BENCHMARKS = [
  { name: 'GitX Cached Health Query', time: '< 1ms', pct: 4, note: 'Instant memory-mapped SQLite index' },
  { name: 'GitX SQLite FTS5 Search (500 commits)', time: '412 µs', pct: 8, note: 'BM25 inverted full-text lookup' },
  { name: 'GitX Incremental Refresh (+1 commit)', time: '40ms', pct: 24, note: 'Parallel delta packfile update' },
  { name: 'Standard git log linear walk', time: '1,840ms', pct: 72, note: 'Linear process-spawning traversal' },
  { name: 'Electron GUI Clients (Cold Scan)', time: '8,400ms', pct: 100, note: 'Heavy DOM and multi-process overhead' },
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
          {/* Status Badge */}
          <div style={{ display: 'inline-flex', alignItems: 'center', justifyContent: 'center', marginBottom: '1.75rem' }}>
            <div
              style={{
                display: 'inline-flex',
                alignItems: 'center',
                gap: '0.65rem',
                borderRadius: '9999px',
                border: '1px solid rgba(255, 255, 255, 0.12)',
                background: 'rgba(255, 255, 255, 0.03)',
                padding: '0.4rem 1rem',
                fontSize: '0.75rem',
                fontFamily: 'var(--font-mono)',
                color: '#d4d4d8',
                backdropFilter: 'blur(12px)',
                lineHeight: 1.4,
              }}
            >
              <span
                style={{
                  width: '7px',
                  height: '7px',
                  borderRadius: '50%',
                  background: '#ffffff',
                  boxShadow: '0 0 10px #ffffff',
                }}
              />
              <span>⚡ v0.1.0 Released · 11 Modular Rust Crates · 100% Offline</span>
            </div>
          </div>

          {/* Dominant Hero Headline */}
          <h1
            className="vg-hero-heading"
            style={{
              fontSize: 'clamp(2.5rem, 6.2vw, 4.8rem)',
              color: '#ffffff',
              marginBottom: '1.5rem',
              letterSpacing: '-0.04em',
            }}
          >
            Terminal-Native Git Archaeology &amp;{' '}
            <em className="vg-serif vg-text-glow" style={{ color: '#ffffff', fontWeight: 400 }}>
              Repository Intelligence
            </em>
          </h1>

          {/* Subtitle */}
          <p
            style={{
              fontSize: 'clamp(1rem, 2vw, 1.25rem)',
              color: '#a1a1aa',
              lineHeight: 1.65,
              maxWidth: '780px',
              margin: '0 auto 2.5rem',
            }}
          >
            Stop wrestling with fragmented <code style={{ color: '#ffffff', background: 'rgba(255,255,255,0.06)', padding: '0.1rem 0.4rem', borderRadius: '4px' }}>git log</code>,
            broken blame trails, and untraceable refactors. GitX indexes your entire repository history into an ultra-fast local SQLite engine for sub-millisecond archaeology, 6-score health metrics, and interactive terminal exploration.
          </p>

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
              Download GitX Free →
            </a>
            <a href="#architecture" className="btn-secondary">
              Explore 11 Crates
            </a>
          </div>

          {/* One-Line Install Chip */}
          <div style={{ maxWidth: '540px', margin: '0 auto 2.5rem' }}>
            <InstallCmd cmd="curl -fsSL https://gitx.sh/install.sh | sh" label="Quick Install" />
          </div>

          {/* Trust Invariant Triad */}
          <div
            style={{
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'center',
              gap: '1.5rem',
              flexWrap: 'wrap',
              fontSize: '0.78rem',
              fontFamily: 'var(--font-mono)',
              color: '#71717a',
            }}
          >
            <span>⚡ 11 Native Rust Crates</span>
            <span>·</span>
            <span>🔍 Sub-Millisecond SQLite FTS5</span>
            <span>·</span>
            <span>🔒 100% Offline &amp; Air-Gapped</span>
            <span>·</span>
            <span>🖥️ 60 FPS Ratatui TUI</span>
          </div>
        </div>
      </section>

      {/* ═══════════════════════════════
          INTERACTIVE TERMINAL SHOWCASE
          ═══════════════════════════════ */}
      <section style={{ padding: '0 1.5rem 5rem' }}>
        <div className="container" style={{ maxWidth: '960px' }}>
          <TerminalMockup />
        </div>
      </section>

      {/* ═══════════════════════════════
          PERFORMANCE BENCHMARKS
          ═══════════════════════════════ */}
      <section
        id="benchmarks"
        style={{
          borderTop: '1px solid rgba(255, 255, 255, 0.08)',
          padding: 'var(--section-py-lg) 1.5rem',
        }}
      >
        <div className="container" style={{ maxWidth: '960px' }}>
          <div style={{ textAlign: 'center', marginBottom: '3.5rem' }}>
            <div className="section-label">
              <span>⚡ CRITERION BENCHMARKS</span>
            </div>
            <h2
              style={{
                fontSize: 'clamp(2rem, 4vw, 3rem)',
                fontWeight: 800,
                color: '#ffffff',
                letterSpacing: '-0.04em',
                marginBottom: '1rem',
              }}
            >
              Engineered for velocity.{' '}
              <em className="vg-serif" style={{ color: '#ffffff', fontWeight: 400 }}>
                Sub-millisecond latency.
              </em>
            </h2>
            <p style={{ color: '#a1a1aa', fontSize: '1rem', maxWidth: '650px', margin: '0 auto' }}>
              Micro-benchmarks conducted across real-world repositories validate microsecond query retrieval and fast parallel packfile updates.
            </p>
          </div>

          <div className="bento-card" style={{ padding: '2.25rem' }}>
            <div className="shine-layer" />
            <div style={{ display: 'flex', flexDirection: 'column', gap: '1.25rem' }}>
              {REAL_BENCHMARKS.map((b, i) => (
                <div key={i} style={{ display: 'flex', flexDirection: 'column', gap: '0.4rem' }}>
                  <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', fontSize: '0.85rem' }}>
                    <span style={{ fontWeight: 700, color: b.pct < 30 ? '#ffffff' : '#d4d4d8' }}>
                      {b.name}
                    </span>
                    <div style={{ display: 'flex', alignItems: 'center', gap: '0.75rem', fontFamily: 'var(--font-mono)' }}>
                      <span style={{ color: '#71717a', fontSize: '0.75rem' }}>{b.note}</span>
                      <span style={{ color: '#ffffff', fontWeight: 800, fontSize: '0.9rem' }}>{b.time}</span>
                    </div>
                  </div>
                  <div style={{ height: '8px', background: 'rgba(255, 255, 255, 0.05)', borderRadius: '4px', overflow: 'hidden' }}>
                    <div
                      style={{
                        width: `${b.pct}%`,
                        height: '100%',
                        background: b.pct < 30 ? '#ffffff' : 'rgba(255, 255, 255, 0.25)',
                        borderRadius: '4px',
                        transition: 'width 0.5s ease',
                      }}
                    />
                  </div>
                </div>
              ))}
            </div>

            <div
              style={{
                marginTop: '1.75rem',
                paddingTop: '1.25rem',
                borderTop: '1px solid rgba(255, 255, 255, 0.06)',
                display: 'flex',
                justifyContent: 'space-between',
                flexWrap: 'wrap',
                gap: '0.75rem',
                fontFamily: 'var(--font-mono)',
                fontSize: '0.75rem',
                color: '#71717a',
              }}
            >
              <span>Benchmarked with Criterion.rs &amp; real Git trees</span>
              <span style={{ color: '#ffffff' }}>Zero network latency · Local disk I/O</span>
            </div>
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
            <div className="section-label">
              <span>🏛️ 11-CRATE SYSTEMS ARCHITECTURE</span>
            </div>
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
            <p style={{ color: '#a1a1aa', fontSize: '1rem', maxWidth: '650px', margin: '0 auto' }}>
              Built as a clean modular pipeline from the low-level object database parser up to the high-performance Ratatui terminal dashboard.
            </p>
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
            <div className="section-label">
              <span>🚀 DEEP REPOSITORY CAPABILITIES</span>
            </div>
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
            <p style={{ color: '#a1a1aa', fontSize: '1rem', maxWidth: '650px', margin: '0 auto' }}>
              Everything your team needs to inspect legacy code, audit maintenance risk, and explore Git history at terminal velocity.
            </p>
          </div>

          <div className="grid-3">
            {/* Card 1 */}
            <div className="bento-card" style={{ minHeight: '340px' }}>
              <div className="shine-layer" />
              <div>
                <span className="section-label">01 / ARCHITECTURE</span>
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
                <span className="section-label">02 / ARCHAEOLOGY</span>
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
                <span className="section-label">03 / METRICS</span>
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
                <span className="section-label">04 / SEARCH</span>
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
                <span className="section-label">05 / RECOVERY</span>
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
                <span className="section-label">06 / PRIVACY</span>
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
          <div className="section-label">
            <span>💻 COMMAND-LINE WORKFLOW</span>
          </div>
          <h2
            style={{
              fontSize: 'clamp(2rem, 4vw, 3rem)',
              fontWeight: 800,
              color: '#ffffff',
              letterSpacing: '-0.04em',
              marginBottom: '1rem',
            }}
          >
            One binary.{' '}
            <em className="vg-serif" style={{ color: '#ffffff', fontWeight: 400 }}>
              Unlimited insights.
            </em>
          </h2>
          <p style={{ color: '#a1a1aa', fontSize: '1rem', maxWidth: '600px', margin: '0 auto 2.5rem' }}>
            Integrates into any terminal, script, or CI pipeline with human-friendly colored output or machine-readable JSON.
          </p>

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
            <div className="section-label">
              <span>📦 FREE &amp; OPEN SOURCE</span>
            </div>
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
            <div className="section-label">
              <span>❓ FREQUENTLY ASKED QUESTIONS</span>
            </div>
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
