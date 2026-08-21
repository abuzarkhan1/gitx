"use client";

import React, { useState } from "react";
import { Flame, Info, Sliders, RefreshCw, AlertTriangle } from "lucide-react";
import { motion, AnimatePresence } from "framer-motion";
import { useCursor } from "@/components/providers/CursorProvider";

interface FileRisk {
  name: string;
  path: string;
  totalScore: number;
  freqScore: number;
  churnScore: number;
  bugFixScore: number;
  ownershipScore: number;
  complexityScore: number;
  summary: string;
}

export function HotspotsRiskMatrix() {
  const [selectedIdx, setSelectedIdx] = useState(0);
  const [churnWeight, setChurnWeight] = useState(30);
  const [bugFixWeight, setBugFixWeight] = useState(20);
  const { setCursorVariant, resetCursor } = useCursor();

  const baseFiles: FileRisk[] = [
    {
      name: "engine.rs",
      path: "crates/gitx-storage/src/engine.rs",
      totalScore: 87,
      freqScore: 92,
      churnScore: 88,
      bugFixScore: 84,
      ownershipScore: 90,
      complexityScore: 78,
      summary: "High change frequency and multiple bug fix commits over 90 days. Ownership is concentrated on 1 primary author.",
    },
    {
      name: "risk.rs",
      path: "crates/gitx-analysis/src/risk.rs",
      totalScore: 78,
      freqScore: 80,
      churnScore: 76,
      bugFixScore: 82,
      ownershipScore: 74,
      complexityScore: 80,
      summary: "Frequent formula calibration modifications with high cyclomatic branching density.",
    },
    {
      name: "lineage.rs",
      path: "crates/gitx-history/src/lineage.rs",
      totalScore: 71,
      freqScore: 74,
      churnScore: 68,
      bugFixScore: 70,
      ownershipScore: 82,
      complexityScore: 64,
      summary: "Rename tracking similarity heuristics. Moderate commit volume with stable ownership.",
    },
    {
      name: "search.rs",
      path: "crates/gitx-search/src/query.rs",
      totalScore: 48,
      freqScore: 45,
      churnScore: 50,
      bugFixScore: 40,
      ownershipScore: 52,
      complexityScore: 55,
      summary: "Stable full-text search query tokenizer with low churn and distributed authorship.",
    },
  ];

  // Dynamically calculate score based on adjusted weight sliders
  const calculateDynamicScore = (f: FileRisk) => {
    const raw = (
      (churnWeight / 100) * f.churnScore +
      0.25 * f.freqScore +
      (bugFixWeight / 100) * f.bugFixScore +
      0.15 * f.ownershipScore +
      0.10 * f.complexityScore
    );
    // Normalize to 100
    const totalWeights = (churnWeight / 100) + 0.25 + (bugFixWeight / 100) + 0.15 + 0.10;
    return Math.round(raw / totalWeights);
  };

  const current = baseFiles[selectedIdx];
  const dynamicCurrentScore = calculateDynamicScore(current);

  return (
    <motion.div
      initial={{ opacity: 0, y: 20 }}
      whileInView={{ opacity: 1, y: 0 }}
      viewport={{ once: true }}
      transition={{ duration: 0.6, ease: [0.25, 1, 0.5, 1] }}
      className="w-full bg-[#ffffff] border border-[#e8e8e8] p-6 md:p-8"
      style={{ borderRadius: "0px" }}
      onMouseEnter={() => setCursorVariant("explore", "INSPECT")}
      onMouseLeave={resetCursor}
    >
      <div className="flex flex-col lg:flex-row lg:items-center justify-between gap-4 pb-6 border-b border-[#e8e8e8]">
        <div>
          <h3 className="font-heading text-2xl md:text-3xl text-[#202020] tracking-[-0.02em]">
            Deterministic Hotspot &amp; Maintenance Risk
          </h3>
        </div>

        <div className="inline-flex items-center gap-2 text-xs font-mono text-[#816729] bg-[#f5f5f5] px-3 py-1.5 border border-[#e8e8e8]">
          <Info size={13} />
          <span>Formula: {churnWeight}%·Churn + 25%·Freq + {bugFixWeight}%·Fixes + 15%·Owner + 10%·AST</span>
        </div>
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-12 gap-8 mt-8 items-start">
        {/* Left: File Selector & Live Weight Calibration */}
        <div className="lg:col-span-5 space-y-4">
          <div className="text-xs font-mono text-[#828282] uppercase">
            Select Inspected File
          </div>

          <div className="space-y-2">
            {baseFiles.map((f, i) => {
              const score = calculateDynamicScore(f);
              return (
                <motion.div
                  key={i}
                  whileHover={{ x: 3 }}
                  onClick={() => setSelectedIdx(i)}
                  className={`p-3.5 border cursor-pointer transition-all flex items-center justify-between ${
                    selectedIdx === i
                      ? "bg-[#f5f5f5] border-[#202020]"
                      : "bg-[#ffffff] border-[#e8e8e8] hover:border-[#828282]"
                  }`}
                  style={{ borderRadius: "0px" }}
                >
                  <div>
                    <div className="font-heading text-sm text-[#202020]">{f.name}</div>
                    <div className="text-xs text-[#828282] font-mono truncate max-w-[200px]">{f.path}</div>
                  </div>

                  <div className="flex items-center gap-2">
                    <motion.span
                      key={score}
                      initial={{ scale: 1.2 }}
                      animate={{ scale: 1 }}
                      className={`font-mono text-sm font-bold ${score > 75 ? "text-[#ff682c]" : "text-[#816729]"}`}
                    >
                      {score}
                    </motion.span>
                    {score > 75 && <Flame size={14} className="text-[#ff682c] animate-bounce" />}
                  </div>
                </motion.div>
              );
            })}
          </div>

          {/* Interactive Weight Tuning Controls */}
          <div className="p-4 bg-[#f9f9f9] border border-[#e8e8e8] space-y-3">
            <div className="flex items-center justify-between text-xs font-mono text-[#202020]">
              <span className="font-semibold flex items-center gap-1.5">
                <Sliders size={12} className="text-[#ff682c]" />
                CALIBRATE WEIGHTS (LIVE)
              </span>
              <button
                onClick={() => {
                  setChurnWeight(30);
                  setBugFixWeight(20);
                }}
                className="text-[10px] text-[#828282] hover:text-[#202020] flex items-center gap-1"
              >
                <RefreshCw size={10} />
                <span>Reset</span>
              </button>
            </div>

            <div>
              <div className="flex justify-between text-[11px] font-mono text-[#4d4d4d] mb-1">
                <span>Churn Volume Weight</span>
                <span className="font-bold text-[#ff682c]">{churnWeight}%</span>
              </div>
              <input
                type="range"
                min="10"
                max="50"
                value={churnWeight}
                onChange={(e) => setChurnWeight(Number(e.target.value))}
                className="w-full accent-[#ff682c] cursor-pointer"
              />
            </div>

            <div>
              <div className="flex justify-between text-[11px] font-mono text-[#4d4d4d] mb-1">
                <span>Bug-Fix Frequency Weight</span>
                <span className="font-bold text-[#816729]">{bugFixWeight}%</span>
              </div>
              <input
                type="range"
                min="10"
                max="40"
                value={bugFixWeight}
                onChange={(e) => setBugFixWeight(Number(e.target.value))}
                className="w-full accent-[#816729] cursor-pointer"
              />
            </div>
          </div>
        </div>

        {/* Right: Explainable Signal Breakdown with Animated Bars */}
        <div className="lg:col-span-7 bg-[#f5f5f5] p-6 border border-[#e8e8e8] space-y-6" style={{ borderRadius: "0px" }}>
          <div>
            <div className="flex justify-between items-center">
              <span className="font-mono text-xs text-[#828282]">INSPECTION TARGET</span>
              <motion.span
                key={dynamicCurrentScore}
                initial={{ scale: 1.15 }}
                animate={{ scale: 1 }}
                className="px-2.5 py-0.5 bg-[#202020] text-white text-[11px] font-mono"
              >
                RISK SCORE: {dynamicCurrentScore} / 100
              </motion.span>
            </div>
            <h4 className="font-heading text-xl text-[#202020] mt-1">{current.path}</h4>
            <p className="text-xs text-[#4d4d4d] mt-1.5 leading-relaxed">{current.summary}</p>
          </div>

          {/* Sub-score Bars with Spring Interpolation */}
          <div className="space-y-3 pt-2 border-t border-[#e8e8e8]">
            <div>
              <div className="flex justify-between text-xs font-mono text-[#4d4d4d] mb-1">
                <span>Change Churn Volume ({churnWeight}%)</span>
                <span className="text-[#202020] font-semibold font-mono">{current.churnScore} / 100</span>
              </div>
              <div className="w-full h-2 bg-[#e8e8e8]">
                <motion.div
                  initial={{ width: 0 }}
                  animate={{ width: `${current.churnScore}%` }}
                  transition={{ duration: 0.6, ease: "easeOut" }}
                  className="h-full bg-[#ff682c]"
                />
              </div>
            </div>

            <div>
              <div className="flex justify-between text-xs font-mono text-[#4d4d4d] mb-1">
                <span>Modification Frequency (25%)</span>
                <span className="text-[#202020] font-semibold font-mono">{current.freqScore} / 100</span>
              </div>
              <div className="w-full h-2 bg-[#e8e8e8]">
                <motion.div
                  initial={{ width: 0 }}
                  animate={{ width: `${current.freqScore}%` }}
                  transition={{ duration: 0.6, delay: 0.05, ease: "easeOut" }}
                  className="h-full bg-[#ff682c]"
                />
              </div>
            </div>

            <div>
              <div className="flex justify-between text-xs font-mono text-[#4d4d4d] mb-1">
                <span>Bug-Fix Commit Ratio ({bugFixWeight}%)</span>
                <span className="text-[#202020] font-semibold font-mono">{current.bugFixScore} / 100</span>
              </div>
              <div className="w-full h-2 bg-[#e8e8e8]">
                <motion.div
                  initial={{ width: 0 }}
                  animate={{ width: `${current.bugFixScore}%` }}
                  transition={{ duration: 0.6, delay: 0.1, ease: "easeOut" }}
                  className="h-full bg-[#816729]"
                />
              </div>
            </div>

            <div>
              <div className="flex justify-between text-xs font-mono text-[#4d4d4d] mb-1">
                <span>Ownership Concentration (15%)</span>
                <span className="text-[#202020] font-semibold font-mono">{current.ownershipScore} / 100</span>
              </div>
              <div className="w-full h-2 bg-[#e8e8e8]">
                <motion.div
                  initial={{ width: 0 }}
                  animate={{ width: `${current.ownershipScore}%` }}
                  transition={{ duration: 0.6, delay: 0.15, ease: "easeOut" }}
                  className="h-full bg-[#816729]"
                />
              </div>
            </div>

            <div>
              <div className="flex justify-between text-xs font-mono text-[#4d4d4d] mb-1">
                <span>Cyclomatic AST Complexity (10%)</span>
                <span className="text-[#202020] font-semibold font-mono">{current.complexityScore} / 100</span>
              </div>
              <div className="w-full h-2 bg-[#e8e8e8]">
                <motion.div
                  initial={{ width: 0 }}
                  animate={{ width: `${current.complexityScore}%` }}
                  transition={{ duration: 0.6, delay: 0.2, ease: "easeOut" }}
                  className="h-full bg-[#202020]"
                />
              </div>
            </div>
          </div>

          <div className="pt-2 border-t border-[#e8e8e8] flex justify-between items-center text-xs font-mono text-[#828282]">
            <span>CLI equivalent: <strong>gitx risk {current.path}</strong></span>
            <span className="text-[#ff682c]">Deterministic Output</span>
          </div>
        </div>
      </div>
    </motion.div>
  );
}
