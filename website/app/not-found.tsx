"use client";

import React from "react";
import Link from "next/link";
import { ArrowLeft } from "lucide-react";
import { Navbar } from "@/components/ui/Navbar";
import { Footer } from "@/components/ui/Footer";

export default function NotFound() {
  return (
    <div className="min-h-screen bg-[#ffffff] text-[#202020] flex flex-col justify-between">
      <Navbar />

      <main id="main-content" className="flex-1 flex flex-col items-center justify-center p-6 text-center max-w-xl mx-auto my-auto pt-36 pb-24">
        <div className="inline-block px-3 py-1 bg-[#efefef] text-[#816729] font-mono text-xs border border-[#e8e8e8] mb-4">
          STATUS 404 · OBJECT_NOT_FOUND
        </div>

        <h1 className="font-heading text-4xl sm:text-5xl text-[#202020] mb-4 tracking-[-0.02em]">
          Unmapped Commit or Tree
        </h1>

        <p className="text-base text-[#4d4d4d] leading-relaxed mb-8">
          The requested route or Git object could not be located. It may have moved, been pruned, or deleted from the index.
        </p>

        <Link
          href="/"
          className="btn-primary inline-flex items-center gap-2"
        >
          <ArrowLeft size={14} />
          <span>Return to Observatory</span>
        </Link>
      </main>

      <Footer />
    </div>
  );
}
