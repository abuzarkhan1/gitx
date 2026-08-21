"use client";

import React, { useState } from "react";
import { RotateCcw, Check, Copy, ShieldCheck, GitBranch, GitCommit, Sparkles } from "lucide-react";
import { motion, AnimatePresence } from "framer-motion";
import { useCursor } from "@/components/providers/CursorProvider";

interface LostCommit {
  hash: string;
  msg: string;
  lostDate: string;
  cause: string;
  filesChanged: number;
  insertions: number;
  deletions: number;
  restoreCmd: string;
}

export function RecoveryStudio() {
  const [selectedHash, setSelectedHash] = useState("7a8f9b2");
  const [copied, setCopied] = useState(false);
  const [restoring, setRestoring] = useState(false);
  const { setCursorVariant, resetCursor } = useCursor();

  const lostCommits: LostCommit[] = [
    {
      hash: "7a8f9b2",
      msg: "feat(index): batch SQLite commit indexing with WAL mode",
      lostDate: "Yesterday (18:42)",
      cause: "git reset --hard HEAD~3 during rebase conflict",
      filesChanged: 4,
      insertions: 480,
      deletions: 12,
      restoreCmd: "git checkout -b recovered-wal-index 7a8f9b2",
    },
    {
      hash: "3c1e48d",
      msg: "fix(tui): handle terminal resize signal during search query",
      lostDate: "4 days ago",
      cause: "Detached HEAD branch deletion",
      filesChanged: 2,
      insertions: 84,
      deletions: 16,
      restoreCmd: "git cherry-pick 3c1e48d",
    },
    {
      hash: "9e5a10f",
      msg: "perf(graph): petgraph topological sort for branch divergence",
      lostDate: "1 week ago",
      cause: "Unmerged stash drop",
      filesChanged: 3,
      insertions: 210,
      deletions: 45,
      restoreCmd: "git checkout -b recovered-toposort 9e5a10f",
    },
  ];

  const current = lostCommits.find((c) => c.hash === selectedHash) || lostCommits[0];

  const handleCopy = () => {
    navigator.clipboard.writeText(current.restoreCmd);
    setCopied(true);
    setRestoring(true);
    setTimeout(() => setCopied(false), 2000);
    setTimeout(() => setRestoring(false), 2500);
  };

  return (
    <motion.div
      initial={{ opacity: 0, y: 20 }}
      whileInView={{ opacity: 1, y: 0 }}
      viewport={{ once: true }}
      transition={{ duration: 0.6, ease: [0.25, 1, 0.5, 1] }}
      className="w-full bg-[#ffffff] border border-[#e8e8e8] p-6 md:p-8"
      style={{ borderRadius: "0px" }}
      onMouseEnter={() => setCursorVariant("explore", "RECOVER")}
      onMouseLeave={resetCursor}
    >
      <div className="flex flex-col lg:flex-row lg:items-center justify-between gap-4 pb-6 border-b border-[#e8e8e8]">
        <div>
          <h3 className="font-heading text-2xl md:text-3xl text-[#202020] tracking-[-0.02em]">
            Dangling Commit &amp; Reflog Recovery
          </h3>
        </div>

        <div className="inline-flex items-center gap-1.5 text-xs font-mono text-[#ff682c] bg-[#f5f5f5] px-3 py-1.5 border border-[#e8e8e8]">
          <ShieldCheck size={14} />
          <span>Scans raw .git/objects without network calls</span>
        </div>
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-12 gap-8 mt-8 items-start">
        {/* Left: Lost Commits List */}
        <div className="lg:col-span-5 space-y-2">
          <div className="text-xs font-mono text-[#828282] uppercase mb-2">
            Recoverable Lost Commits
          </div>

          {lostCommits.map((c) => (
            <motion.div
              key={c.hash}
              whileHover={{ x: 3 }}
              onClick={() => setSelectedHash(c.hash)}
              className={`p-3.5 border cursor-pointer transition-all ${
                selectedHash === c.hash
                  ? "bg-[#f5f5f5] border-[#202020]"
                  : "bg-[#ffffff] border-[#e8e8e8] hover:border-[#828282]"
              }`}
              style={{ borderRadius: "0px" }}
            >
              <div className="flex justify-between items-center text-xs font-mono">
                <span className="font-bold text-[#ff682c]">{c.hash}</span>
                <span className="text-[#828282]">{c.lostDate}</span>
              </div>
              <div className="font-sans text-sm text-[#202020] font-medium mt-1 truncate">{c.msg}</div>
              <div className="text-xs text-[#828282] mt-1">{c.cause}</div>
            </motion.div>
          ))}
        </div>

        {/* Right: Interactive Commit Graph & Restore Command */}
        <div className="lg:col-span-7 bg-[#f5f5f5] p-6 border border-[#e8e8e8] flex flex-col justify-between" style={{ borderRadius: "0px" }}>
          <div className="space-y-4">
            <div className="flex justify-between items-center pb-3 border-b border-[#e8e8e8]">
              <span className="font-mono text-xs text-[#828282]">COMMIT OBJECT DETAILS</span>
              <span className="font-mono text-xs font-bold text-[#ff682c]">{current.hash}</span>
            </div>

            <div>
              <h4 className="font-heading text-xl text-[#202020]">{current.msg}</h4>
              <div className="text-xs text-[#4d4d4d] mt-1 flex gap-4 font-mono">
                <span>{current.filesChanged} files</span>
                <span className="text-[#27c93f]">+{current.insertions}</span>
                <span className="text-[#ff682c]">-{current.deletions}</span>
              </div>
            </div>

            {/* Interactive Animated DAG Visualizer */}
            <div className="p-4 bg-[#202020] text-[#ffffff] border border-[#333333] space-y-3 relative overflow-hidden">
              <div className="flex justify-between items-center text-[10px] font-mono text-[#828282]">
                <span>GIT GRAPH DAG TOPOLOGY</span>
                <span>{restoring ? "RECONNECTING TREE..." : "ORPHANED NODE DETECTED"}</span>
              </div>

              {/* Visual Branch Line Diagram */}
              <div className="py-2 flex items-center justify-between font-mono text-xs">
                {/* Main Branch Line */}
                <div className="flex items-center gap-2">
                  <span className="w-3 h-3 rounded-full bg-[#27c93f]" />
                  <span className="text-xs text-[#ebe6dd]">main (HEAD)</span>
                </div>

                {/* Animated Bridge Line */}
                <div className="flex-1 mx-4 relative h-0.5 bg-[#444444]">
                  {restoring && (
                    <motion.div
                      initial={{ left: "100%", width: "0%" }}
                      animate={{ left: "0%", width: "100%" }}
                      transition={{ duration: 0.8, ease: "easeInOut" }}
                      className="absolute inset-0 bg-[#ff682c]"
                    />
                  )}
                </div>

                {/* Dangling Node */}
                <div className="flex items-center gap-2">
                  <motion.span
                    animate={{
                      scale: restoring ? [1, 1.3, 1] : 1,
                      backgroundColor: restoring ? "#27c93f" : "#ff682c",
                    }}
                    transition={{ duration: 0.5 }}
                    className="w-3 h-3 rounded-full bg-[#ff682c]"
                  />
                  <span className="text-xs text-[#ff682c] font-bold">{current.hash}</span>
                </div>
              </div>

              <div className="text-[11px] text-[#828282] font-mono">
                Reason: {current.cause}
              </div>
            </div>
          </div>

          <div className="pt-6 mt-6 border-t border-[#e8e8e8] space-y-2">
            <div className="text-xs font-mono text-[#828282]">ONE-CLICK RESTORE COMMAND:</div>
            <div className="flex items-center justify-between p-3 bg-[#202020] text-[#ffffff] font-mono text-xs">
              <span className="truncate mr-2 text-[#ebe6dd]">{current.restoreCmd}</span>
              <button
                onClick={handleCopy}
                className="flex items-center gap-1.5 text-[#ff682c] hover:text-[#ffffff] flex-shrink-0 transition-colors"
              >
                {copied ? <Check size={13} className="text-[#27c93f]" /> : <Copy size={13} />}
                <span>{copied ? "Copied & Ready" : "Copy"}</span>
              </button>
            </div>
          </div>
        </div>
      </div>
    </motion.div>
  );
}
