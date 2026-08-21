"use client";

import React, { useState } from "react";
import Link from "next/link";
import { Menu, X, ArrowUpRight } from "lucide-react";
import { motion, AnimatePresence } from "framer-motion";
import { GithubIcon } from "@/components/ui/GithubIcon";
import { TactileButton } from "@/components/motion/TactileButton";
import { useCursor } from "@/components/providers/CursorProvider";

export function Navbar() {
  const [mobileMenuOpen, setMobileMenuOpen] = useState(false);
  const [hoveredNav, setHoveredNav] = useState<string | null>(null);
  const { setCursorVariant, resetCursor } = useCursor();

  const navLinks = [
    { label: "Dashboard", href: "/#tui" },
    { label: "Hotspots", href: "/#hotspots" },
    { label: "Archaeology", href: "/#lineage" },
    { label: "Recovery", href: "/#recovery" },
    { label: "Benchmarks", href: "/#benchmarks" },
    { label: "Install", href: "/#install" },
  ];

  return (
    <header className="fixed top-0 left-0 right-0 z-50 py-4 px-4 md:px-8 pointer-events-none">
      <div className="max-w-[1200px] mx-auto flex items-center justify-between pointer-events-auto">
        {/* Brand Wordmark */}
        <Link
          href="/"
          className="flex items-center gap-2 text-[#202020] font-heading text-lg md:text-xl font-medium tracking-[-0.02em] group"
          onMouseEnter={() => setCursorVariant("hover")}
          onMouseLeave={resetCursor}
          aria-label="GitX Observatory Home"
        >
          <span>GITX</span>
        </Link>

        {/* Center Floating Pill Navigation Container */}
        <nav
          aria-label="Primary Navigation"
          className="hidden md:flex items-center gap-1 bg-[#ffffff]/90 backdrop-blur-md border border-[#e8e8e8] px-4 py-1.5 shadow-sm"
          style={{ borderRadius: "200px" }}
        >
          {navLinks.map((link) => (
            <Link
              key={link.href}
              href={link.href}
              onMouseEnter={() => setHoveredNav(link.href)}
              onMouseLeave={() => setHoveredNav(null)}
              className="relative px-3.5 py-1 text-xs font-heading font-medium text-[#4d4d4d] transition-colors z-10 hover:text-[#202020] tracking-[-0.01em]"
            >
              {hoveredNav === link.href && (
                <motion.div
                  layoutId="navbarHoverPill"
                  className="absolute inset-0 bg-[#f5f5f5] rounded-full z-[-1] border border-[#e8e8e8]"
                  transition={{ type: "spring", stiffness: 450, damping: 32 }}
                />
              )}
              {link.label}
            </Link>
          ))}
        </nav>

        {/* Right CTA Cluster */}
        <div className="flex items-center gap-3">
          <a
            href="https://github.com/abuzarkhan1/gitx"
            target="_blank"
            rel="noopener noreferrer"
            className="hidden sm:inline-flex items-center gap-1.5 text-xs font-mono font-medium text-[#4d4d4d] hover:text-[#202020] hover:border-[#202020] transition-colors px-3 py-1.5 bg-[#ffffff] border border-[#e8e8e8]"
            style={{ borderRadius: "0px" }}
            aria-label="GitHub repository"
          >
            <GithubIcon size={12} />
            <span>GitHub</span>
          </a>

          <TactileButton
            href="/#install"
            variant="primary"
            icon={<ArrowUpRight size={13} />}
          >
            Get GitX
          </TactileButton>

          {/* Mobile Menu Trigger */}
          <button
            onClick={() => setMobileMenuOpen(!mobileMenuOpen)}
            className="md:hidden min-w-[44px] min-h-[44px] flex items-center justify-center p-2.5 text-[#202020] bg-[#ffffff] border border-[#e8e8e8]"
            style={{ borderRadius: "0px" }}
            aria-expanded={mobileMenuOpen}
            aria-label="Toggle navigation drawer"
          >
            {mobileMenuOpen ? <X size={18} /> : <Menu size={18} />}
          </button>
        </div>
      </div>

      {/* Animated Mobile Drawer */}
      <AnimatePresence>
        {mobileMenuOpen && (
          <motion.div
            initial={{ opacity: 0, y: -10 }}
            animate={{ opacity: 1, y: 0 }}
            exit={{ opacity: 0, y: -10 }}
            transition={{ duration: 0.2, ease: [0.76, 0, 0.24, 1] }}
            className="md:hidden mt-3 mx-2 p-6 bg-[#ffffff] border border-[#e8e8e8] pointer-events-auto space-y-4 shadow-xl backdrop-blur-md"
            style={{ borderRadius: "0px" }}
          >
            {navLinks.map((link) => (
              <Link
                key={link.href}
                href={link.href}
                onClick={() => setMobileMenuOpen(false)}
                className="flex items-center min-h-[44px] py-2 px-1 text-base font-heading font-medium text-[#202020] hover:text-[#ff682c] transition-colors"
              >
                {link.label}
              </Link>
            ))}
            <div className="pt-3 border-t border-[#e8e8e8] flex items-center justify-between">
              <Link
                href="/about"
                onClick={() => setMobileMenuOpen(false)}
                className="text-sm font-heading text-[#4d4d4d]"
              >
                Architecture &amp; Docs
              </Link>
              <Link
                href="/contact"
                onClick={() => setMobileMenuOpen(false)}
                className="text-sm font-heading text-[#4d4d4d]"
              >
                Issues &amp; RFCs
              </Link>
            </div>
          </motion.div>
        )}
      </AnimatePresence>
    </header>
  );
}
