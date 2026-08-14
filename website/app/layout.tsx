import type { Metadata } from 'next';
import { Navbar } from '@/components/Navbar';
import { Footer } from '@/components/Footer';
import { ScrollControls } from '@/components/ScrollControls';
import './globals.css';

export const metadata: Metadata = {
  title: {
    default: 'GitX — Terminal-Native Git Repository Intelligence & Code Archaeology',
    template: '%s — GitX',
  },
  description:
    'High-performance, local-first repository intelligence and code archaeology in pure Rust. SQLite FTS5 search, 6-score health metrics, rename-tracking lineage, and 60 FPS Ratatui TUI dashboard. 100% offline, zero telemetry.',
  keywords: [
    'gitx',
    'git archaeology',
    'rust git tool',
    'terminal ui',
    'ratatui',
    'git health score',
    'sqlite fts5',
    'repository intelligence',
    'code hotspots',
    'local-first',
  ],
  authors: [{ name: 'Abuzar Khan', url: 'https://github.com/abuzarkhan1' }],
  metadataBase: new URL('https://gitx.sh'),
  openGraph: {
    title: 'GitX — Terminal-Native Git Repository Intelligence',
    description:
      'Pure Rust, local-first Git repository intelligence, continuous rename lineage, deterministic health scoring, and 60 FPS terminal dashboard.',
    type: 'website',
    url: 'https://gitx.sh',
    siteName: 'GitX',
  },
  twitter: {
    card: 'summary_large_image',
    title: 'GitX — Terminal-Native Git Repository Intelligence',
    description:
      'Pure Rust, local-first Git repository intelligence, continuous rename lineage, deterministic health scoring, and 60 FPS terminal dashboard.',
  },
};

export default function RootLayout({
  children,
}: Readonly<{ children: React.ReactNode }>) {
  return (
    <html lang="en" className="dark">
      <head>
        <link rel="preconnect" href="https://fonts.googleapis.com" />
        <link rel="preconnect" href="https://fonts.gstatic.com" crossOrigin="anonymous" />
      </head>
      <body style={{ minHeight: '100vh', display: 'flex', flexDirection: 'column', background: '#08080a', color: '#ffffff' }}>
        <Navbar />
        <main id="main-content" style={{ flex: 1, outline: 'none' }}>
          {children}
        </main>
        <Footer />
        <ScrollControls />
      </body>
    </html>
  );
}
