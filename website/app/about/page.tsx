import type { Metadata } from 'next';

export const metadata: Metadata = {
  title: 'About GitX — Architecture & Vision',
  description: 'Learn about the 11-crate Rust architecture, core design invariants, and the engineering behind GitX.',
};

function GitHubIcon({ size = 16 }: { size?: number }) {
  return (
    <svg width={size} height={size} viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" aria-hidden="true">
      <path d="M15 22v-4a4.8 4.8 0 0 0-1-3.5c3 0 6-2 6-5.5.08-1.25-.27-2.48-1-3.5.28-1.15.28-2.35 0-3.5 0 0-1 0-3 1.5-2.64-.5-5.36-.5-8 0C6 2 5 2 5 2c-.3 1.15-.3 2.35 0 3.5A5.403 5.403 0 0 0 4 9c0 3.5 3 5.5 6 5.5-.39.49-.68 1.05-.85 1.65-.17.6-.22 1.23-.15 1.85v4" />
      <path d="M9 18c-4.51 2-5-2-7-2" />
    </svg>
  );
}

export default function AboutPage() {
  return (
    <div style={{ background: '#08080a', minHeight: '100vh', paddingTop: '8.5rem', paddingBottom: '6rem', color: '#ffffff' }}>
      <div className="container" style={{ maxWidth: '860px' }}>
        {/* Header */}
        <div style={{ textAlign: 'center', marginBottom: '4rem' }}>
          <h1 className="vg-hero-heading" style={{ fontSize: 'clamp(2.25rem, 5vw, 3.5rem)', color: '#ffffff', marginBottom: '1.25rem' }}>
            Engineered for <span className="vg-serif" style={{ color: '#ffffff', fontWeight: 400 }}>Deep Git Intelligence</span>
          </h1>
          <p style={{ color: '#a1a1aa', fontSize: '1.1rem', lineHeight: 1.6, maxWidth: '640px', margin: '0 auto' }}>
            GitX turns a Git repository&apos;s history, structure, changes, ownership, branches, dependencies, and recoverable work into a fast, interactive, explainable terminal experience.
          </p>
        </div>

        <div style={{ display: 'flex', flexDirection: 'column', gap: '2rem' }}>
          {/* Core Constraints Card */}
          <div className="bento-card" style={{ padding: '2.25rem' }}>
            <div className="shine-layer" />
            <h2 style={{ fontSize: '1.35rem', fontWeight: 800, color: '#ffffff', marginBottom: '1.25rem' }}>
              The 3 Architectural Invariants
            </h2>
            <div style={{ display: 'flex', flexDirection: 'column', gap: '1rem' }}>
              <div style={{ padding: '1.1rem', background: 'rgba(255, 255, 255, 0.02)', border: '1px solid rgba(255, 255, 255, 0.06)', borderRadius: '0.75rem' }}>
                <div style={{ fontWeight: 800, color: '#ffffff', marginBottom: '0.25rem', fontFamily: 'var(--font-mono)', fontSize: '0.85rem' }}>
                  1. Zero Network Calls
                </div>
                <div style={{ color: '#a1a1aa', fontSize: '0.9rem', lineHeight: 1.6 }}>
                  Everything runs directly against your local <code style={{ color: '#ffffff', background: 'rgba(255,255,255,0.08)', padding: '0.1rem 0.35rem', borderRadius: '4px' }}>.git</code> object database. Nothing is transmitted over the network.
                </div>
              </div>

              <div style={{ padding: '1.1rem', background: 'rgba(255, 255, 255, 0.02)', border: '1px solid rgba(255, 255, 255, 0.06)', borderRadius: '0.75rem' }}>
                <div style={{ fontWeight: 800, color: '#ffffff', marginBottom: '0.25rem', fontFamily: 'var(--font-mono)', fontSize: '0.85rem' }}>
                  2. Zero Accounts / Cloud Dependencies
                </div>
                <div style={{ color: '#a1a1aa', fontSize: '0.9rem', lineHeight: 1.6 }}>
                  No signup, no telemetry, no tracking tokens. You own 100% of your data and indices.
                </div>
              </div>

              <div style={{ padding: '1.1rem', background: 'rgba(255, 255, 255, 0.02)', border: '1px solid rgba(255, 255, 255, 0.06)', borderRadius: '0.75rem' }}>
                <div style={{ fontWeight: 800, color: '#ffffff', marginBottom: '0.25rem', fontFamily: 'var(--font-mono)', fontSize: '0.85rem' }}>
                  3. 100% Deterministic Calculations
                </div>
                <div style={{ color: '#a1a1aa', fontSize: '0.9rem', lineHeight: 1.6 }}>
                  Every score is a transparent, deterministic formula over raw git signals. Safe for automated CI/CD gating and compliance audits.
                </div>
              </div>
            </div>
          </div>

          {/* 11 Crates Breakdown */}
          <div className="bento-card" style={{ padding: '2.25rem' }}>
            <div className="shine-layer" />
            <h2 style={{ fontSize: '1.35rem', fontWeight: 800, color: '#ffffff', marginBottom: '1.25rem' }}>
              11-Crate Clean Rust Architecture
            </h2>
            <pre
              style={{
                fontFamily: 'var(--font-mono)',
                fontSize: '0.85rem',
                color: '#d4d4d8',
                background: 'rgba(0, 0, 0, 0.5)',
                padding: '1.25rem',
                borderRadius: '0.75rem',
                border: '1px solid rgba(255, 255, 255, 0.06)',
                lineHeight: 1.6,
                overflowX: 'auto',
              }}
            >
{`crates/
├── gitx-cli/        commands · clap dispatch · exit codes
├── gitx-core/       domain types · config · result types
├── gitx-git/        objects · refs · diffs · reflog (gix wrapper)
├── gitx-index/      initial + incremental scans · change detection
├── gitx-storage/    SQLite provider · migrations · transactions
├── gitx-history/    timeline · blame · lineage · renames
├── gitx-analysis/   metrics · hotspots · ownership · 6-score health
├── gitx-graph/      module graph · architecture dependencies
├── gitx-search/     SQLite FTS5 full-text search · BM25
├── gitx-services/   application facade (no business logic)
└── gitx-tui/        Ratatui views · keymaps · charts · themes`}
            </pre>
          </div>

          {/* Creator Attribution Card */}
          <div className="bento-card" style={{ padding: '2.25rem', display: 'flex', flexDirection: 'row', flexWrap: 'wrap', alignItems: 'center', justifyContent: 'space-between', gap: '1.5rem' }}>
            <div className="shine-layer" />
            <div>
              <div style={{ fontSize: '0.75rem', fontFamily: 'var(--font-mono)', color: '#a1a1aa', textTransform: 'uppercase', letterSpacing: '0.1em', marginBottom: '0.25rem', fontWeight: 700 }}>
                Engineered By
              </div>
              <div style={{ fontSize: '1.4rem', fontWeight: 900, color: '#ffffff' }}>
                Abuzar Khan
              </div>
              <div style={{ color: '#71717a', fontSize: '0.85rem' }}>
                Creator &amp; Maintainer · MIT Licensed
              </div>
            </div>

            <a
              href="https://github.com/abuzarkhan1/gitx"
              target="_blank"
              rel="noopener noreferrer"
              className="btn-secondary"
            >
              <GitHubIcon size={16} />
              <span>GitHub Repository</span>
            </a>
          </div>
        </div>
      </div>
    </div>
  );
}
