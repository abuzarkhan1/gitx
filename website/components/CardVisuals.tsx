'use client';

import React from 'react';

export function RustPulseRing() {
  return (
    <div style={{ position: 'relative', width: '100%', height: '140px', display: 'flex', alignItems: 'center', justifyContent: 'center' }}>
      <div
        style={{
          position: 'absolute',
          width: '110px',
          height: '110px',
          borderRadius: '50%',
          border: '1px solid rgba(255, 255, 255, 0.1)',
          animation: 'ping 3s cubic-bezier(0, 0, 0.2, 1) infinite',
        }}
      />
      <div
        style={{
          position: 'absolute',
          width: '80px',
          height: '80px',
          borderRadius: '50%',
          border: '1px solid rgba(255, 255, 255, 0.2)',
        }}
      />
      <div
        style={{
          position: 'relative',
          width: '54px',
          height: '54px',
          borderRadius: '50%',
          background: 'rgba(255, 255, 255, 0.08)',
          border: '1px solid rgba(255, 255, 255, 0.3)',
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'center',
          color: '#ffffff',
          fontWeight: 900,
          fontFamily: 'var(--font-mono)',
          fontSize: '0.85rem',
          boxShadow: '0 0 20px rgba(255, 255, 255, 0.15)',
        }}
      >
        🦀 11
      </div>
      <div
        style={{
          position: 'absolute',
          bottom: '8px',
          fontFamily: 'var(--font-mono)',
          fontSize: '0.72rem',
          color: '#a1a1aa',
          letterSpacing: '0.05em',
        }}
      >
        PARALLEL OBJECT PACKFILE PARSER
      </div>
    </div>
  );
}

export function LineageForensicsVisual() {
  const commits = [
    { sha: 'a8f2c19', date: '2026-03-12', action: 'modified', path: 'crates/gitx-core/src/index.rs' },
    { sha: '3b41e90', date: '2026-01-04', action: 'renamed', path: 'crates/gitx-index/src/index.rs' },
    { sha: 'f104d2e', date: '2025-11-19', action: 'modified', path: 'crates/gitx-storage/src/fts.rs' },
  ];

  return (
    <div style={{ background: 'rgba(0, 0, 0, 0.4)', borderRadius: '0.75rem', padding: '1rem', border: '1px solid rgba(255, 255, 255, 0.06)' }}>
      <div style={{ display: 'flex', justifyContent: 'space-between', marginBottom: '0.65rem', fontFamily: 'var(--font-mono)', fontSize: '0.72rem', color: '#a1a1aa' }}>
        <span>FILE LINEAGE DAG</span>
        <span style={{ color: '#ffffff', fontWeight: 700 }}>100% Rename Continuity</span>
      </div>
      <div style={{ display: 'flex', flexDirection: 'column', gap: '0.45rem' }}>
        {commits.map((c, i) => (
          <div key={i} style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', fontFamily: 'var(--font-mono)', fontSize: '0.75rem', color: '#d4d4d8' }}>
            <div style={{ display: 'flex', alignItems: 'center', gap: '0.5rem' }}>
              <span style={{ color: '#ffffff', fontWeight: 700 }}>{c.sha}</span>
              <span style={{ color: c.action === 'renamed' ? '#ffffff' : '#71717a', fontSize: '0.7rem' }}>
                {c.action}
              </span>
            </div>
            <span style={{ color: '#a1a1aa', fontSize: '0.7rem', overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap', maxWidth: '170px' }}>
              {c.path}
            </span>
          </div>
        ))}
      </div>
    </div>
  );
}

export function HealthScorecardVisual() {
  const scores = [
    { name: 'Commit hygiene', score: 94 },
    { name: 'Branch hygiene', score: 88 },
    { name: 'Hotspot risk', score: 82 },
    { name: 'Ownership balance', score: 76 },
  ];

  return (
    <div style={{ background: 'rgba(0, 0, 0, 0.4)', borderRadius: '0.75rem', padding: '1rem', border: '1px solid rgba(255, 255, 255, 0.06)' }}>
      <div style={{ display: 'flex', justifyContent: 'space-between', marginBottom: '0.65rem', fontFamily: 'var(--font-mono)', fontSize: '0.72rem', color: '#a1a1aa' }}>
        <span>6-SCORE HEALTH ENGINE</span>
        <span style={{ color: '#ffffff', fontWeight: 700 }}>Overall: 87/100 (Grade A)</span>
      </div>
      <div style={{ display: 'flex', flexDirection: 'column', gap: '0.4rem' }}>
        {scores.map((s, i) => (
          <div key={i} style={{ display: 'flex', alignItems: 'center', gap: '0.6rem' }}>
            <span style={{ width: '120px', fontFamily: 'var(--font-mono)', fontSize: '0.72rem', color: '#a1a1aa', flexShrink: 0 }}>
              {s.name}
            </span>
            <div style={{ flex: 1, height: '6px', background: 'rgba(255, 255, 255, 0.08)', borderRadius: '3px', overflow: 'hidden' }}>
              <div style={{ width: `${s.score}%`, height: '100%', background: '#ffffff', borderRadius: '3px' }} />
            </div>
            <span style={{ width: '28px', fontFamily: 'var(--font-mono)', fontSize: '0.72rem', color: '#ffffff', fontWeight: 700, textAlign: 'right' }}>
              {s.score}
            </span>
          </div>
        ))}
      </div>
    </div>
  );
}

export function SqliteFts5Visual() {
  return (
    <div style={{ background: 'rgba(0, 0, 0, 0.4)', borderRadius: '0.75rem', padding: '1rem', border: '1px solid rgba(255, 255, 255, 0.06)' }}>
      <div style={{ display: 'flex', justifyContent: 'space-between', marginBottom: '0.65rem', fontFamily: 'var(--font-mono)', fontSize: '0.72rem', color: '#a1a1aa' }}>
        <span>SQLITE FTS5 SEARCH</span>
        <span style={{ color: '#ffffff', fontWeight: 700 }}>Query latency: 380µs</span>
      </div>
      <div style={{ display: 'flex', alignItems: 'center', gap: '0.5rem', background: 'rgba(255, 255, 255, 0.04)', padding: '0.4rem 0.6rem', borderRadius: '0.4rem', fontFamily: 'var(--font-mono)', fontSize: '0.75rem', color: '#ffffff', marginBottom: '0.5rem' }}>
        <span style={{ color: '#71717a' }}>$</span>
        <span>gitx search &quot;sqlite AND token&quot;</span>
      </div>
      <div style={{ display: 'flex', justifyContent: 'space-between', fontFamily: 'var(--font-mono)', fontSize: '0.7rem', color: '#a1a1aa' }}>
        <span>3 commits indexed in BM25</span>
        <span style={{ color: '#ffffff' }}>0.00s cold start</span>
      </div>
    </div>
  );
}

export function DisasterRecoveryVisual() {
  return (
    <div style={{ background: 'rgba(0, 0, 0, 0.4)', borderRadius: '0.75rem', padding: '1rem', border: '1px solid rgba(255, 255, 255, 0.06)' }}>
      <div style={{ display: 'flex', justifyContent: 'space-between', marginBottom: '0.65rem', fontFamily: 'var(--font-mono)', fontSize: '0.72rem', color: '#a1a1aa' }}>
        <span>DISASTER RECOVERY</span>
        <span style={{ color: '#ffffff', fontWeight: 700 }}>Read-Only Reflog Forensics</span>
      </div>
      <div style={{ display: 'flex', flexDirection: 'column', gap: '0.35rem', fontFamily: 'var(--font-mono)', fontSize: '0.73rem', color: '#d4d4d8' }}>
        <div style={{ display: 'flex', justifyContent: 'space-between' }}>
          <span style={{ color: '#ffffff' }}>● 4d7a2c8 (dangling commit)</span>
          <span style={{ color: '#a1a1aa' }}>2h ago</span>
        </div>
        <div style={{ color: '#71717a', fontSize: '0.7rem' }}>
          &quot;wip: experimental streaming graph parser&quot;
        </div>
        <div style={{ marginTop: '0.2rem', color: '#ffffff', fontSize: '0.7rem' }}>
          → Export unified patch: gitx recovery export 4d7a2c8
        </div>
      </div>
    </div>
  );
}

export function AirGapPrivacyVisual() {
  return (
    <div style={{ background: 'rgba(0, 0, 0, 0.4)', borderRadius: '0.75rem', padding: '1rem', border: '1px solid rgba(255, 255, 255, 0.06)' }}>
      <div style={{ display: 'flex', justifyContent: 'space-between', marginBottom: '0.65rem', fontFamily: 'var(--font-mono)', fontSize: '0.72rem', color: '#a1a1aa' }}>
        <span>AIR-GAPPED PRIVACY</span>
        <span style={{ color: '#ffffff', fontWeight: 700 }}>100% Offline</span>
      </div>
      <div style={{ display: 'grid', gridTemplateColumns: 'repeat(2, 1fr)', gap: '0.5rem', fontFamily: 'var(--font-mono)', fontSize: '0.72rem' }}>
        <div style={{ background: 'rgba(255,255,255,0.03)', padding: '0.4rem', borderRadius: '4px', textAlign: 'center', color: '#ffffff' }}>
          0 Network Calls
        </div>
        <div style={{ background: 'rgba(255,255,255,0.03)', padding: '0.4rem', borderRadius: '4px', textAlign: 'center', color: '#ffffff' }}>
          0 Telemetry
        </div>
        <div style={{ background: 'rgba(255,255,255,0.03)', padding: '0.4rem', borderRadius: '4px', textAlign: 'center', color: '#ffffff' }}>
          0 Cloud Accounts
        </div>
        <div style={{ background: 'rgba(255,255,255,0.03)', padding: '0.4rem', borderRadius: '4px', textAlign: 'center', color: '#ffffff' }}>
          0 AI Token Fees
        </div>
      </div>
    </div>
  );
}
