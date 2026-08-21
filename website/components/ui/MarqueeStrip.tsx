"use client";

import React from "react";
import { motion } from "framer-motion";

export function MarqueeStrip() {
  const items = [
    "100% LOCAL & OFFLINE",
    "SUB-15MS SQLITE WAL CACHE",
    "RATATUI 0.28 TUI",
    "RENAME-FOLLOWING LINEAGE",
    "REFLOG & DANGLING RESCUE",
    "ZERO AI · ZERO TELEMETRY",
    "PETGRAPH DAG TOPOLOGICAL SORT",
    "11 MODULAR RUST CRATES",
    "CRITERION BENCHMARKED",
    "LOCAL-FIRST ARCHAEOLOGY",
    "CLAP 4.5 CLI PARSER",
    "GIX (GIT2) NATIVE",
  ];

  // Repeat for continuous seamless loop
  const marqueeItems = [...items, ...items, ...items];

  return (
    <div
      className="w-full bg-[#ff682c] border-y border-[#e0561f] py-3 overflow-hidden select-none relative z-20 group shadow-inner"
      aria-label="GitX Key Capabilities"
    >
      <div className="flex w-max animate-marquee group-hover:[animation-play-state:paused]">
        {marqueeItems.map((item, index) => (
          <div
            key={index}
            className="flex items-center gap-6 px-4 text-xs font-mono text-[#ffffff] font-medium tracking-wider uppercase whitespace-nowrap"
          >
            <span>{item}</span>
            <span className="w-1.5 h-1.5 rounded-full bg-[#181818] flex-shrink-0" />
          </div>
        ))}
      </div>
    </div>
  );
}
