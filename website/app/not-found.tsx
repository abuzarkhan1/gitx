import React from "react";
import Link from "next/link";
import { ArrowLeft, Terminal, Compass } from "lucide-react";
import { Navbar } from "@/components/Navbar";
import { Footer } from "@/components/Footer";

export default function NotFound() {
  return (
    <div className="min-h-screen bg-[#08080a] text-white flex flex-col justify-between selection:bg-white selection:text-black">
      <Navbar />

      <main className="flex-1 flex flex-col items-center justify-center p-6 text-center max-w-xl mx-auto my-auto relative z-10 pt-28">
        {/* Custom Git Archaeology 404 SVG Artwork */}
        <div className="relative w-56 h-56 sm:w-64 sm:h-64 mb-6 flex items-center justify-center">
          <svg
            viewBox="0 0 300 300"
            className="w-full h-full drop-shadow-[0_0_35px_rgba(255,255,255,0.08)]"
            fill="none"
            xmlns="http://www.w3.org/2000/svg"
          >
            <defs>
              <radialGradient id="gitx-404-glow" cx="50%" cy="50%" r="50%">
                <stop offset="0%" stopColor="#ffffff" stopOpacity="0.1" />
                <stop offset="100%" stopColor="transparent" stopOpacity="0" />
              </radialGradient>
            </defs>

            {/* Glowing Core */}
            <circle cx="150" cy="150" r="130" fill="url(#gitx-404-glow)" />

            {/* Tree Branch Commit Graph Lines */}
            <path
              d="M150 40 L150 260"
              stroke="rgba(255,255,255,0.15)"
              strokeWidth="2"
              strokeDasharray="4 4"
            />
            <path
              d="M150 110 C200 110, 230 140, 230 190 L230 260"
              stroke="rgba(255,255,255,0.12)"
              strokeWidth="2"
              strokeDasharray="6 6"
            />
            <path
              d="M150 160 C100 160, 70 190, 70 230 L70 260"
              stroke="rgba(255,255,255,0.12)"
              strokeWidth="2"
              strokeDasharray="6 6"
            />

            {/* Commit Nodes */}
            <circle cx="150" cy="70" r="7" fill="#111115" stroke="rgba(255,255,255,0.4)" strokeWidth="2" />
            <circle cx="150" cy="110" r="8" fill="#111115" stroke="#ffffff" strokeWidth="2.5" />
            <circle cx="230" cy="190" r="6" fill="#111115" stroke="rgba(255,255,255,0.3)" strokeWidth="2" />
            <circle cx="70" cy="230" r="6" fill="#111115" stroke="rgba(255,255,255,0.3)" strokeWidth="2" />

            {/* Broken Target Head / Dangling Ref */}
            <g transform="translate(130, 180)">
              <rect x="0" y="0" width="40" height="40" rx="10" fill="#18181f" stroke="#f43f5e" strokeWidth="2" />
              <line x1="12" y1="12" x2="28" y2="28" stroke="#f43f5e" strokeWidth="2.5" strokeLinecap="round" />
              <line x1="28" y1="12" x2="12" y2="28" stroke="#f43f5e" strokeWidth="2.5" strokeLinecap="round" />
            </g>

            {/* Orbiting Scan Dot */}
            <g className="animate-[spin_6s_linear_infinite]" style={{ transformOrigin: "150px 150px" }}>
              <circle cx="250" cy="150" r="3" fill="#ffffff" />
            </g>

            {/* Mono status */}
            <text
              x="150"
              y="288"
              textAnchor="middle"
              fill="rgba(255,255,255,0.35)"
              fontSize="11"
              fontFamily="monospace"
              letterSpacing="2"
            >
              REF_HEAD_DANGLING_0x404
            </text>
          </svg>
        </div>

        {/* Status Pill */}
        <div className="inline-flex items-center gap-2 px-3 py-1 rounded-full text-xs font-mono text-amber-300 bg-amber-500/10 border border-amber-500/20 mb-4">
          <span className="w-1.5 h-1.5 rounded-full bg-amber-400 animate-ping" />
          <span>COMMIT_OR_ROUTE_NOT_FOUND</span>
        </div>

        {/* Heading */}
        <h1 className="text-3xl sm:text-5xl font-bold tracking-tight text-white mb-4 font-sans">
          Dangling <span className="font-serif italic font-normal text-white/90">Reference</span>
        </h1>

        <p className="text-sm sm:text-base text-zinc-400 max-w-md mx-auto mb-8 leading-relaxed font-sans">
          The requested page or repository view could not be located. It may have been rebased, deleted, or never committed to origin.
        </p>

        {/* Actions */}
        <div className="flex flex-col sm:flex-row items-center justify-center gap-3 w-full">
          <Link
            href="/"
            className="btn-primary w-full sm:w-auto px-6 py-3"
          >
            <ArrowLeft className="w-4 h-4" />
            <span>Return to Repository</span>
          </Link>
          <a
            href="https://github.com/abuzarkhan1/gitx"
            target="_blank"
            rel="noopener noreferrer"
            className="btn-secondary w-full sm:w-auto px-6 py-3"
          >
            <Terminal className="w-4 h-4" />
            <span>GitHub Releases</span>
          </a>
        </div>
      </main>

      <Footer />
    </div>
  );
}
