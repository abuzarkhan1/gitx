"use client";

import React, { useRef } from "react";
import { motion, useMotionValue, useSpring, useTransform } from "framer-motion";

interface PillarCardProps {
  icon: React.ReactNode;
  title: string;
  description: string;
  meta: string;
  accent?: string;
  delay?: number;
}

export function PillarCard({
  icon,
  title,
  description,
  meta,
  accent = "#ff682c",
  delay = 0,
}: PillarCardProps) {
  const cardRef = useRef<HTMLDivElement>(null);

  const mouseX = useMotionValue(0.5);
  const mouseY = useMotionValue(0.5);

  const rotateX = useSpring(useTransform(mouseY, [0, 1], [4, -4]), { stiffness: 300, damping: 20 });
  const rotateY = useSpring(useTransform(mouseX, [0, 1], [-4, 4]), { stiffness: 300, damping: 20 });

  const handleMouseMove = (e: React.MouseEvent<HTMLDivElement>) => {
    if (!cardRef.current) return;
    const rect = cardRef.current.getBoundingClientRect();
    const x = (e.clientX - rect.left) / rect.width;
    const y = (e.clientY - rect.top) / rect.height;
    mouseX.set(x);
    mouseY.set(y);
  };

  const handleMouseLeave = () => {
    mouseX.set(0.5);
    mouseY.set(0.5);
  };

  return (
    <motion.div
      ref={cardRef}
      initial={{ opacity: 0, y: 24 }}
      whileInView={{ opacity: 1, y: 0 }}
      viewport={{ once: true, margin: "-10%" }}
      transition={{ duration: 0.6, delay, ease: [0.25, 1, 0.5, 1] }}
      onMouseMove={handleMouseMove}
      onMouseLeave={handleMouseLeave}
      style={{
        rotateX,
        rotateY,
        transformStyle: "preserve-3d",
        borderRadius: "0px",
      }}
      whileHover={{ y: -6 }}
      className="relative bg-[#ffffff] p-8 border border-[#e8e8e8] space-y-4 hover:border-[#202020] transition-colors duration-300 group shadow-sm hover:shadow-md"
    >
      {/* Dynamic Cursor Spotlight Effect */}
      <motion.div
        className="absolute inset-0 pointer-events-none opacity-0 group-hover:opacity-100 transition-opacity duration-300"
        style={{
          background: `radial-gradient(400px circle at var(--mouse-x, 50%) var(--mouse-y, 50%), rgba(255, 104, 44, 0.04), transparent 80%)`,
        }}
      />

      <div className="w-10 h-10 bg-[#f5f5f5] text-[#ff682c] flex items-center justify-center border border-[#e8e8e8] group-hover:scale-105 group-hover:border-[#ff682c] transition-all duration-300">
        {icon}
      </div>

      <h3 className="font-heading text-xl text-[#202020] group-hover:text-[#ff682c] transition-colors duration-200">
        {title}
      </h3>

      <p className="text-sm text-[#4d4d4d] leading-relaxed">
        {description}
      </p>

      <div className="pt-2 border-t border-[#f0f0f0] flex items-center justify-between text-xs font-mono text-[#816729]">
        <span>{meta}</span>
        <span className="opacity-0 group-hover:opacity-100 text-[#ff682c] transition-opacity duration-200 font-bold">&rarr;</span>
      </div>
    </motion.div>
  );
}
