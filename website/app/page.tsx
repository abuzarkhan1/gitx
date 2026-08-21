"use client";

import React, { useState } from "react";
import Link from "next/link";
import { ArrowRight, Terminal, Copy, Check, Zap, Shield, History, Database, Flame, RotateCcw } from "lucide-react";
import { motion, AnimatePresence } from "framer-motion";
import { Navbar } from "@/components/ui/Navbar";
import { Footer } from "@/components/ui/Footer";
import { MarqueeStrip } from "@/components/ui/MarqueeStrip";
import { FaqAccordion } from "@/components/ui/FaqAccordion";
import { DeveloperHub } from "@/components/ui/DeveloperHub";
import { GitxTerminalPlayground } from "@/components/gitx/GitxTerminalPlayground";
import { HotspotsRiskMatrix } from "@/components/gitx/HotspotsRiskMatrix";
import { LineageArchaeologyVisualizer } from "@/components/gitx/LineageArchaeologyVisualizer";
import { RecoveryStudio } from "@/components/gitx/RecoveryStudio";
import { BenchmarkObservatory } from "@/components/gitx/BenchmarkObservatory";
import { TextReveal } from "@/components/motion/TextReveal";
import { TactileButton } from "@/components/motion/TactileButton";
import { PillarCard } from "@/components/motion/PillarCard";
import { ObservatoryGridCanvas } from "@/components/motion/ObservatoryGridCanvas";
import { useCursor } from "@/components/providers/CursorProvider";

export default function HomePage() {
  const [copiedInstall, setCopiedInstall] = useState(false);
  const { setCursorVariant, resetCursor } = useCursor();

  const installCmd = "curl -fsSL https://gitx.dev/install.sh | sh";

  const handleCopyInstall = () => {
    navigator.clipboard.writeText(installCmd);
    setCopiedInstall(true);
    setTimeout(() => setCopiedInstall(false), 2000);
  };

  return (
    <div className="min-h-screen bg-[#ffffff] text-[#202020] flex flex-col selection:bg-[#ff682c] selection:text-white">
      <Navbar />

      <main id="main-content" className="flex-1">
        {/* =========================================================================
            HERO OBSERVATORY — Full-Screen with Creative Reactive Grid
            ========================================================================= */}
        <section className="min-h-screen flex flex-col justify-center pt-28 pb-12 md:pt-32 md:pb-16 bg-[#ffffff] relative overflow-hidden">
          {/* Subtle Creative Observatory Background Canvas */}
          <ObservatoryGridCanvas />

          <div className="section-container my-auto z-10">
            <div className="grid grid-cols-1 lg:grid-cols-12 gap-12 lg:gap-16 items-center">
              {/* Left Typographic Statement with Staggered Entry */}
              <motion.div
                initial={{ opacity: 0, x: -24 }}
                animate={{ opacity: 1, x: 0 }}
                transition={{ duration: 0.7, ease: [0.25, 1, 0.5, 1] }}
                className="lg:col-span-6 space-y-6"
              >
                <TextReveal
                  as="h1"
                  className="font-heading text-4xl sm:text-5xl lg:text-[60px] text-[#202020] tracking-[-0.03em] leading-[1.02]"
                >
                  Local-first Git archaeology.
                </TextReveal>

                <motion.p
                  initial={{ opacity: 0, y: 12 }}
                  animate={{ opacity: 1, y: 0 }}
                  transition={{ duration: 0.6, delay: 0.15, ease: "easeOut" }}
                  className="text-lg md:text-xl text-[#4d4d4d] leading-relaxed max-w-xl font-sans font-normal"
                >
                  GitX turns commit history, code churn, ownership concentration, and lost reflogs into an instant, explainable terminal experience.
                </motion.p>

                {/* Primary CTA Cluster & Instant Install Pill */}
                <motion.div
                  initial={{ opacity: 0, y: 12 }}
                  animate={{ opacity: 1, y: 0 }}
                  transition={{ duration: 0.6, delay: 0.25, ease: "easeOut" }}
                  className="pt-2 space-y-3"
                >
                  <div className="flex flex-wrap items-center gap-3">
                    <TactileButton
                      href="#install"
                      variant="primary"
                      icon={<ArrowRight size={14} />}
                    >
                      Install GitX
                    </TactileButton>

                    <TactileButton
                      href="#tui"
                      variant="ghost"
                    >
                      Interactive TUI
                    </TactileButton>

                    <a
                      href="https://github.com/abuzarkhan1/gitx"
                      target="_blank"
                      rel="noopener noreferrer"
                      className="link-orange text-sm font-sans font-medium ml-1"
                    >
                      GitHub Docs &rarr;
                    </a>
                  </div>

                  {/* 1-Click Terminal Quick Install Box with Ember Orange Accent */}
                  <motion.div
                    whileHover={{ scale: 1.01 }}
                    whileTap={{ scale: 0.98 }}
                    onClick={handleCopyInstall}
                    className="inline-flex items-center justify-between gap-3 px-3.5 py-2.5 bg-[#ff682c] text-white border border-[#ff682c] hover:bg-[#e0561f] cursor-pointer transition-all max-w-md group shadow-md"
                    style={{ borderRadius: "0px" }}
                    title="Click to copy install command"
                  >
                    <div className="flex items-center gap-2 font-mono text-xs text-white truncate">
                      <span className="text-white/70 select-none">$</span>
                      <span className="truncate font-semibold">{installCmd}</span>
                    </div>

                    <div className="flex items-center gap-1 text-[11px] font-mono text-white/90 group-hover:text-white flex-shrink-0 bg-black/20 px-2 py-0.5">
                      {copiedInstall ? <Check size={12} className="text-white" /> : <Copy size={12} />}
                      <span>{copiedInstall ? "Copied" : "Copy"}</span>
                    </div>
                  </motion.div>
                </motion.div>

                {/* 3 Key figures with Counting Animation */}
                <motion.div
                  initial={{ opacity: 0, y: 12 }}
                  animate={{ opacity: 1, y: 0 }}
                  transition={{ duration: 0.6, delay: 0.35, ease: "easeOut" }}
                  className="grid grid-cols-3 gap-6 pt-6 border-t border-[#e8e8e8]"
                >
                  <div>
                    <div className="font-heading text-3xl text-[#202020] tracking-tight">&lt;15 ms</div>
                    <div className="text-xs text-[#828282] font-mono mt-1">SQLite Hot Query</div>
                  </div>
                  <div>
                    <div className="font-heading text-3xl text-[#ff682c] tracking-tight">100%</div>
                    <div className="text-xs text-[#828282] font-mono mt-1">Local &amp; Offline</div>
                  </div>
                  <div>
                    <div className="font-heading text-3xl text-[#202020] tracking-tight">11 Crates</div>
                    <div className="text-xs text-[#828282] font-mono mt-1">Rust Workspace</div>
                  </div>
                </motion.div>
              </motion.div>

              {/* Right: Live Interactive Ratatui TUI */}
              <div className="lg:col-span-6" id="tui">
                <GitxTerminalPlayground />
              </div>
            </div>
          </div>
        </section>

        {/* =========================================================================
            PROFESSIONAL INFINITE MARQUEE STRIP (AFTER HERO)
            ========================================================================= */}
        <MarqueeStrip />

        {/* =========================================================================
            THREE PILLARS OF REPOSITORY INTELLIGENCE (WITH 3D TILT CARDS)
            ========================================================================= */}
        <section className="py-20 md:py-24 bg-[#f9f9f9]">
          <div className="section-container space-y-12">
            <div className="max-w-2xl">
              <h2 className="font-heading text-3xl md:text-4xl text-[#202020] tracking-[-0.02em]">
                Built for deep codebase archaeology.
              </h2>
              <p className="text-base text-[#4d4d4d] mt-2 leading-relaxed">
                Raw Git commands are slow for historical analytics. GUI tools are closed, heavy, and leak telemetry. GitX provides an explainable, local-first alternative.
              </p>
            </div>

            <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
              <PillarCard
                icon={<Database size={20} />}
                title="Sub-Millisecond SQLite Cache"
                description="Incrementally indexes commits into an embedded SQLite database with WAL mode. Re-runs take <15ms instead of minutes of shell parsing."
                meta="rusqlite · bundled 3.45"
                delay={0.05}
              />

              <PillarCard
                icon={<Flame size={20} />}
                title="Deterministic Risk Scores"
                description="Exposes the exact mathematical formula behind file maintenance risk. No opaque AI ratings or hidden weights."
                meta="5 raw signals · linear model"
                delay={0.15}
              />

              <PillarCard
                icon={<RotateCcw size={20} />}
                title="Lossless Reflog Recovery"
                description="Scans local Git object storage to recover commits lost during hard resets, aborted rebases, or detached HEAD deletions."
                meta="zero network · raw .git/objects"
                delay={0.25}
              />
            </div>
          </div>
        </section>

        {/* =========================================================================
            HOTSPOTS & RISK MATRIX
            ========================================================================= */}
        <section id="hotspots" className="py-20 md:py-24 bg-[#ffffff]">
          <div className="section-container space-y-8">
            <div className="max-w-2xl">
              <h2 className="font-heading text-3xl md:text-4xl text-[#202020] tracking-[-0.02em]">
                Explainable maintenance risk without black-box scores.
              </h2>
              <p className="text-base text-[#4d4d4d] mt-2 leading-relaxed">
                Rank files by churn velocity, modification frequency, bug-fix commits, ownership concentration, and AST complexity.
              </p>
            </div>

            <HotspotsRiskMatrix />
          </div>
        </section>

        {/* =========================================================================
            LINEAGE & ARCHAEOLOGY
            ========================================================================= */}
        <section id="lineage" className="py-20 md:py-24 border-t border-[#e8e8e8] bg-[#f9f9f9]">
          <div className="section-container space-y-8">
            <div className="max-w-2xl">
              <h2 className="font-heading text-3xl md:text-4xl text-[#202020] tracking-[-0.02em]">
                Rename-following lineage across multi-year history.
              </h2>
              <p className="text-base text-[#4d4d4d] mt-2 leading-relaxed">
                Track a file's entire lifetime through renames, modular crate extractions, and refactors with tree-diff similarity heuristics.
              </p>
            </div>

            <LineageArchaeologyVisualizer />
          </div>
        </section>

        {/* =========================================================================
            RECOVERY & REFLOG
            ========================================================================= */}
        <section id="recovery" className="py-20 md:py-24 border-t border-[#e8e8e8] bg-[#ffffff]">
          <div className="section-container space-y-8">
            <div className="max-w-2xl">
              <h2 className="font-heading text-3xl md:text-4xl text-[#202020] tracking-[-0.02em]">
                Rescue lost commits, rebases, and dangling trees.
              </h2>
              <p className="text-base text-[#4d4d4d] mt-2 leading-relaxed">
                Scan local Git object storage to recover work orphaned by hard resets, abandoned cherry-picks, or deleted branches.
              </p>
            </div>

            <RecoveryStudio />
          </div>
        </section>

        {/* =========================================================================
            BENCHMARKS — Solid Ember Orange Section
            ========================================================================= */}
        <section id="benchmarks" className="py-20 md:py-24 bg-[#ff682c] text-white border-t border-[#e0561f]">
          <div className="section-container space-y-8">
            <div className="max-w-2xl">
              <h2 className="font-heading text-3xl md:text-4xl text-white tracking-[-0.02em]">
                Criterion benchmarks against frontier tooling.
              </h2>
              <p className="text-base text-white/90 mt-2 leading-relaxed">
                Tested across 1.2M commits on the Linux kernel repository for index throughput, query latency, and memory footprint.
              </p>
            </div>

            <BenchmarkObservatory />
          </div>
        </section>

        {/* =========================================================================
            INSTALLATION & DEVELOPER HUB
            ========================================================================= */}
        <section id="install" className="py-20 md:py-24 border-t border-[#e8e8e8] bg-[#ffffff]">
          <div className="section-container space-y-8">
            <div className="max-w-2xl">
              <h2 className="font-heading text-3xl md:text-4xl text-[#202020] tracking-[-0.02em]">
                Install in seconds. Run anywhere.
              </h2>
              <p className="text-base text-[#4d4d4d] mt-2 leading-relaxed">
                Distributed as standalone single binaries via shell installer, Cargo, Homebrew, or PowerShell.
              </p>
            </div>

            <DeveloperHub />
          </div>
        </section>

        {/* =========================================================================
            FAQ
            ========================================================================= */}
        <section className="py-20 md:py-24 border-t border-[#e8e8e8] bg-[#f9f9f9]">
          <div className="section-container max-w-3xl space-y-8">
            <div>
              <h2 className="font-heading text-3xl md:text-4xl text-[#202020] tracking-[-0.02em]">
                Frequently answered questions.
              </h2>
            </div>

            <FaqAccordion />
          </div>
        </section>
      </main>

      <Footer />
    </div>
  );
}
