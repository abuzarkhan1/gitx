"use client";

import React, { useState } from "react";
import { Plus, Minus } from "lucide-react";
import { motion, AnimatePresence } from "framer-motion";

interface FaqItem {
  q: string;
  a: string;
}

export function FaqAccordion() {
  const [openIndex, setOpenIndex] = useState<number | null>(0);

  const faqs: FaqItem[] = [
    {
      q: "How does GitX achieve sub-second query performance?",
      a: "GitX parses raw Git objects into a local, lightweight SQLite index on first run. Subsequent queries read directly from cached tables with WAL mode, running in single-digit milliseconds.",
    },
    {
      q: "Does GitX require any network connection or cloud account?",
      a: "No. GitX is 100% local, offline, and private. Zero telemetry, zero AI, and zero external network calls. All analysis runs directly on your machine against your local .git directory.",
    },
    {
      q: "How is the maintenance risk score computed?",
      a: "Risk is a deterministic linear combination of 5 measurable signals: churn volume (30%), commit frequency (25%), bug-fix keywords in history (20%), ownership concentration / bus factor (15%), and AST complexity (10%). Every score exposes its raw mathematical inputs.",
    },
    {
      q: "How does rename-following archaeology work?",
      a: "GitX tracks file lineage using gix tree-diff similarity heuristics across git renames, directory modularizations, and file splits, preserving complete commit history across multiple years.",
    },
    {
      q: "Can I use GitX in CI/CD pipelines?",
      a: "Yes. Every analytical command supports machine-readable output: `gitx --json hotspots` and `gitx --csv contributors`, returning stable JSON schemas for automated linting or CI quality gates.",
    },
  ];

  return (
    <div className="divide-y divide-[#e8e8e8] border-y border-[#e8e8e8]">
      {faqs.map((faq, index) => {
        const isOpen = openIndex === index;
        return (
          <div key={index} className="transition-colors">
            <button
              onClick={() => setOpenIndex(isOpen ? null : index)}
              className="w-full py-5 text-left flex items-center justify-between gap-4 select-none hover:text-[#ff682c] transition-colors"
              aria-expanded={isOpen}
              aria-controls={`faq-answer-${index}`}
              id={`faq-question-${index}`}
            >
              <h4 className="font-heading text-lg text-[#202020] tracking-[-0.02em]">
                {faq.q}
              </h4>
              <div className="p-1 text-[#202020] flex-shrink-0">
                {isOpen ? <Minus size={15} /> : <Plus size={15} />}
              </div>
            </button>

            <AnimatePresence initial={false}>
              {isOpen && (
                <motion.div
                  id={`faq-answer-${index}`}
                  role="region"
                  aria-labelledby={`faq-question-${index}`}
                  initial={{ height: 0, opacity: 0 }}
                  animate={{ height: "auto", opacity: 1 }}
                  exit={{ height: 0, opacity: 0 }}
                  transition={{ duration: 0.2, ease: [0.76, 0, 0.24, 1] }}
                  className="overflow-hidden"
                >
                  <p className="pb-5 text-sm text-[#4d4d4d] leading-relaxed">
                    {faq.a}
                  </p>
                </motion.div>
              )}
            </AnimatePresence>
          </div>
        );
      })}
    </div>
  );
}
