'use client';

import React, { useState } from 'react';
import { useClipboard } from './InstallCmd';

interface TerminalTab {
  id: string;
  label: string;
  command: string;
  summary: string;
  output: string[];
  isTui?: boolean;
}

const TERMINAL_TABS: TerminalTab[] = [
  {
    id: 'scan',
    label: 'gitx scan',
    command: 'gitx scan',
    summary: 'Parallel object tree indexing',
    output: [
      'Indexed 1,480 commits at .gitx/index.db (full scan)',
      '  commits      : 1,480 parsed in 14.2ms',
      '  branches     : 12 (1 default, 11 feature)',
      '  contributors : 8 unique commit authors',
      '  languages    : Rust (82.4%), TypeScript (11.2%), TOML (6.4%)',
      '✓ Index fresh and up to date with HEAD (0 unindexed objects)',
    ],
  },
  {
    id: 'lineage',
    label: 'gitx lineage',
    command: 'gitx lineage crates/gitx-core/src/index.rs',
    summary: 'Full file life & rename forensics',
    output: [
      'Lineage of crates/gitx-core/src/index.rs (newest first):',
      '  a8f2c19  2026-03-12 14:22:01 +0000  modified',
      '  3b41e90  2026-01-04 09:15:33 +0000  renamed from crates/gitx-scan/src/index.rs',
      '  f104d2e  2025-11-19 18:40:12 +0000  modified',
      '  c5079a1  2025-08-01 11:03:45 +0000  added',
      '✓ Tracked across 4 commits & 1 structural rename with zero history loss',
    ],
  },
  {
    id: 'hotspots',
    label: 'gitx hotspots',
    command: 'gitx hotspots --limit 4',
    summary: 'High-risk change hotspots',
    output: [
      'Hotspots (change/maintenance risk, 0–100):',
      '  crates/gitx-core/src/index.rs',
      '    risk score: 84/100 · churn: 142 commits · 4 authors · bus factor: 68%',
      '  crates/gitx-analysis/src/risk.rs',
      '    risk score: 71/100 · churn: 98 commits · 3 authors · bus factor: 54%',
      '  crates/gitx-tui/src/views/overview.rs',
      '    risk score: 63/100 · churn: 76 commits · 2 authors · bus factor: 89%',
      '  crates/gitx-git/src/diff.rs',
      '    risk score: 58/100 · churn: 52 commits · 3 authors · bus factor: 42%',
    ],
  },
  {
    id: 'health',
    label: 'gitx health',
    command: 'gitx health',
    summary: 'Composite repository health scorecard',
    output: [
      'Repository Health Scorecard',
      '  Commit hygiene         94/100  (clean messages, linear history)',
      '  Branch hygiene         88/100  (12 branches, 0 stale >90d)',
      '  Hotspot risk           82/100  (4 high-risk files under observation)',
      '  Ownership & bus-factor 76/100  (2 files with >85% single author)',
      '  Churn velocity         91/100  (142 commits / last 30 days)',
      '  Release rhythm         95/100  (regular semantic tags)',
      '  Overall Health: 87/100 · Grade: A (Healthy)',
    ],
  },
  {
    id: 'recovery',
    label: 'gitx recovery',
    command: 'gitx recovery',
    summary: 'Dangling commit & reflog archaeology',
    output: [
      'Reflog & Dangling Object Recovery (Read-Only)',
      '  HEAD@{0}  9c3e1b0 → a8f2c19  commit: refactor sqlite storage engine',
      '  HEAD@{1}  4d7a2c8 → 9c3e1b0  reset: moving to HEAD~1 (dangling commit saved)',
      '  HEAD@{2}  1e8f0a3 → 4d7a2c8  checkout: moving from feat/ast to main',
      '',
      'Unreachable Commits (Recoverable):',
      '  4d7a2c8  2026-04-18 10:12  "wip: experimental streaming graph parser"',
      '  → Run `gitx recovery export 4d7a2c8` to emit patch safely',
    ],
  },
  {
    id: 'tui',
    label: 'gitx tui',
    command: 'gitx tui',
    summary: 'Interactive 60 FPS Ratatui dashboard',
    isTui: true,
    output: [
      '┌─ GitX TUI v0.1.0 ────────────────────── [1] Overview  [2] Timeline  [3] Hotspots  [4] Health ─┐',
      '│ Repo: gitx (main @ a8f2c19)                                                                   │',
      '│ ┌─ Commits (1,480) ──────────────────┐ ┌─ Health Scorecard ─────────────────────────────────┐ │',
      '│ │ ● a8f2c19 refactor: sqlite FTS5    │ │ Overall: [████████████████████░░░░] 87/100 (Grade A)│ │',
      '│ │ ● 9c3e1b0 feat: petgraph code map  │ │ Churn: 91/100  Hygiene: 94/100  Branch: 88/100     │ │',
      '│ │ ● 3b41e90 fix: 60fps ratatui view  │ └────────────────────────────────────────────────────┘ │',
      '│ └────────────────────────────────────┘ ┌─ Selected Commit Detail ───────────────────────────┐ │',
      '│ ┌─ Hotspot Matrix ───────────────────┐ │ Author: Abuzar Khan · 2026-03-12 14:22:01 +0000      │ │',
      '│ │ 84/100 crates/gitx-core/src/index  │ │ Summary: refactor(storage): sqlite FTS5 index      │ │',
      '│ └────────────────────────────────────┘ └────────────────────────────────────────────────────┘ │',
      '└─ [Tab] Switch View · [/] Search · [j/k] Navigate · [q] Quit ─────────────────── 60 FPS ───────┘',
    ],
  },
];

export function TerminalMockup() {
  const [activeTabId, setActiveTabId] = useState('scan');
  const activeTab = TERMINAL_TABS.find((t) => t.id === activeTabId) || TERMINAL_TABS[0];
  const { copied, copy } = useClipboard(2000);

  return (
    <div className="terminal-mock" role="region" aria-label="Interactive GitX Terminal Preview">
      {/* Header Bar */}
      <div className="terminal-header" style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', flexWrap: 'wrap', gap: '0.75rem' }}>
        <div style={{ display: 'flex', alignItems: 'center', gap: '0.75rem' }}>
          <div className="traffic-dots">
            <span className="traffic-dot traffic-red" />
            <span className="traffic-dot traffic-yellow" />
            <span className="traffic-dot traffic-green" />
          </div>
          <span style={{ fontFamily: 'var(--font-mono)', fontSize: '0.75rem', color: '#a1a1aa' }} className="terminal-title">
            gitx — terminal session
          </span>
        </div>

        {/* Tab Controls */}
        <div className="terminal-tabs" role="tablist">
          {TERMINAL_TABS.map((tab) => {
            const isActive = tab.id === activeTabId;
            return (
              <button
                key={tab.id}
                role="tab"
                aria-selected={isActive}
                aria-controls={`terminal-tabpanel-${tab.id}`}
                onClick={() => setActiveTabId(tab.id)}
                style={{
                  background: isActive ? 'rgba(255, 255, 255, 0.15)' : 'transparent',
                  border: isActive ? '1px solid rgba(255, 255, 255, 0.2)' : '1px solid transparent',
                  borderRadius: '0.4rem',
                  padding: '0.25rem 0.6rem',
                  fontFamily: 'var(--font-mono)',
                  fontSize: '0.72rem',
                  color: isActive ? '#ffffff' : '#71717a',
                  cursor: 'pointer',
                  fontWeight: isActive ? 700 : 500,
                  whiteSpace: 'nowrap',
                  transition: 'all 0.15s ease',
                }}
              >
                {tab.label}
              </button>
            );
          })}
        </div>
      </div>

      {/* Terminal Viewport */}
      <div
        id={`terminal-tabpanel-${activeTab.id}`}
        role="tabpanel"
        className="terminal-body"
      >
        {/* Command Line & Copy Action */}
        <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', marginBottom: '1rem', borderBottom: '1px solid rgba(255, 255, 255, 0.06)', paddingBottom: '0.6rem' }}>
          <div style={{ display: 'flex', alignItems: 'center', gap: '0.5rem' }}>
            <span style={{ color: '#71717a', fontWeight: 700 }}>$</span>
            <span style={{ color: '#ffffff', fontWeight: 800 }}>{activeTab.command}</span>
          </div>

          <button
            onClick={() => copy(activeTab.command)}
            style={{
              background: 'transparent',
              border: 'none',
              color: copied ? '#ffffff' : '#71717a',
              cursor: 'pointer',
              display: 'flex',
              alignItems: 'center',
              gap: '0.35rem',
              fontSize: '0.72rem',
              fontFamily: 'var(--font-mono)',
            }}
            aria-label={`Copy command: ${activeTab.command}`}
          >
            {copied ? '✓ Copied' : 'Copy command'}
          </button>
        </div>

        {/* Output lines */}
        <div style={{ display: 'flex', flexDirection: 'column', gap: '0.2rem' }}>
          {activeTab.output.map((line, i) => {
            const isSuccess = line.startsWith('✓') || line.includes('Overall Health: 87/100');
            const isHeader = line.startsWith('┌') || line.startsWith('└') || line.startsWith('Lineage') || line.startsWith('Hotspots');
            const isDim = line.startsWith('  commits') || line.startsWith('  branches') || line.startsWith('  contributors');

            return (
              <div
                key={i}
                style={{
                  color: isSuccess ? '#ffffff' : isHeader ? '#e4e4e7' : isDim ? '#a1a1aa' : '#d4d4d8',
                  fontWeight: isSuccess || isHeader ? 700 : 400,
                  whiteSpace: 'pre',
                }}
              >
                {line}
              </div>
            );
          })}
        </div>

        {/* Live Active Blinking Cursor */}
        {!activeTab.isTui && (
          <div style={{ marginTop: '0.85rem', color: '#71717a', display: 'flex', alignItems: 'center', gap: '0.35rem' }}>
            <span>$</span>
            <span style={{ animation: 'vg-caret-blink 1s infinite', color: '#ffffff', fontWeight: 900 }}>▌</span>
          </div>
        )}
      </div>
    </div>
  );
}
