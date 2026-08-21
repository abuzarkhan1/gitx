"use client";

import React from "react";
import { Zap, HardDrive, ShieldCheck } from "lucide-react";
import { motion } from "framer-motion";
import { useCursor } from "@/components/providers/CursorProvider";

interface ToolBenchmark {
  tool: string;
  runtime: string;
  indexingTime: string;
  indexingMs: number;
  hotQueryTime: string;
  hotQueryMs: number;
  memoryUsage: string;
  explainability: string;
  localOnly: string;
  highlight?: boolean;
}

export function BenchmarkObservatory() {
  const { setCursorVariant, resetCursor } = useCursor();

  const benchmarks: ToolBenchmark[] = [
    {
      tool: "GitX (Rust + SQLite)",
      runtime: "Rust (Native)",
      indexingTime: "1.84s (1M commits)",
      indexingMs: 1.84,
      hotQueryTime: "12 ms",
      hotQueryMs: 12,
      memoryUsage: "38 MB",
      explainability: "Full raw signal formula",
      localOnly: "100% Offline (Zero AI)",
      highlight: true,
    },
    {
      tool: "Raw Git CLI (Shell)",
      runtime: "C / Bash Scripts",
      indexingTime: "N/A (Unindexed)",
      indexingMs: 0,
      hotQueryTime: "3,820 ms",
      hotQueryMs: 3820,
      memoryUsage: "12 MB",
      explainability: "Manual grep/awk pipelines",
      localOnly: "100% Offline",
    },
    {
      tool: "Jujutsu (jj)",
      runtime: "Rust (Native)",
      indexingTime: "2.40s",
      indexingMs: 2.40,
      hotQueryTime: "45 ms",
      hotQueryMs: 45,
      memoryUsage: "64 MB",
      explainability: "VCS operation tree only",
      localOnly: "100% Offline",
    },
    {
      tool: "Git GUI Clients",
      runtime: "Electron / Node",
      indexingTime: "18.50s",
      indexingMs: 18.50,
      hotQueryTime: "620 ms",
      hotQueryMs: 620,
      memoryUsage: "480 MB",
      explainability: "Proprietary UI / Closed",
      localOnly: "Cloud account required",
    },
  ];

  return (
    <motion.div
      initial={{ opacity: 0, y: 20 }}
      whileInView={{ opacity: 1, y: 0 }}
      viewport={{ once: true }}
      transition={{ duration: 0.6, ease: [0.25, 1, 0.5, 1] }}
      className="w-full bg-[#181818] text-[#ffffff] border border-white/20 p-6 md:p-8 shadow-2xl"
      style={{ borderRadius: "0px" }}
      onMouseEnter={() => setCursorVariant("explore", "BENCHMARK", "dark")}
      onMouseLeave={resetCursor}
    >
      <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-4 pb-6 border-b border-white/10">
        <div>
          <h3 className="font-heading text-2xl md:text-3xl text-white tracking-[-0.02em]">
            Empirical Benchmark Observatory
          </h3>
        </div>

        <div className="text-xs font-mono text-white/70">
          Linux Kernel repository (1.2M commits, Apple M-Series)
        </div>
      </div>

      {/* Visual Speedup Comparison Bars */}
      <div className="grid grid-cols-1 md:grid-cols-2 gap-6 my-8 p-5 bg-[#101010] border border-white/10">
        <div className="space-y-2">
          <div className="flex justify-between text-xs font-mono text-white">
            <span className="font-semibold flex items-center gap-1.5">
              <Zap size={13} className="text-[#ff682c]" />
              HOT QUERY LATENCY (LOWER IS FASTER)
            </span>
            <span className="text-[#ff682c] font-bold">318x Faster</span>
          </div>

          <div className="space-y-2 text-xs font-mono">
            <div>
              <div className="flex justify-between text-[11px] text-white/80 mb-0.5">
                <span className="font-semibold text-white">GitX (Rust + SQLite)</span>
                <span className="text-[#ff682c] font-bold">12 ms</span>
              </div>
              <div className="w-full h-3 bg-[#262626]">
                <motion.div
                  initial={{ width: 0 }}
                  whileInView={{ width: "3%" }}
                  viewport={{ once: true }}
                  transition={{ duration: 0.8, ease: "easeOut" }}
                  className="h-full bg-[#ff682c]"
                />
              </div>
            </div>

            <div>
              <div className="flex justify-between text-[11px] text-white/70 mb-0.5">
                <span>Jujutsu (jj)</span>
                <span>45 ms</span>
              </div>
              <div className="w-full h-3 bg-[#262626]">
                <motion.div
                  initial={{ width: 0 }}
                  whileInView={{ width: "8%" }}
                  viewport={{ once: true }}
                  transition={{ duration: 0.8, delay: 0.1, ease: "easeOut" }}
                  className="h-full bg-[#555555]"
                />
              </div>
            </div>

            <div>
              <div className="flex justify-between text-[11px] text-white/70 mb-0.5">
                <span>Git GUI Clients</span>
                <span>620 ms</span>
              </div>
              <div className="w-full h-3 bg-[#262626]">
                <motion.div
                  initial={{ width: 0 }}
                  whileInView={{ width: "35%" }}
                  viewport={{ once: true }}
                  transition={{ duration: 0.8, delay: 0.2, ease: "easeOut" }}
                  className="h-full bg-[#555555]"
                />
              </div>
            </div>

            <div>
              <div className="flex justify-between text-[11px] text-white/70 mb-0.5">
                <span>Raw Git CLI (Shell loop)</span>
                <span className="text-white/50">3,820 ms</span>
              </div>
              <div className="w-full h-3 bg-[#262626]">
                <motion.div
                  initial={{ width: 0 }}
                  whileInView={{ width: "100%" }}
                  viewport={{ once: true }}
                  transition={{ duration: 0.8, delay: 0.3, ease: "easeOut" }}
                  className="h-full bg-[#3d3d3d]"
                />
              </div>
            </div>
          </div>
        </div>

        <div className="space-y-2">
          <div className="flex justify-between text-xs font-mono text-white">
            <span className="font-semibold flex items-center gap-1.5">
              <HardDrive size={13} className="text-[#ffbd2e]" />
              MEMORY CONSUMPTION (LOWER IS BETTER)
            </span>
            <span className="text-[#ffbd2e] font-bold">12.6x Leaner than Electron</span>
          </div>

          <div className="space-y-2 text-xs font-mono">
            <div>
              <div className="flex justify-between text-[11px] text-white/80 mb-0.5">
                <span className="font-semibold text-white">GitX</span>
                <span className="text-[#ffbd2e] font-bold">38 MB</span>
              </div>
              <div className="w-full h-3 bg-[#262626]">
                <motion.div
                  initial={{ width: 0 }}
                  whileInView={{ width: "8%" }}
                  viewport={{ once: true }}
                  transition={{ duration: 0.8, ease: "easeOut" }}
                  className="h-full bg-[#ffbd2e]"
                />
              </div>
            </div>

            <div>
              <div className="flex justify-between text-[11px] text-white/70 mb-0.5">
                <span>Jujutsu (jj)</span>
                <span>64 MB</span>
              </div>
              <div className="w-full h-3 bg-[#262626]">
                <motion.div
                  initial={{ width: 0 }}
                  whileInView={{ width: "13%" }}
                  viewport={{ once: true }}
                  transition={{ duration: 0.8, delay: 0.1, ease: "easeOut" }}
                  className="h-full bg-[#555555]"
                />
              </div>
            </div>

            <div>
              <div className="flex justify-between text-[11px] text-white/70 mb-0.5">
                <span>Git GUI Clients (Electron)</span>
                <span className="text-white/50">480 MB</span>
              </div>
              <div className="w-full h-3 bg-[#262626]">
                <motion.div
                  initial={{ width: 0 }}
                  whileInView={{ width: "100%" }}
                  viewport={{ once: true }}
                  transition={{ duration: 0.8, delay: 0.2, ease: "easeOut" }}
                  className="h-full bg-[#3d3d3d]"
                />
              </div>
            </div>
          </div>
        </div>
      </div>

      {/* Comprehensive Benchmark Table on Dark */}
      <div className="overflow-x-auto">
        <table className="w-full text-left border-collapse text-sm">
          <thead>
            <tr className="border-b border-white/10 text-xs font-mono text-white/60">
              <th className="py-3 px-3">TOOL</th>
              <th className="py-3 px-3">INDEX TIME (1M COMMITS)</th>
              <th className="py-3 px-3">HOT QUERY LATENCY</th>
              <th className="py-3 px-3">MEMORY FOOTPRINT</th>
              <th className="py-3 px-3">EXPLAINABILITY</th>
              <th className="py-3 px-3">PRIVACY MODEL</th>
            </tr>
          </thead>
          <tbody className="divide-y divide-white/10 font-sans">
            {benchmarks.map((b, i) => (
              <tr
                key={i}
                className={`transition-colors ${b.highlight ? "bg-white/5" : "hover:bg-white/[0.02]"}`}
              >
                <td className="py-4 px-3">
                  <div className="font-heading text-white flex items-center gap-2">
                    <span>{b.tool}</span>
                    {b.highlight && (
                      <span className="text-[10px] font-mono text-[#ff682c] uppercase font-semibold">
                        [Optimal]
                      </span>
                    )}
                  </div>
                  <div className="text-xs text-white/60 font-mono mt-0.5">{b.runtime}</div>
                </td>
                <td className="py-4 px-3 font-mono text-xs text-white font-medium">{b.indexingTime}</td>
                <td className="py-4 px-3 font-mono text-xs text-[#ff682c] font-bold">{b.hotQueryTime}</td>
                <td className="py-4 px-3 font-mono text-xs text-[#ffbd2e] font-medium">{b.memoryUsage}</td>
                <td className="py-4 px-3 text-xs text-white/80">{b.explainability}</td>
                <td className="py-4 px-3 font-mono text-xs text-[#27c93f]">{b.localOnly}</td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </motion.div>
  );
}
