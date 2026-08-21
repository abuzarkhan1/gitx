"use client";

import React, { useState } from "react";
import { Users, FileCode, ArrowRight, GitCommit, Check } from "lucide-react";
import { useCursor } from "@/components/providers/CursorProvider";

export function LineageArchaeologyVisualizer() {
  const [activeStep, setActiveStep] = useState(2);
  const { setCursorVariant, resetCursor } = useCursor();

  const lineageNodes = [
    {
      date: "2021-11-04",
      hash: "8f1a02d",
      path: "src/utils/tokenizer.rs",
      action: "Created in initial commit",
      author: "Alex Rivera",
      churn: "+340 / -0",
      similarity: "100%",
      diffSnippet: `+pub struct Tokenizer {
+    raw_stream: Vec<u8>,
+    position: usize,
+}
+impl Tokenizer {
+    pub fn next_token(&mut self) -> Option<Token> { ... }
+}`,
    },
    {
      date: "2023-04-19",
      hash: "2b9e41a",
      path: "src/lexer/parse_tokens.rs",
      action: "Renamed during core tokenizer rewrite",
      author: "Elena Rostova",
      churn: "+180 / -95",
      similarity: "92% match",
      diffSnippet: `// Renamed from src/utils/tokenizer.rs (92% content similarity)
-pub struct Tokenizer
+pub struct LexerTokenizer {
+    cursor: SpanCursor,
 }`,
    },
    {
      date: "2025-01-14",
      hash: "e4d79c0",
      path: "crates/gitx-core/src/ast.rs",
      action: "Extracted into standalone core workspace crate",
      author: "Marcus Vance",
      churn: "+620 / -140",
      similarity: "88% match",
      diffSnippet: `// Extracted into crates/gitx-core crate
+#[derive(Debug, Clone, PartialEq)]
+pub struct AstNode {
+    pub kind: NodeKind,
+    pub span: TextSpan,
+}`,
    },
  ];

  const current = lineageNodes[activeStep];

  return (
    <div
      className="w-full bg-[#ffffff] border border-[#e8e8e8] p-6 md:p-8"
      style={{ borderRadius: "0px" }}
      onMouseEnter={() => setCursorVariant("explore", "LINEAGE")}
      onMouseLeave={resetCursor}
    >
      <div className="flex flex-col lg:flex-row lg:items-center justify-between gap-4 pb-6 border-b border-[#e8e8e8]">
        <div>
          <h3 className="font-heading text-2xl md:text-3xl text-[#202020] tracking-[-0.02em]">
            Rename-Following File Archaeology
          </h3>
        </div>

        <div className="text-xs font-mono text-[#828282]">
          Target: <span className="text-[#202020] font-semibold">crates/gitx-core/src/ast.rs</span>
        </div>
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-12 gap-8 mt-8 items-start">
        {/* Left: Timeline Steps */}
        <div className="lg:col-span-6 space-y-4">
          <div className="text-xs font-mono text-[#828282] uppercase mb-2">
            File Evolution History (3 Stages)
          </div>

          <div className="relative pl-6 sm:pl-8 border-l-2 border-[#e8e8e8] space-y-4">
            {lineageNodes.map((node, i) => (
              <div
                key={i}
                onClick={() => setActiveStep(i)}
                className={`relative cursor-pointer transition-all p-4 border ${
                  activeStep === i
                    ? "bg-[#f5f5f5] border-[#202020]"
                    : "bg-[#ffffff] border-[#e8e8e8] hover:border-[#828282]"
                }`}
                style={{ borderRadius: "0px" }}
              >
                {/* Bullet Node */}
                <div
                  className={`absolute -left-[31px] sm:-left-[39px] top-5 w-4 h-4 rounded-full border-2 border-white transition-colors ${
                    activeStep === i ? "bg-[#ff682c]" : "bg-[#828282]"
                  }`}
                />

                <div className="flex flex-wrap items-center justify-between gap-2 pb-2 border-b border-[#e8e8e8]">
                  <div className="flex items-center gap-2">
                    <span className="font-mono text-xs font-bold text-[#ff682c]">{node.hash}</span>
                    <span className="text-xs text-[#828282] font-mono">{node.date}</span>
                  </div>
                  <span className="px-2 py-0.5 bg-[#efefef] text-[10px] font-mono text-[#816729]">
                    {node.similarity}
                  </span>
                </div>

                <div className="mt-2.5">
                  <div className="font-mono text-sm text-[#202020] font-semibold">{node.path}</div>
                  <div className="text-xs text-[#4d4d4d] mt-0.5">{node.action}</div>
                </div>

                <div className="flex flex-wrap items-center justify-between gap-4 mt-2.5 pt-2 text-xs font-mono text-[#828282]">
                  <div className="flex items-center gap-1.5">
                    <Users size={12} />
                    <span>{node.author}</span>
                  </div>
                  <div>{node.churn} lines</div>
                </div>
              </div>
            ))}
          </div>
        </div>

        {/* Right: Selected Node Diff & Blame Inspection */}
        <div className="lg:col-span-6 bg-[#202020] text-[#ffffff] p-5 border border-[#333333] space-y-4" style={{ borderRadius: "0px" }}>
          <div className="flex justify-between items-center pb-3 border-b border-[#333333] text-xs font-mono">
            <span className="text-[#828282]">COMMIT DIFF INSPECTION</span>
            <span className="text-[#ff682c] font-bold">{current.hash} ({current.date})</span>
          </div>

          <div>
            <div className="text-xs font-mono text-[#828282]">PATH:</div>
            <div className="text-sm font-mono text-[#ffffff] font-semibold">{current.path}</div>
            <div className="text-xs text-[#828282] mt-1">Author: <span className="text-[#ebe6dd]">{current.author}</span></div>
          </div>

          <div className="space-y-1.5">
            <div className="text-xs font-mono text-[#828282]">CODE DIFF PREVIEW:</div>
            <pre className="bg-[#141414] p-3 text-xs font-mono text-[#27c93f] overflow-x-auto leading-relaxed border border-[#333333]">
              <code>{current.diffSnippet}</code>
            </pre>
          </div>

          <div className="pt-2 border-t border-[#333333] flex justify-between items-center text-xs font-mono text-[#828282]">
            <span>Similarity heuristic: {current.similarity}</span>
            <span className="text-[#ff682c]">gitx lineage</span>
          </div>
        </div>
      </div>
    </div>
  );
}
