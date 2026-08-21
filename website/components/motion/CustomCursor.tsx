"use client";

import React, { useEffect, useRef, useState } from "react";
import { motion, useMotionValue, useSpring, AnimatePresence } from "framer-motion";
import { useCursor } from "@/components/providers/CursorProvider";

export function CustomCursor() {
  const { variant, cursorText, cursorTheme } = useCursor();
  const dotRef = useRef<HTMLDivElement>(null);
  const [isVisible, setIsVisible] = useState(false);

  const mouseX = useMotionValue(-100);
  const mouseY = useMotionValue(-100);

  // High-frequency responsive spring physics
  const springX = useSpring(mouseX, { stiffness: 450, damping: 30, mass: 0.35 });
  const springY = useSpring(mouseY, { stiffness: 450, damping: 30, mass: 0.35 });

  useEffect(() => {
    const onMouseMove = (e: MouseEvent) => {
      if (!isVisible) setIsVisible(true);
      mouseX.set(e.clientX);
      mouseY.set(e.clientY);

      if (dotRef.current) {
        dotRef.current.style.transform = `translate3d(${e.clientX}px, ${e.clientY}px, 0)`;
      }
    };

    const onMouseLeave = () => setIsVisible(false);
    const onMouseEnter = () => setIsVisible(true);

    window.addEventListener("mousemove", onMouseMove, { passive: true });
    document.addEventListener("mouseleave", onMouseLeave);
    document.addEventListener("mouseenter", onMouseEnter);

    return () => {
      window.removeEventListener("mousemove", onMouseMove);
      document.removeEventListener("mouseleave", onMouseLeave);
      document.removeEventListener("mouseenter", onMouseEnter);
    };
  }, [mouseX, mouseY, isVisible]);

  if (variant === "hidden" || !isVisible) return null;

  const hasPillText = Boolean(cursorText);
  const isDarkSurface = cursorTheme === "dark";

  return (
    <>
      {/* 1. Precision Center Focal Dot */}
      <div
        ref={dotRef}
        className={`pointer-events-none fixed left-0 top-0 z-[9999] -ml-1 -mt-1 h-2 w-2 rounded-full transition-all duration-200 hidden md:block ${
          isDarkSurface ? "bg-[#ffffff]" : "bg-[#202020]"
        } ${hasPillText || variant === "hover" ? "scale-0 opacity-0" : "scale-100 opacity-100"}`}
        style={{ willChange: "transform" }}
      />

      {/* 2. Adaptive Follower Capsule */}
      <motion.div
        className="pointer-events-none fixed left-0 top-0 z-[9998] hidden md:flex items-center justify-center rounded-full select-none"
        style={{
          x: springX,
          y: springY,
          translateX: "-50%",
          translateY: "-50%",
          willChange: "transform, width, height",
        }}
        animate={{
          width: hasPillText ? "auto" : variant === "hover" ? 48 : variant === "magnetic" ? 40 : 28,
          height: hasPillText ? 26 : variant === "hover" ? 48 : variant === "magnetic" ? 40 : 28,
          paddingLeft: hasPillText ? 12 : 0,
          paddingRight: hasPillText ? 12 : 0,
          backgroundColor: hasPillText
            ? "#202020"
            : isDarkSurface
            ? "rgba(255, 255, 255, 0.12)"
            : "rgba(32, 32, 32, 0.05)",
          borderColor: hasPillText
            ? "#ff682c"
            : isDarkSurface
            ? "rgba(255, 255, 255, 0.35)"
            : "rgba(32, 32, 32, 0.2)",
          borderWidth: 1,
          scale: variant === "magnetic" ? 1.2 : 1,
        }}
        transition={{ type: "spring", stiffness: 420, damping: 28 }}
      >
        <AnimatePresence mode="wait">
          {cursorText && (
            <motion.span
              key={cursorText}
              initial={{ opacity: 0, scale: 0.8 }}
              animate={{ opacity: 1, scale: 1 }}
              exit={{ opacity: 0, scale: 0.8 }}
              transition={{ duration: 0.15 }}
              className="text-[10px] font-mono font-semibold tracking-wider text-[#ffffff] uppercase whitespace-nowrap"
            >
              {cursorText}
            </motion.span>
          )}
        </AnimatePresence>
      </motion.div>
    </>
  );
}
