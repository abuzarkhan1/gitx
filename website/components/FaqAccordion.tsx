'use client';

import React, { useState } from 'react';

interface FaqItem {
  q: string;
  a: string;
}

const FAQS: FaqItem[] = [
  {
    q: 'Is GitX really 100% free, open-source, and offline?',
    a: 'Yes. GitX is released under the permissive MIT License across all 11 crates. There are zero subscriptions, zero cloud dependencies, zero telemetry, and zero network calls. Your codebase and history never leave your machine.',
  },
  {
    q: 'How is GitX so much faster than traditional Git tools?',
    a: 'GitX is written in pure Rust using gix and rayon for parallel packfile parsing. It indexes repository commit graphs and symbol tables into a local SQLite database with FTS5 virtual tables, enabling sub-millisecond archaeology and BM25 full-text queries.',
  },
  {
    q: 'How does File Lineage differ from standard git blame?',
    a: 'Standard git blame stops or breaks when files are renamed, moved into submodules, or refactored across directory hierarchies. GitX lineage computes continuous identity hashes along the commit DAG to follow file lifecycles across years and structural renames.',
  },
  {
    q: 'What is the 6-Score Repository Health formula?',
    a: 'GitX calculates a deterministic 0–100 score across 6 weighted sub-dimensions: Code Hotspots (25%), Ownership/Bus-Factor (20%), Branch Hygiene (15%), Change Volatility (15%), Architecture Stability (15%), and Recovery Risk (10%). All metrics are reproducible and can be emitted as JSON for CI/CD gates.',
  },
  {
    q: 'How does Disaster Recovery resurrect lost commits?',
    a: 'GitX scans the local object database and reflogs for dangling, detached, or orphaned commit objects created before botched rebases or resets. You can inspect unreachable commits and export unified patches with `gitx recovery export <oid>`.',
  },
  {
    q: 'Which platforms and installation methods are supported?',
    a: 'GitX compiles natively on macOS (Apple Silicon & Intel), Linux, and Windows via standard Rust tooling. You can build from source using `git clone https://github.com/abuzarkhan1/gitx.git && cd gitx && cargo build --release` or install directly via `cargo install --git https://github.com/abuzarkhan1/gitx.git gitx-cli`. Source code and release tags are hosted openly on GitHub.',
  },
];

export function FaqAccordion() {
  const [openIndex, setOpenIndex] = useState<number | null>(null);

  const toggle = (index: number) => {
    setOpenIndex((prev) => (prev === index ? null : index));
  };

  return (
    <div style={{ display: 'flex', flexDirection: 'column', borderTop: '1px solid rgba(255, 255, 255, 0.08)' }}>
      {FAQS.map((faq, i) => {
        const isOpen = openIndex === i;
        return (
          <div
            key={i}
            style={{
              borderBottom: '1px solid rgba(255, 255, 255, 0.08)',
              transition: 'background 0.2s ease',
            }}
          >
            <button
              onClick={() => toggle(i)}
              aria-expanded={isOpen}
              aria-controls={`faq-answer-${i}`}
              id={`faq-question-${i}`}
              style={{
                width: '100%',
                padding: '1.5rem 0',
                background: 'transparent',
                border: 'none',
                color: '#ffffff',
                textAlign: 'left',
                display: 'flex',
                alignItems: 'center',
                justifyContent: 'space-between',
                cursor: 'pointer',
                fontSize: '1.1rem',
                fontWeight: 800,
                letterSpacing: '-0.02em',
                fontFamily: 'var(--font-space)',
              }}
            >
              <span>{faq.q}</span>
              <span
                style={{
                  fontSize: '1.4rem',
                  color: isOpen ? '#ffffff' : '#71717a',
                  transition: 'transform 0.25s cubic-bezier(0.16, 1, 0.3, 1)',
                  transform: isOpen ? 'rotate(45deg)' : 'rotate(0deg)',
                  display: 'inline-block',
                  lineHeight: 1,
                  userSelect: 'none',
                  marginLeft: '1rem',
                  flexShrink: 0,
                }}
                aria-hidden="true"
              >
                +
              </span>
            </button>

            {/* Zero Layout-Shift CSS Grid Container */}
            <div
              id={`faq-answer-${i}`}
              role="region"
              aria-labelledby={`faq-question-${i}`}
              style={{
                display: 'grid',
                gridTemplateRows: isOpen ? '1fr' : '0fr',
                transition: 'grid-template-rows 0.3s cubic-bezier(0.16, 1, 0.3, 1), opacity 0.2s ease',
                opacity: isOpen ? 1 : 0,
              }}
            >
              <div style={{ overflow: 'hidden' }}>
                <p style={{ paddingBottom: '1.5rem', color: '#a1a1aa', fontSize: '0.95rem', lineHeight: 1.65 }}>
                  {faq.a}
                </p>
              </div>
            </div>
          </div>
        );
      })}
    </div>
  );
}
