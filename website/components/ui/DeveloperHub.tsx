"use client";

import React, { useState } from "react";
import { Copy, Check, Terminal } from "lucide-react";
import { motion, AnimatePresence } from "framer-motion";
import { useCursor } from "@/components/providers/CursorProvider";

export function DeveloperHub() {
  const [activeTab, setActiveTab] = useState<"curl" | "cargo" | "brew" | "powershell">("curl");
  const [copied, setCopied] = useState(false);
  const { setCursorVariant, resetCursor } = useCursor();

  const commands = {
    curl: `# One-line binary installer for macOS (aarch64/x86_64) & Linux
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/abuzarkhan1/gitx/releases/latest/download/gitx-installer.sh | sh

# Verify installation:
gitx --version`,

    cargo: `# Install from crates.io (locked dependencies)
cargo install gitx-cli --locked

# Generate shell completions:
gitx completions zsh > ~/.zfunc/_gitx`,

    brew: `# Install via Homebrew tap
brew install abuzarkhan1/tap/gitx

# Launch interactive TUI:
gitx`,

    powershell: `# Windows PowerShell installer
irm https://github.com/abuzarkhan1/gitx/releases/latest/download/gitx-installer.ps1 | iex

# Run stats on current repository:
gitx stats`
  };

  const copyCode = () => {
    navigator.clipboard.writeText(commands[activeTab]);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  return (
    <motion.div
      initial={{ opacity: 0, y: 20 }}
      whileInView={{ opacity: 1, y: 0 }}
      viewport={{ once: true }}
      transition={{ duration: 0.6, ease: [0.25, 1, 0.5, 1] }}
      className="w-full bg-[#181818] text-[#ffffff] p-6 md:p-8 border-t-2 border-[#ff682c] border-x border-b border-[#333333] shadow-xl"
      style={{ borderRadius: "0px" }}
      onMouseEnter={() => setCursorVariant("explore", "INSTALL", "dark")}
      onMouseLeave={resetCursor}
    >
      <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-4 pb-4 border-b border-[#333333]">
        <div className="flex items-center gap-2">
          <Terminal size={18} className="text-[#ff682c]" />
          <h3 className="font-heading text-xl md:text-2xl text-[#ffffff] tracking-[-0.02em]">
            Installation &amp; Quickstart
          </h3>
        </div>

        <div className="flex items-center gap-2">
          <div role="tablist" aria-label="Installation methods" className="flex items-center bg-[#101010] p-1 border border-[#333333]">
            {(["curl", "cargo", "brew", "powershell"] as const).map((tab) => (
              <button
                key={tab}
                role="tab"
                aria-selected={activeTab === tab}
                onClick={() => setActiveTab(tab)}
                className={`relative px-3 py-1 text-xs font-mono uppercase transition-colors z-10 ${
                  activeTab === tab ? "text-[#181818] font-bold" : "text-[#828282] hover:text-[#ffffff]"
                }`}
              >
                {activeTab === tab && (
                  <motion.div
                    layoutId="activeInstallTab"
                    className="absolute inset-0 bg-[#ff682c] z-[-1]"
                    transition={{ type: "spring", stiffness: 500, damping: 32 }}
                  />
                )}
                {tab === "curl" ? "Shell Script" : tab}
              </button>
            ))}
          </div>

          <motion.button
            whileTap={{ scale: 0.95 }}
            onClick={copyCode}
            aria-label="Copy installation command"
            className="flex items-center gap-1.5 px-3 py-1.5 text-xs font-mono border border-[#4d4d4d] text-[#ebe6dd] hover:border-[#ff682c] hover:text-[#ff682c] transition-colors"
            style={{ borderRadius: "0px" }}
          >
            <AnimatePresence mode="wait" initial={false}>
              {copied ? (
                <motion.div
                  key="check"
                  initial={{ scale: 0.8, opacity: 0 }}
                  animate={{ scale: 1, opacity: 1 }}
                  exit={{ scale: 0.8, opacity: 0 }}
                  className="flex items-center gap-1 text-[#27c93f]"
                >
                  <Check size={12} />
                  <span>Copied</span>
                </motion.div>
              ) : (
                <motion.div
                  key="copy"
                  initial={{ opacity: 0 }}
                  animate={{ opacity: 1 }}
                  exit={{ opacity: 0 }}
                  className="flex items-center gap-1"
                >
                  <Copy size={12} />
                  <span>Copy</span>
                </motion.div>
              )}
            </AnimatePresence>
          </motion.button>
        </div>
      </div>

      <div className="mt-4 font-mono text-xs leading-relaxed bg-[#101010] p-5 border border-[#2c2c2c] overflow-x-auto text-[#ebe6dd]">
        <pre>{commands[activeTab]}</pre>
      </div>

      <div className="flex flex-wrap items-center justify-between gap-4 mt-4 pt-4 border-t border-[#333333] text-xs font-mono text-[#828282]">
        <div>Prebuilt binaries for macOS, Linux, Windows</div>
        <div>Zero configuration</div>
        <div>No dependencies required</div>
      </div>
    </motion.div>
  );
}
