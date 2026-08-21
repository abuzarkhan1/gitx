"use client";

import React, { useState } from "react";
import Link from "next/link";
import { ArrowUpRight } from "lucide-react";
import { motion } from "framer-motion";
import { GithubIcon } from "@/components/ui/GithubIcon";
import { useCursor } from "@/components/providers/CursorProvider";

export function Footer() {
  const { setCursorVariant, resetCursor } = useCursor();
  const [isWordmarkHovered, setIsWordmarkHovered] = useState(false);

  const letters = ["G", "I", "T", "X"];

  return (
    <footer
      className="w-full bg-[#ff682c] text-[#ffffff] pt-16 pb-12 px-4 sm:px-6 md:px-12 overflow-hidden selection:bg-[#181818] selection:text-white"
      role="contentinfo"
    >
      <div className="max-w-[1200px] mx-auto space-y-12">
        {/* Top CTA Row on Orange */}
        <div
          className="flex flex-col lg:flex-row lg:items-center justify-between gap-6 pb-2"
          onMouseEnter={() => setCursorVariant("hover")}
          onMouseLeave={resetCursor}
        >
          <div className="space-y-2 max-w-xl">
            <h3 className="font-heading text-2xl md:text-3xl lg:text-4xl text-white tracking-[-0.02em]">
              Open-source Git repository intelligence.
            </h3>
          </div>

          <div className="flex flex-wrap items-center gap-3 flex-shrink-0">
            <a
              href="https://github.com/abuzarkhan1/gitx"
              target="_blank"
              rel="noopener noreferrer"
              className="inline-flex items-center gap-2 px-5 py-2.5 min-h-[44px] bg-[#ffffff] text-[#202020] text-xs font-mono uppercase tracking-wider hover:bg-[#f5f5f5] transition-colors border border-[#ffffff] font-medium"
              style={{ borderRadius: "0px" }}
            >
              <GithubIcon size={14} />
              <span>Clone on GitHub</span>
            </a>

            <a
              href="https://crates.io/crates/gitx-cli"
              target="_blank"
              rel="noopener noreferrer"
              className="inline-flex items-center gap-1 px-5 py-2.5 min-h-[44px] bg-[#ffffff] text-[#202020] text-xs font-mono uppercase tracking-wider hover:bg-[#f5f5f5] transition-colors border border-[#ffffff] font-medium"
              style={{ borderRadius: "0px" }}
            >
              <span>crates.io</span>
              <ArrowUpRight size={13} className="text-[#202020]" />
            </a>
          </div>
        </div>

        {/* 4-Column Navigation Links on Orange */}
        <div className="grid grid-cols-2 md:grid-cols-4 gap-8 pt-8 border-t border-white/25">
          <div>
            <div className="font-heading text-base text-white font-medium tracking-[-0.01em] mb-3.5">
              Commands
            </div>
            <ul className="space-y-2.5 text-xs text-white font-sans font-medium">
              <li><Link href="/#tui" className="text-white hover:text-black transition-colors">gitx (Interactive TUI)</Link></li>
              <li><Link href="/#hotspots" className="text-white hover:text-black transition-colors">gitx hotspots</Link></li>
              <li><Link href="/#lineage" className="text-white hover:text-black transition-colors">gitx lineage &lt;path&gt;</Link></li>
              <li><Link href="/#recovery" className="text-white hover:text-black transition-colors">gitx recovery</Link></li>
            </ul>
          </div>

          <div>
            <div className="font-heading text-base text-white font-medium tracking-[-0.01em] mb-3.5">
              Rust Crates
            </div>
            <ul className="space-y-2.5 text-xs text-white font-sans font-medium">
              <li><a href="https://github.com/abuzarkhan1/gitx" target="_blank" rel="noreferrer" className="text-white hover:text-black transition-colors">gitx-cli</a></li>
              <li><a href="https://github.com/abuzarkhan1/gitx" target="_blank" rel="noreferrer" className="text-white hover:text-black transition-colors">gitx-tui (Ratatui)</a></li>
              <li><a href="https://github.com/abuzarkhan1/gitx" target="_blank" rel="noreferrer" className="text-white hover:text-black transition-colors">gitx-history</a></li>
              <li><a href="https://github.com/abuzarkhan1/gitx" target="_blank" rel="noreferrer" className="text-white hover:text-black transition-colors">gitx-analysis</a></li>
            </ul>
          </div>

          <div>
            <div className="font-heading text-base text-white font-medium tracking-[-0.01em] mb-3.5">
              Documentation
            </div>
            <ul className="space-y-2.5 text-xs text-white font-sans font-medium">
              <li><Link href="/about" className="text-white hover:text-black transition-colors">System Architecture</Link></li>
              <li><Link href="/about" className="text-white hover:text-black transition-colors">Database Schema</Link></li>
              <li><Link href="/contact" className="text-white hover:text-black transition-colors">Issues &amp; RFCs</Link></li>
              <li><a href="https://github.com/abuzarkhan1/gitx" target="_blank" rel="noreferrer" className="text-white hover:text-black transition-colors">GitHub Repository</a></li>
            </ul>
          </div>

          <div>
            <div className="font-heading text-base text-white font-medium tracking-[-0.01em] mb-3.5">
              Invariants
            </div>
            <div className="text-xs text-white space-y-2 font-mono font-medium">
              <div>LATENCY: &lt;15ms</div>
              <div>INDEX: SQLite (WAL)</div>
              <div>AI/CLOUD: 0% (Local)</div>
            </div>
          </div>
        </div>

        {/* =====================================================================
            GIANT ANIMATED GITX WORDMARK (PURE WHITE ON ORANGE)
            ===================================================================== */}
        <div
          className="relative pt-12 pb-6 border-t border-white/20 overflow-hidden"
          onMouseEnter={() => {
            setIsWordmarkHovered(true);
            setCursorVariant("explore", "GITX", "dark");
          }}
          onMouseLeave={() => {
            setIsWordmarkHovered(false);
            resetCursor();
          }}
        >
          {/* Giant Animated Letters Container */}
          <div className="relative flex items-center justify-between select-none py-2 group">
            {letters.map((char, idx) => (
              <motion.div
                key={char}
                initial={{ y: 60, opacity: 0 }}
                whileInView={{ y: 0, opacity: 1 }}
                viewport={{ once: true, margin: "-10%" }}
                transition={{
                  duration: 0.8,
                  delay: idx * 0.08,
                  ease: [0.25, 1, 0.5, 1],
                }}
                whileHover={{
                  y: -14,
                  scale: 1.04,
                  transition: { type: "spring", stiffness: 400, damping: 15 },
                }}
                className="relative cursor-pointer transition-colors duration-300"
              >
                {/* Giant Typographic Character */}
                <span
                  className={`block font-heading text-[22vw] sm:text-[23vw] lg:text-[250px] font-normal leading-[0.75] tracking-[-0.04em] transition-all duration-300 text-white ${
                    isWordmarkHovered ? "drop-shadow-2xl" : "drop-shadow-md"
                  }`}
                >
                  {char}
                </span>
              </motion.div>
            ))}
          </div>

          {/* Ambient Subtle White Glow Underlay */}
          <motion.div
            animate={{
              opacity: isWordmarkHovered ? 0.3 : 0.1,
              scale: isWordmarkHovered ? 1.05 : 1,
            }}
            transition={{ duration: 0.4 }}
            className="absolute -bottom-10 left-1/2 -translate-x-1/2 w-[90%] h-32 bg-white blur-3xl pointer-events-none -z-10"
          />
        </div>

        {/* Clean Copyright on Orange */}
        <div className="pt-6 border-t border-white/20 text-xs font-mono text-white/70">
          <div>&copy; 2026 GitX Project. Open-source Git repository intelligence.</div>
        </div>
      </div>
    </footer>
  );
}
