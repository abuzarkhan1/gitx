"use client";

import React, { useState } from "react";
import Link from "next/link";
import { ArrowLeft, CheckCircle2, Copy, Check } from "lucide-react";
import { GithubIcon } from "@/components/ui/GithubIcon";
import { Navbar } from "@/components/ui/Navbar";
import { Footer } from "@/components/ui/Footer";
import { TextReveal } from "@/components/motion/TextReveal";

export default function AboutPage() {
  const [citationFormat, setCitationFormat] = useState<"bibtex" | "cargo" | "schema">("bibtex");
  const [copiedCitation, setCopiedCitation] = useState(false);

  const snippets = {
    bibtex: `@software{gitx2026archaeology,
  title={GitX: Local-First Terminal Repository Intelligence and Code Archaeology},
  author={Abuzar Khan and GitX Contributors},
  year={2026},
  url={https://github.com/abuzarkhan1/gitx},
}`,
    cargo: `[dependencies]
gitx-core = "0.1.0"
gitx-analysis = "0.1.0"
gitx-history = "0.1.0"
gitx-storage = "0.1.0"`,
    schema: `-- SQLite Cached Tables Schema (WAL Mode)
CREATE TABLE commits (hash TEXT PRIMARY KEY, author TEXT, timestamp INTEGER, message TEXT);
CREATE TABLE file_changes (commit_hash TEXT, file_path TEXT, additions INTEGER, deletions INTEGER);
CREATE TABLE rename_lineage (source_path TEXT, target_path TEXT, similarity_pct INTEGER);
CREATE TABLE recovery_dangling (hash TEXT PRIMARY KEY, discovered_at INTEGER, reason TEXT);`,
  };

  const handleCopyCitation = () => {
    navigator.clipboard.writeText(snippets[citationFormat]);
    setCopiedCitation(true);
    setTimeout(() => setCopiedCitation(false), 2000);
  };

  return (
    <div className="min-h-screen bg-[#ffffff] text-[#202020] flex flex-col">
      <Navbar />

      <main id="main-content" className="flex-1 pt-32 pb-24">
        <div className="section-container max-w-3xl space-y-12">
          <Link
            href="/"
            className="inline-flex items-center gap-2 text-xs font-mono text-[#828282] hover:text-[#202020] transition-colors"
          >
            <ArrowLeft size={13} />
            <span>Observatory</span>
          </Link>

          <div className="space-y-3">
            <TextReveal as="h1" className="font-heading text-4xl md:text-5xl text-[#202020] leading-tight tracking-[-0.02em]">
              The 11-Crate Rust Architecture
            </TextReveal>
            <p className="text-base text-[#4d4d4d] leading-relaxed">
              GitX is engineered as a clean 5-layer modular Rust workspace separating UI presentation, domain services, analytical algorithms, SQLite caching, and raw Git object storage.
            </p>
          </div>

          <div className="grid grid-cols-1 sm:grid-cols-2 gap-6 border-t border-[#e8e8e8] pt-8">
            <div className="p-6 border border-[#e8e8e8]">
              <div className="font-mono text-xs text-[#828282] mb-1">PRESENTATION LAYER</div>
              <h3 className="font-heading text-lg text-[#202020] mb-2">gitx-cli &amp; gitx-tui</h3>
              <p className="text-xs text-[#4d4d4d] leading-relaxed">
                Ratatui 0.28 terminal UI with 60fps keyboard navigation, mouse support, ANSI color fidelity, and Clap 4.5 CLI parser.
              </p>
            </div>

            <div className="p-6 border border-[#202020]">
              <div className="font-mono text-xs text-[#ff682c] mb-1">DOMAIN &amp; ENGINE LAYER</div>
              <h3 className="font-heading text-lg text-[#202020] mb-2">gitx-analysis &amp; gitx-history</h3>
              <p className="text-xs text-[#4d4d4d] leading-relaxed">
                Deterministic maintenance risk engine, rename-following lineage tracking, and Petgraph branch divergence analyzer.
              </p>
            </div>
          </div>

          <div className="space-y-4">
            <h3 className="font-heading text-2xl text-[#202020] tracking-[-0.02em]">Core Invariants</h3>
            <div className="grid grid-cols-1 sm:grid-cols-3 gap-4">
              <div className="p-4 border border-[#e8e8e8] space-y-1">
                <div className="font-heading text-sm text-[#202020]">Zero AI / 100% Local</div>
                <p className="text-xs text-[#4d4d4d]">Every metric exposes its mathematical formula and raw Git signals.</p>
              </div>

              <div className="p-4 border border-[#e8e8e8] space-y-1">
                <div className="font-heading text-sm text-[#202020]">Sub-15ms SQLite Cache</div>
                <p className="text-xs text-[#4d4d4d]">Incremental indexing with WAL mode eliminates repeated `git log` overhead.</p>
              </div>

              <div className="p-4 border border-[#e8e8e8] space-y-1">
                <div className="font-heading text-sm text-[#202020]">Stable JSON Contracts</div>
                <p className="text-xs text-[#4d4d4d]">Every major command emits machine-readable JSON for CI integration.</p>
              </div>
            </div>
          </div>

          <div className="space-y-4 border-t border-[#e8e8e8] pt-8">
            <div className="flex items-center justify-between">
              <span className="font-mono text-xs text-[#828282] uppercase">
                Technical Specifications &amp; Schema
              </span>
              <div className="flex items-center gap-1 bg-[#f5f5f5] p-1 border border-[#e8e8e8]">
                {(["bibtex", "cargo", "schema"] as const).map((fmt) => (
                  <button
                    key={fmt}
                    onClick={() => setCitationFormat(fmt)}
                    className={`px-2.5 py-0.5 text-xs font-mono uppercase ${
                      citationFormat === fmt ? "bg-[#202020] text-[#ffffff]" : "text-[#4d4d4d]"
                    }`}
                  >
                    {fmt}
                  </button>
                ))}
              </div>
            </div>

            <div className="bg-[#202020] p-5 border border-[#333333] space-y-2">
              <div className="flex justify-between items-center text-xs font-mono text-[#828282] pb-2 border-b border-[#333333]">
                <span>FORMAT: {citationFormat.toUpperCase()}</span>
                <button
                  onClick={handleCopyCitation}
                  className="flex items-center gap-1 text-[#ff682c] hover:text-[#ffffff]"
                >
                  {copiedCitation ? <Check size={12} /> : <Copy size={12} />}
                  <span>{copiedCitation ? "Copied" : "Copy"}</span>
                </button>
              </div>
              <pre className="font-mono text-xs text-[#ebe6dd] whitespace-pre-wrap leading-relaxed">
                {snippets[citationFormat]}
              </pre>
            </div>
          </div>

          <div className="p-6 border border-[#e8e8e8] flex flex-col sm:flex-row sm:items-center justify-between gap-4">
            <div>
              <div className="font-heading text-base text-[#202020]">GitX Project</div>
              <div className="text-xs text-[#828282]">Open Source Repository Intelligence</div>
            </div>

            <a
              href="https://github.com/abuzarkhan1/gitx"
              target="_blank"
              rel="noreferrer"
              className="btn-primary text-xs"
            >
              <GithubIcon size={13} className="mr-1.5" />
              <span>GitHub</span>
            </a>
          </div>
        </div>
      </main>

      <Footer />
    </div>
  );
}
