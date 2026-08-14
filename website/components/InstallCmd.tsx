'use client';

import React, { useState, useEffect, useRef, useCallback } from 'react';

export function useClipboard(timeout = 2000) {
  const [copied, setCopied] = useState(false);
  const timerRef = useRef<NodeJS.Timeout | null>(null);

  useEffect(() => {
    return () => {
      if (timerRef.current) clearTimeout(timerRef.current);
    };
  }, []);

  const copy = useCallback(async (text: string): Promise<boolean> => {
    let success = false;

    // Modern Navigator Clipboard API
    if (typeof navigator !== 'undefined' && navigator.clipboard && window.isSecureContext) {
      try {
        await navigator.clipboard.writeText(text);
        success = true;
      } catch (err) {
        console.warn('Navigator clipboard failed, attempting fallback', err);
      }
    }

    // Fallback for non-secure / restricted iframe contexts
    if (!success && typeof document !== 'undefined') {
      try {
        const textarea = document.createElement('textarea');
        textarea.value = text;
        textarea.style.position = 'fixed';
        textarea.style.left = '-999999px';
        textarea.style.top = '-999999px';
        textarea.setAttribute('readonly', '');
        document.body.appendChild(textarea);
        textarea.select();
        success = document.execCommand('copy');
        document.body.removeChild(textarea);
      } catch (err) {
        console.error('Fallback clipboard copy failed', err);
      }
    }

    if (success) {
      if (timerRef.current) clearTimeout(timerRef.current);
      setCopied(true);
      timerRef.current = setTimeout(() => setCopied(false), timeout);
    }
    return success;
  }, [timeout]);

  return { copied, copy };
}

export function InstallCmd({ cmd, label }: { cmd: string; label?: string }) {
  const { copied, copy } = useClipboard(2000);

  return (
    <div
      style={{
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'space-between',
        gap: '0.75rem',
        borderRadius: '0.85rem',
        border: copied ? '1px solid rgba(255, 255, 255, 0.4)' : '1px solid rgba(255, 255, 255, 0.08)',
        background: copied ? 'rgba(39, 39, 42, 0.8)' : 'rgba(24, 24, 27, 0.6)',
        padding: '0.75rem 1.1rem',
        fontFamily: 'var(--font-mono)',
        fontSize: '0.82rem',
        color: '#d4d4d8',
        transition: 'all 0.2s ease',
      }}
    >
      <div style={{ display: 'flex', alignItems: 'center', gap: '0.6rem', overflow: 'hidden' }}>
        <span style={{ color: '#a1a1aa', fontWeight: 700, userSelect: 'none' }}>$</span>
        <span style={{ overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap', color: '#ffffff', fontWeight: 600 }}>
          {cmd}
        </span>
      </div>

      <div style={{ display: 'flex', alignItems: 'center', gap: '0.5rem', flexShrink: 0 }}>
        {label && (
          <span style={{ fontSize: '0.7rem', color: '#71717a', textTransform: 'uppercase', letterSpacing: '0.05em' }}>
            {label}
          </span>
        )}
        <button
          onClick={() => copy(cmd)}
          style={{
            background: copied ? '#ffffff' : 'rgba(255, 255, 255, 0.08)',
            border: 'none',
            color: copied ? '#000000' : '#ffffff',
            borderRadius: '0.4rem',
            padding: '0.35rem 0.65rem',
            cursor: 'pointer',
            display: 'inline-flex',
            alignItems: 'center',
            gap: '0.35rem',
            fontSize: '0.75rem',
            fontWeight: 700,
            fontFamily: 'var(--font-space)',
            transition: 'all 0.15s ease',
          }}
          aria-label={copied ? `Copied: ${cmd}` : `Copy ${cmd} to clipboard`}
        >
          {copied ? (
            <>
              <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="3">
                <polyline points="20 6 9 17 4 12" />
              </svg>
              <span>Copied</span>
            </>
          ) : (
            <>
              <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                <rect x="9" y="9" width="13" height="13" rx="2" />
                <path d="M5 15H4a2 2 0 01-2-2V4a2 2 0 012-2h9a2 2 0 012 2v1" />
              </svg>
              <span>Copy</span>
            </>
          )}
        </button>
      </div>

      <span className="sr-only" aria-live="polite">
        {copied ? 'Command copied to clipboard' : ''}
      </span>
    </div>
  );
}
