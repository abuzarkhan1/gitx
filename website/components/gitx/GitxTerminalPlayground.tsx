"use client";

import React, { useState, useEffect } from "react";
import { Terminal, Shield, Flame, History, RotateCcw, Copy, Check, ChevronRight, Play, Pause, RefreshCw } from "lucide-react";
import { motion, AnimatePresence } from "framer-motion";
import { useCursor } from "@/components/providers/CursorProvider";

export function GitxTerminalPlayground() {
  const [activeTab, setActiveTab] = useState<"overview" | "hotspots" | "lineage" | "ownership" | "recovery">("overview");
  const [copied, setCopied] = useState(false);
  const [selectedFileIdx, setSelectedFileIdx] = useState(0);
  const [isSimulating, setIsSimulating] = useState(false);
  const [simulatedCommits, setSimulatedCommits] = useState(1248);
  const [lastKeyPressed, setLastKeyPressed] = useState<string | null>(null);
  const { setCursorVariant, resetCursor } = useCursor();

  // Keyboard shortcut listener with visual feedback
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (["INPUT", "TEXTAREA"].includes((e.target as HTMLElement)?.tagName)) return;
      if (["1", "2", "3", "4", "5"].includes(e.key)) {
        setLastKeyPressed(e.key);
        setTimeout(() => setLastKeyPressed(null), 1200);
      }
      if (e.key === "1") setActiveTab("overview");
      if (e.key === "2") setActiveTab("hotspots");
      if (e.key === "3") setActiveTab("lineage");
      if (e.key === "4") setActiveTab("ownership");
      if (e.key === "5") setActiveTab("recovery");
    };
    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, []);

  // Live indexing counter simulation
  useEffect(() => {
    if (!isSimulating) return;
    const interval = setInterval(() => {
      setSimulatedCommits((prev) => prev + Math.floor(Math.random() * 8 + 3));
    }, 280);
    return () => clearInterval(interval);
  }, [isSimulating]);

  const hotspots = [
    { path: "crates/gitx-storage/src/engine.rs", risk: 87, churn: "+1,420 / -890", commits: 46, authors: 4, topAuthor: "Alex R. (68%)", bugFixes: 12 },
    { path: "crates/gitx-analysis/src/risk.rs", risk: 78, churn: "+940 / -310", commits: 32, authors: 3, topAuthor: "Elena R. (54%)", bugFixes: 8 },
    { path: "crates/gitx-history/src/lineage.rs", risk: 71, churn: "+620 / -180", commits: 24, authors: 2, topAuthor: "Marcus V. (82%)", bugFixes: 5 },
    { path: "crates/gitx-tui/src/views/hotspots.rs", risk: 58, churn: "+410 / -95", commits: 18, authors: 2, topAuthor: "Dev Team (50%)", bugFixes: 3 },
    { path: "crates/gitx-core/src/model.rs", risk: 42, churn: "+180 / -40", commits: 12, authors: 5, topAuthor: "Distributed", bugFixes: 1 },
  ];

  const tabs = [
    { id: "overview", key: "1", label: "Overview", icon: <Terminal size={12} /> },
    { id: "hotspots", key: "2", label: "Hotspots", icon: <Flame size={12} /> },
    { id: "lineage", key: "3", label: "Lineage", icon: <History size={12} /> },
    { id: "ownership", key: "4", label: "Ownership", icon: <Shield size={12} /> },
    { id: "recovery", key: "5", label: "Recovery", icon: <RotateCcw size={12} /> },
  ] as const;

  const copyCommand = (cmd: string) => {
    navigator.clipboard.writeText(cmd);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  return (
    <motion.div
      initial={{ opacity: 0, y: 30, scale: 0.98 }}
      whileInView={{ opacity: 1, y: 0, scale: 1 }}
      viewport={{ once: true }}
      transition={{ duration: 0.7, ease: [0.25, 1, 0.5, 1] }}
      className="w-full bg-[#181818] text-[#ffffff] border border-[#333333] select-none font-mono text-xs shadow-2xl transition-all relative overflow-hidden"
      style={{ borderRadius: "0px" }}
      onMouseEnter={() => setCursorVariant("explore", "TUI", "dark")}
      onMouseLeave={resetCursor}
    >
      {/* Terminal Title Bar */}
      <div className="flex items-center justify-between px-4 py-2.5 bg-[#121212] border-b border-[#333333]">
        <div className="flex items-center gap-2">
          <div className="flex items-center gap-1.5">
            <span className="w-2.5 h-2.5 rounded-full bg-[#ff5f56]" />
            <span className="w-2.5 h-2.5 rounded-full bg-[#ffbd2e]" />
            <span className="w-2.5 h-2.5 rounded-full bg-[#27c93f]" />
          </div>
          <span className="text-[#828282] ml-2 text-[11px] font-mono">gitx v0.1.0 — ~/hyper-engine</span>
        </div>

        <div className="flex items-center gap-3 text-[11px] text-[#828282]">
          <button
            onClick={() => setIsSimulating(!isSimulating)}
            className="flex items-center gap-1 text-[10px] px-2 py-0.5 bg-[#252525] border border-[#444444] text-[#ebe6dd] hover:border-[#ff682c] transition-colors"
            title="Toggle live indexing simulation"
          >
            {isSimulating ? <Pause size={10} className="text-[#ff682c]" /> : <Play size={10} className="text-[#27c93f]" />}
            <span>{isSimulating ? "Streaming" : "Simulate Live"}</span>
          </button>

          <span className="text-[#27c93f] flex items-center gap-1">
            <span className={`w-1.5 h-1.5 rounded-full bg-[#27c93f] ${isSimulating ? "animate-ping" : "animate-pulse"}`} />
            WAL Mode
          </span>
          <span className="hidden sm:inline font-mono">{simulatedCommits.toLocaleString()} commits</span>
          <span className="text-[#ff682c] font-semibold">12ms</span>
        </div>
      </div>

      {/* TUI Navigation Tabs */}
      <div role="tablist" aria-label="Terminal views" className="flex items-center border-b border-[#333333] bg-[#1a1a1a] overflow-x-auto no-scrollbar relative">
        {tabs.map((tab) => (
          <button
            key={tab.id}
            role="tab"
            aria-selected={activeTab === tab.id}
            aria-controls={`terminal-panel-${tab.id}`}
            onClick={() => setActiveTab(tab.id)}
            className={`flex items-center gap-1.5 px-4 py-2 text-[11px] border-r border-[#333333] transition-colors whitespace-nowrap relative ${
              activeTab === tab.id
                ? "bg-[#242424] text-[#ff682c] font-semibold"
                : "text-[#828282] hover:text-[#ffffff] hover:bg-[#202020]"
            }`}
          >
            {tab.icon}
            <span>{tab.key}: {tab.label}</span>
            {activeTab === tab.id && (
              <motion.div
                layoutId="tuiActiveTabUnderline"
                className="absolute bottom-0 left-0 right-0 h-[2px] bg-[#ff682c]"
                transition={{ type: "spring", stiffness: 500, damping: 35 }}
              />
            )}
          </button>
        ))}

        {/* Keystroke HUD Notification */}
        {lastKeyPressed && (
          <motion.div
            initial={{ opacity: 0, scale: 0.8 }}
            animate={{ opacity: 1, scale: 1 }}
            exit={{ opacity: 0 }}
            className="absolute right-3 top-1/2 -translate-y-1/2 px-2 py-0.5 bg-[#ff682c] text-white text-[10px] font-bold uppercase tracking-wider"
          >
            Key [{lastKeyPressed}] Active
          </motion.div>
        )}
      </div>

      {/* TUI View Canvas */}
      <div className="p-5 min-h-[310px] flex flex-col justify-between bg-[#181818]">
        <AnimatePresence mode="wait">
          {activeTab === "overview" && (
            <motion.div
              key="overview"
              id="terminal-panel-overview"
              role="tabpanel"
              initial={{ opacity: 0, y: 6 }}
              animate={{ opacity: 1, y: 0 }}
              exit={{ opacity: 0, y: -6 }}
              transition={{ duration: 0.2 }}
              className="space-y-4"
            >
              {/* Repository Health Matrix */}
              <div className="grid grid-cols-2 sm:grid-cols-4 gap-2.5">
                <div className="p-3 bg-[#202020] border border-[#333333] hover:border-[#444] transition-colors">
                  <div className="text-[10px] text-[#828282] uppercase">Repo Health</div>
                  <div className="text-xl text-[#27c93f] font-bold mt-0.5">94 / 100</div>
                  <div className="text-[10px] text-[#828282] mt-0.5">Low risk velocity</div>
                </div>

                <div className="p-3 bg-[#202020] border border-[#333333] hover:border-[#444] transition-colors">
                  <div className="text-[10px] text-[#828282] uppercase">Hotspots</div>
                  <div className="text-xl text-[#ff682c] font-bold mt-0.5">5 critical</div>
                  <div className="text-[10px] text-[#828282] mt-0.5">&gt;70 risk score</div>
                </div>

                <div className="p-3 bg-[#202020] border border-[#333333] hover:border-[#444] transition-colors">
                  <div className="text-[10px] text-[#828282] uppercase">Bus Factor</div>
                  <div className="text-xl text-[#ffffff] font-bold mt-0.5">3 authors</div>
                  <div className="text-[10px] text-[#828282] mt-0.5">Healthy spread</div>
                </div>

                <div className="p-3 bg-[#202020] border border-[#333333] hover:border-[#444] transition-colors">
                  <div className="text-[10px] text-[#828282] uppercase">Dangling</div>
                  <div className="text-xl text-[#ffbd2e] font-bold mt-0.5">3 commits</div>
                  <div className="text-[10px] text-[#828282] mt-0.5">Recoverable</div>
                </div>
              </div>

              {/* Animated Commit Cadence Sparkline */}
              <div className="p-3 bg-[#202020] border border-[#333333] space-y-2">
                <div className="flex justify-between text-[11px] text-[#828282]">
                  <span>COMMIT CADENCE (LAST 30 DAYS)</span>
                  <span className="text-[#27c93f] font-mono">148 total commits</span>
                </div>
                <div className="flex items-end gap-1.5 h-12 pt-2">
                  {[12, 18, 8, 24, 30, 45, 20, 15, 28, 38, 52, 40, 22, 19, 35, 48, 60, 42, 30, 25, 33, 44, 55, 38, 20, 15, 29, 41, 50, 62].map((v, i) => (
                    <motion.div
                      key={i}
                      initial={{ height: 0 }}
                      animate={{ height: `${(v / 65) * 100}%` }}
                      transition={{ duration: 0.5, delay: i * 0.015, ease: "easeOut" }}
                      className="flex-1 bg-[#333333] hover:bg-[#ff682c] transition-colors rounded-none cursor-pointer"
                      title={`Day ${i + 1}: ${v} commits`}
                    />
                  ))}
                </div>
              </div>
            </motion.div>
          )}

          {activeTab === "hotspots" && (
            <motion.div
              key="hotspots"
              id="terminal-panel-hotspots"
              role="tabpanel"
              initial={{ opacity: 0, y: 6 }}
              animate={{ opacity: 1, y: 0 }}
              exit={{ opacity: 0, y: -6 }}
              transition={{ duration: 0.2 }}
              className="space-y-3"
            >
              <div className="flex justify-between items-center text-[10px] text-[#828282] uppercase px-1">
                <span>Ranked High-Risk Files</span>
                <span>Click to inspect</span>
              </div>
              <div className="space-y-1.5">
                {hotspots.map((h, i) => (
                  <motion.div
                    key={i}
                    whileHover={{ x: 3 }}
                    onClick={() => setSelectedFileIdx(i)}
                    className={`flex items-center justify-between p-2.5 border cursor-pointer transition-all ${
                      selectedFileIdx === i
                        ? "bg-[#252525] border-[#ff682c] text-[#ffffff]"
                        : "bg-[#202020] border-[#333333] text-[#828282] hover:border-[#4d4d4d]"
                    }`}
                  >
                    <div className="flex items-center gap-2 truncate pr-2">
                      <ChevronRight size={12} className={selectedFileIdx === i ? "text-[#ff682c]" : "text-transparent"} />
                      <span className="text-[#ffffff] font-medium truncate">{h.path}</span>
                    </div>
                    <div className="flex items-center gap-4 flex-shrink-0 text-[11px]">
                      <span className="text-[#828282] hidden sm:inline font-mono">{h.churn}</span>
                      <span className={`font-bold ${h.risk > 75 ? "text-[#ff682c]" : "text-[#ffbd2e]"}`}>
                        RISK: {h.risk}
                      </span>
                    </div>
                  </motion.div>
                ))}
              </div>
            </motion.div>
          )}

          {activeTab === "lineage" && (
            <motion.div
              key="lineage"
              id="terminal-panel-lineage"
              role="tabpanel"
              initial={{ opacity: 0, y: 6 }}
              animate={{ opacity: 1, y: 0 }}
              exit={{ opacity: 0, y: -6 }}
              transition={{ duration: 0.2 }}
              className="space-y-3 font-mono text-[11px]"
            >
              <div className="p-3 bg-[#202020] border border-[#333333] space-y-2">
                <div className="text-[#828282]">QUERY: gitx lineage crates/gitx-core/src/ast.rs</div>
                <div className="space-y-1.5 pl-2 border-l-2 border-[#ff682c]">
                  <div className="text-[#ebe6dd]">2021-11-04 (8f1a02d) &rarr; src/utils/tokenizer.rs [Created]</div>
                  <div className="text-[#ebe6dd]">2023-04-19 (2b9e41a) &rarr; src/lexer/parse_tokens.rs [92% match]</div>
                  <div className="text-[#ff682c] font-semibold">2025-01-14 (e4d79c0) &rarr; crates/gitx-core/src/ast.rs [Crate Extract]</div>
                </div>
              </div>
              <div className="text-[10px] text-[#828282]">
                &radic; 46 commits tracked across 3 file paths and 2 repository refactors.
              </div>
            </motion.div>
          )}

          {activeTab === "ownership" && (
            <motion.div
              key="ownership"
              id="terminal-panel-ownership"
              role="tabpanel"
              initial={{ opacity: 0, y: 6 }}
              animate={{ opacity: 1, y: 0 }}
              exit={{ opacity: 0, y: -6 }}
              transition={{ duration: 0.2 }}
              className="space-y-3"
            >
              <div className="text-[11px] text-[#828282]">KNOWLEDGE BUS FACTOR BY MODULE</div>
              <div className="space-y-2.5">
                <div>
                  <div className="flex justify-between text-[11px] mb-1">
                    <span>crates/gitx-storage (Alex R.)</span>
                    <span className="text-[#ff682c] font-bold">68% ownership [High]</span>
                  </div>
                  <div className="w-full h-1.5 bg-[#333333]">
                    <motion.div initial={{ width: 0 }} animate={{ width: "68%" }} transition={{ duration: 0.6 }} className="h-full bg-[#ff682c]" />
                  </div>
                </div>
                <div>
                  <div className="flex justify-between text-[11px] mb-1">
                    <span>crates/gitx-analysis (Elena R. + 2 others)</span>
                    <span className="text-[#27c93f] font-bold">38% spread [Healthy]</span>
                  </div>
                  <div className="w-full h-1.5 bg-[#333333]">
                    <motion.div initial={{ width: 0 }} animate={{ width: "38%" }} transition={{ duration: 0.6 }} className="h-full bg-[#27c93f]" />
                  </div>
                </div>
                <div>
                  <div className="flex justify-between text-[11px] mb-1">
                    <span>crates/gitx-history (Marcus V.)</span>
                    <span className="text-[#ffbd2e] font-bold">52% ownership [Moderate]</span>
                  </div>
                  <div className="w-full h-1.5 bg-[#333333]">
                    <motion.div initial={{ width: 0 }} animate={{ width: "52%" }} transition={{ duration: 0.6 }} className="h-full bg-[#ffbd2e]" />
                  </div>
                </div>
              </div>
            </motion.div>
          )}

          {activeTab === "recovery" && (
            <motion.div
              key="recovery"
              id="terminal-panel-recovery"
              role="tabpanel"
              initial={{ opacity: 0, y: 6 }}
              animate={{ opacity: 1, y: 0 }}
              exit={{ opacity: 0, y: -6 }}
              transition={{ duration: 0.2 }}
              className="space-y-3 font-mono text-[11px]"
            >
              <div className="p-3 bg-[#202020] border border-[#333333] space-y-1.5">
                <div className="flex justify-between text-[#ffbd2e]">
                  <span>FOUND ORPHAN COMMIT: 7a8f9b2</span>
                  <span>18:42 Yesterday</span>
                </div>
                <div className="text-[#ffffff]">feat(index): batch SQLite commit indexing with WAL mode</div>
                <div className="text-[10px] text-[#828282]">Lost during: git reset --hard HEAD~3</div>
              </div>
              <div className="text-[11px] text-[#27c93f]">
                Restore command: <strong>git checkout -b recovered-wal-index 7a8f9b2</strong>
              </div>
            </motion.div>
          )}
        </AnimatePresence>

        {/* Terminal Bottom Controls & Shortcuts */}
        <div className="pt-4 mt-4 border-t border-[#333333] flex flex-wrap items-center justify-between gap-3 text-[11px] text-[#828282]">
          <div className="flex items-center gap-3">
            <span className="text-[#ebe6dd]">Keys: <kbd className="px-1 bg-[#252525] border border-[#444] text-[#ff682c]">1-5</kbd> switch views</span>
            <span><kbd className="px-1 bg-[#252525] border border-[#444]">q</kbd> quit</span>
          </div>

          <button
            onClick={() => copyCommand("gitx")}
            className="flex items-center gap-1.5 text-[#ff682c] hover:text-[#ffffff] transition-colors"
          >
            {copied ? <Check size={12} /> : <Copy size={12} />}
            <span>{copied ? "Copied 'gitx'" : "Run in terminal: gitx"}</span>
          </button>
        </div>
      </div>
    </motion.div>
  );
}
