"use client";

import React, { useRef, useState, useEffect } from "react";
import Link from "next/link";
import { motion, useMotionValue, useSpring } from "framer-motion";
import { useCursor } from "@/components/providers/CursorProvider";

interface TactileButtonProps {
  children: React.ReactNode;
  href?: string;
  onClick?: () => void;
  variant?: "primary" | "ghost" | "dark" | "subtle";
  magnetic?: boolean;
  magneticStrength?: number;
  className?: string;
  target?: string;
  rel?: string;
  icon?: React.ReactNode;
  type?: "button" | "submit" | "reset";
  disabled?: boolean;
  ariaLabel?: string;
}

export function TactileButton({
  children,
  href,
  onClick,
  variant = "primary",
  magnetic = true,
  magneticStrength = 0.25,
  className = "",
  target,
  rel,
  icon,
  type = "button",
  disabled = false,
  ariaLabel,
}: TactileButtonProps) {
  const ref = useRef<HTMLDivElement>(null);
  const [isHovered, setIsHovered] = useState(false);
  const [isPointerDevice, setIsPointerDevice] = useState(true);
  const { setCursorVariant, resetCursor } = useCursor();

  const x = useMotionValue(0);
  const y = useMotionValue(0);

  const springConfig = { stiffness: 280, damping: 18, mass: 0.1 };
  const springX = useSpring(x, springConfig);
  const springY = useSpring(y, springConfig);

  useEffect(() => {
    if (typeof window !== "undefined") {
      setIsPointerDevice(window.matchMedia("(pointer: fine)").matches);
    }
  }, []);

  const handleMouseMove = (e: React.MouseEvent<HTMLDivElement>) => {
    if (!magnetic || !isPointerDevice || !ref.current || disabled) return;
    const { left, top, width, height } = ref.current.getBoundingClientRect();
    const centerX = left + width / 2;
    const centerY = top + height / 2;

    const deltaX = (e.clientX - centerX) * magneticStrength;
    const deltaY = (e.clientY - centerY) * magneticStrength;

    x.set(deltaX);
    y.set(deltaY);
  };

  const handleMouseEnter = () => {
    if (disabled) return;
    setIsHovered(true);
    setCursorVariant("hover");
  };

  const handleMouseLeave = () => {
    setIsHovered(false);
    x.set(0);
    y.set(0);
    resetCursor();
  };

  const variantStyles = {
    primary: "bg-[#ff682c] text-[#ffffff] border-[#ff682c] hover:bg-[#e0561f] hover:border-[#e0561f] shadow-sm",
    ghost: "bg-transparent text-[#202020] border-[#202020] hover:bg-[#f5f5f5]",
    dark: "bg-[#202020] text-[#ffffff] border-[#202020] hover:bg-[#4d4d4d]",
    subtle: "bg-[#efefef] text-[#202020] border-[#e8e8e8] hover:border-[#202020]",
  };

  const content = (
    <motion.div
      ref={ref}
      style={{
        borderRadius: "0px",
        x: springX,
        y: springY,
      }}
      onMouseMove={handleMouseMove}
      onMouseEnter={handleMouseEnter}
      onMouseLeave={handleMouseLeave}
      whileTap={{ scale: disabled ? 1 : 0.96 }}
      className={`relative inline-flex items-center justify-center gap-2 border px-5 py-2.5 min-h-[44px] font-heading text-xs font-semibold uppercase tracking-wider cursor-pointer select-none transition-colors duration-200 ${variantStyles[variant]} ${
        disabled ? "opacity-40 cursor-not-allowed pointer-events-none" : ""
      } ${className}`}
    >
      <span className="block leading-none font-medium whitespace-nowrap">
        {children}
      </span>

      {icon && (
        <motion.span
          animate={{
            x: isHovered ? 2 : 0,
            y: isHovered ? -1 : 0,
          }}
          transition={{ duration: 0.18, ease: "easeOut" }}
          className="flex-shrink-0"
        >
          {icon}
        </motion.span>
      )}
    </motion.div>
  );

  if (href) {
    if (href.startsWith("http")) {
      return (
        <a
          href={href}
          target={target || "_blank"}
          rel={rel || "noopener noreferrer"}
          aria-label={ariaLabel}
          className="inline-block focus:outline-none"
        >
          {content}
        </a>
      );
    }
    return (
      <Link href={href} aria-label={ariaLabel} className="inline-block focus:outline-none">
        {content}
      </Link>
    );
  }

  return (
    <button
      onClick={onClick}
      type={type}
      disabled={disabled}
      aria-label={ariaLabel}
      className="inline-block bg-transparent border-0 p-0 focus:outline-none"
    >
      {content}
    </button>
  );
}
