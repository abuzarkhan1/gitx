'use client';

import React, { useState } from 'react';

const ISSUES_BASE_URL = 'https://github.com/abuzarkhan1/gitx/issues/new';

type Category = 'bug' | 'feature' | 'perf' | 'docs';

interface CategoryOption {
  id: Category;
  label: string;
  badge: string;
  template: (title: string, details: string) => string;
}

const CATEGORIES: CategoryOption[] = [
  {
    id: 'bug',
    label: 'Bug Report',
    badge: 'bug',
    template: (t, d) =>
      `### Problem Description\n${d || 'N/A'}\n\n### Expected Behavior\n...\n\n### Environment\n- GitX Version: v0.1.0\n- OS: macOS / Linux / Windows\n\n---\n_Reported from GitX Website_`,
  },
  {
    id: 'feature',
    label: 'Feature Request',
    badge: 'enhancement',
    template: (t, d) =>
      `### Proposed Feature\n${t}\n\n### Use Case & Context\n${d || 'N/A'}\n\n---\n_Submitted from GitX Website_`,
  },
  {
    id: 'perf',
    label: 'Performance',
    badge: 'performance',
    template: (t, d) =>
      `### Performance Bottleneck\n${t}\n\n### Repository Size & Benchmark Details\n${d || 'N/A'}\n\n---\n_Submitted from GitX Website_`,
  },
  {
    id: 'docs',
    label: 'Documentation',
    badge: 'documentation',
    template: (t, d) =>
      `### Documentation Feedback\n${t}\n\n### Details\n${d || 'N/A'}\n\n---\n_Submitted from GitX Website_`,
  },
];

export default function IssueForm() {
  const [category, setCategory] = useState<Category>('bug');
  const [title, setTitle] = useState('');
  const [body, setBody] = useState('');
  const [error, setError] = useState<string | null>(null);
  const [submittedUrl, setSubmittedUrl] = useState<string | null>(null);

  function handleSubmit(event: React.FormEvent) {
    event.preventDefault();

    if (!title.trim()) {
      setError('Please enter a brief issue title.');
      return;
    }
    if (!body.trim()) {
      setError('Please provide description details for your issue.');
      return;
    }

    setError(null);

    const catConfig = CATEGORIES.find((c) => c.id === category) || CATEGORIES[0];
    const generatedBody = catConfig.template(title.trim(), body.trim());

    const params = new URLSearchParams();
    params.set('title', `[${catConfig.label}] ${title.trim()}`);
    params.set('body', generatedBody);
    params.set('labels', catConfig.badge);

    const fullUrl = `${ISSUES_BASE_URL}?${params.toString()}`;
    setSubmittedUrl(fullUrl);

    const win = window.open(fullUrl, '_blank', 'noopener,noreferrer');
    if (!win) {
      console.warn('Popup blocked, fallback button presented.');
    }
  }

  function handleReset() {
    setTitle('');
    setBody('');
    setError(null);
    setSubmittedUrl(null);
  }

  return (
    <form onSubmit={handleSubmit} style={{ display: 'flex', flexDirection: 'column', gap: '1.25rem' }}>
      {/* Issue Category Pill Selector */}
      <div>
        <label style={{ display: 'block', fontFamily: 'var(--font-mono)', fontSize: '0.75rem', fontWeight: 700, textTransform: 'uppercase', letterSpacing: '0.1em', color: '#a1a1aa', marginBottom: '0.5rem' }}>
          Issue Category
        </label>
        <div style={{ display: 'flex', gap: '0.5rem', flexWrap: 'wrap' }}>
          {CATEGORIES.map((cat) => {
            const isSelected = category === cat.id;
            return (
              <button
                key={cat.id}
                type="button"
                onClick={() => setCategory(cat.id)}
                style={{
                  background: isSelected ? '#ffffff' : 'rgba(255, 255, 255, 0.04)',
                  border: isSelected ? '1px solid #ffffff' : '1px solid rgba(255, 255, 255, 0.1)',
                  color: isSelected ? '#000000' : '#d4d4d8',
                  borderRadius: '9999px',
                  padding: '0.35rem 0.85rem',
                  fontSize: '0.78rem',
                  fontWeight: 700,
                  cursor: 'pointer',
                  fontFamily: 'var(--font-space)',
                  transition: 'all 0.15s ease',
                }}
              >
                {cat.label}
              </button>
            );
          })}
        </div>
      </div>

      {/* Title */}
      <div>
        <label
          htmlFor="issue-title"
          style={{ display: 'block', fontFamily: 'var(--font-mono)', fontSize: '0.75rem', fontWeight: 700, textTransform: 'uppercase', letterSpacing: '0.1em', color: '#a1a1aa', marginBottom: '0.4rem' }}
        >
          Issue Title <span style={{ color: '#ffffff' }}>*</span>
        </label>
        <input
          id="issue-title"
          type="text"
          value={title}
          onChange={(e) => {
            setTitle(e.target.value);
            if (error) setError(null);
          }}
          placeholder="e.g. feat: add Maven pom.xml manifest parser"
          aria-invalid={!!error && !title.trim()}
          style={{
            width: '100%',
            background: 'rgba(0, 0, 0, 0.5)',
            border: error && !title.trim() ? '1px solid #ef4444' : '1px solid rgba(255, 255, 255, 0.1)',
            borderRadius: '0.6rem',
            padding: '0.75rem 1rem',
            color: '#ffffff',
            fontSize: '0.9rem',
            fontFamily: 'var(--font-space)',
            outline: 'none',
            transition: 'border-color 0.2s ease, box-shadow 0.2s ease',
          }}
          onFocus={(e) => {
            e.currentTarget.style.borderColor = 'rgba(255, 255, 255, 0.4)';
            e.currentTarget.style.boxShadow = '0 0 12px rgba(255, 255, 255, 0.15)';
          }}
          onBlur={(e) => {
            e.currentTarget.style.borderColor = error && !title.trim() ? '#ef4444' : 'rgba(255, 255, 255, 0.1)';
            e.currentTarget.style.boxShadow = 'none';
          }}
        />
      </div>

      {/* Details */}
      <div>
        <label
          htmlFor="issue-body"
          style={{ display: 'block', fontFamily: 'var(--font-mono)', fontSize: '0.75rem', fontWeight: 700, textTransform: 'uppercase', letterSpacing: '0.1em', color: '#a1a1aa', marginBottom: '0.4rem' }}
        >
          Details / Reproduction Steps <span style={{ color: '#ffffff' }}>*</span>
        </label>
        <textarea
          id="issue-body"
          rows={4}
          value={body}
          onChange={(e) => {
            setBody(e.target.value);
            if (error) setError(null);
          }}
          placeholder="Describe your suggestion, steps to reproduce, or performance details..."
          aria-invalid={!!error && !body.trim()}
          style={{
            width: '100%',
            background: 'rgba(0, 0, 0, 0.5)',
            border: error && !body.trim() ? '1px solid #ef4444' : '1px solid rgba(255, 255, 255, 0.1)',
            borderRadius: '0.6rem',
            padding: '0.75rem 1rem',
            color: '#ffffff',
            fontSize: '0.9rem',
            fontFamily: 'var(--font-space)',
            outline: 'none',
            resize: 'vertical',
            transition: 'border-color 0.2s ease, box-shadow 0.2s ease',
          }}
          onFocus={(e) => {
            e.currentTarget.style.borderColor = 'rgba(255, 255, 255, 0.4)';
            e.currentTarget.style.boxShadow = '0 0 12px rgba(255, 255, 255, 0.15)';
          }}
          onBlur={(e) => {
            e.currentTarget.style.borderColor = error && !body.trim() ? '#ef4444' : 'rgba(255, 255, 255, 0.1)';
            e.currentTarget.style.boxShadow = 'none';
          }}
        />
      </div>

      {error && (
        <div style={{ color: '#ef4444', fontSize: '0.8rem', fontFamily: 'var(--font-mono)' }}>
          ⚠ {error}
        </div>
      )}

      <p style={{ fontFamily: 'var(--font-mono)', fontSize: '0.75rem', color: '#71717a' }}>
        Submits directly as a pre-filled GitHub issue — 100% offline preparation, zero tracking.
      </p>

      <button type="submit" className="btn-primary" style={{ width: '100%' }}>
        Open Issue on GitHub →
      </button>

      {submittedUrl && (
        <div
          style={{
            padding: '0.85rem 1rem',
            background: 'rgba(255, 255, 255, 0.04)',
            borderRadius: '0.6rem',
            border: '1px solid rgba(255, 255, 255, 0.15)',
            fontFamily: 'var(--font-mono)',
            fontSize: '0.78rem',
            color: '#a1a1aa',
            display: 'flex',
            flexDirection: 'column',
            gap: '0.5rem',
          }}
        >
          <div>✓ Pre-filled issue generated.</div>
          <a
            href={submittedUrl}
            target="_blank"
            rel="noreferrer"
            style={{ color: '#ffffff', textDecoration: 'underline', fontWeight: 700 }}
          >
            Click here if GitHub didn&apos;t open automatically →
          </a>
          <button
            type="button"
            onClick={handleReset}
            style={{
              background: 'transparent',
              border: 'none',
              color: '#71717a',
              textAlign: 'left',
              cursor: 'pointer',
              fontSize: '0.72rem',
              padding: 0,
            }}
          >
            Clear and submit another issue
          </button>
        </div>
      )}
    </form>
  );
}
