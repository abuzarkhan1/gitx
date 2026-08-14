import type { Metadata } from 'next';
import { Terminal, ExternalLink } from 'lucide-react';
import IssueForm from '@/components/IssueForm';

export const metadata: Metadata = {
  title: 'Contact & Community — GitX',
  description: 'Submit issue reports, feature requests, or explore the 22 engineering architecture specifications of GitX.',
};

function GitHubIcon({ size = 18 }: { size?: number }) {
  return (
    <svg width={size} height={size} viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" aria-hidden="true">
      <path d="M15 22v-4a4.8 4.8 0 0 0-1-3.5c3 0 6-2 6-5.5.08-1.25-.27-2.48-1-3.5.28-1.15.28-2.35 0-3.5 0 0-1 0-3 1.5-2.64-.5-5.36-.5-8 0C6 2 5 2 5 2c-.3 1.15-.3 2.35 0 3.5A5.403 5.403 0 0 0 4 9c0 3.5 3 5.5 6 5.5-.39.49-.68 1.05-.85 1.65-.17.6-.22 1.23-.15 1.85v4" />
      <path d="M9 18c-4.51 2-5-2-7-2" />
    </svg>
  );
}

export default function ContactPage() {
  return (
    <div style={{ background: '#08080a', minHeight: '100vh', paddingTop: '8.5rem', paddingBottom: '6rem', color: '#ffffff' }}>
      <div className="container" style={{ maxWidth: '960px' }}>
        {/* Header */}
        <div style={{ textAlign: 'center', marginBottom: '4rem' }}>
          <span className="section-label">Community &amp; Support</span>
          <h1 className="vg-hero-heading" style={{ fontSize: 'clamp(2.25rem, 5vw, 3.5rem)', color: '#ffffff', marginBottom: '1.25rem' }}>
            Get in <span className="vg-serif" style={{ color: '#ffffff', fontWeight: 400 }}>Touch</span>
          </h1>
          <p style={{ color: '#a1a1aa', fontSize: '1.1rem', lineHeight: 1.6, maxWidth: '600px', margin: '0 auto' }}>
            Have questions, bug reports, or feature proposals for GitX? Submit feedback or explore our open-source codebase.
          </p>
        </div>

        <div className="grid-2" style={{ gap: '2rem' }}>
          {/* Direct Channels */}
          <div className="bento-card" style={{ padding: '2.25rem' }}>
            <div className="shine-layer" />
            <h2 style={{ fontSize: '1.25rem', fontWeight: 800, color: '#ffffff', marginBottom: '1.25rem' }}>
              Official Channels
            </h2>
            <div style={{ display: 'flex', flexDirection: 'column', gap: '1rem' }}>
              <a
                href="https://github.com/abuzarkhan1/gitx/issues"
                target="_blank"
                rel="noopener noreferrer"
                style={{
                  display: 'flex',
                  alignItems: 'center',
                  justifyContent: 'space-between',
                  padding: '1.1rem',
                  background: 'rgba(255, 255, 255, 0.02)',
                  border: '1px solid rgba(255, 255, 255, 0.06)',
                  borderRadius: '0.75rem',
                  transition: 'border-color 0.2s ease',
                }}
                className="hover-border-white"
              >
                <div style={{ display: 'flex', alignItems: 'center', gap: '0.75rem' }}>
                  <GitHubIcon size={18} />
                  <div>
                    <div style={{ fontWeight: 700, color: '#ffffff', fontSize: '0.9rem' }}>GitHub Issues</div>
                    <div style={{ fontSize: '0.8rem', color: '#71717a' }}>Bug reports and feature requests</div>
                  </div>
                </div>
                <ExternalLink size={14} style={{ color: '#71717a' }} />
              </a>

              <a
                href="https://github.com/abuzarkhan1/gitx/tree/main/docs"
                target="_blank"
                rel="noopener noreferrer"
                style={{
                  display: 'flex',
                  alignItems: 'center',
                  justifyContent: 'space-between',
                  padding: '1.1rem',
                  background: 'rgba(255, 255, 255, 0.02)',
                  border: '1px solid rgba(255, 255, 255, 0.06)',
                  borderRadius: '0.75rem',
                  transition: 'border-color 0.2s ease',
                }}
                className="hover-border-white"
              >
                <div style={{ display: 'flex', alignItems: 'center', gap: '0.75rem' }}>
                  <Terminal size={18} style={{ color: '#ffffff' }} />
                  <div>
                    <div style={{ fontWeight: 700, color: '#ffffff', fontSize: '0.9rem' }}>Documentation</div>
                    <div style={{ fontSize: '0.8rem', color: '#71717a' }}>22 engineering specs and architecture docs</div>
                  </div>
                </div>
                <ExternalLink size={14} style={{ color: '#71717a' }} />
              </a>
            </div>
          </div>

          {/* Feedback Form */}
          <div className="bento-card" style={{ padding: '2.25rem' }}>
            <div className="shine-layer" />
            <h2 style={{ fontSize: '1.25rem', fontWeight: 800, color: '#ffffff', marginBottom: '1.25rem' }}>
              Submit an Issue / Feedback
            </h2>
            <IssueForm />
          </div>
        </div>
      </div>
    </div>
  );
}
