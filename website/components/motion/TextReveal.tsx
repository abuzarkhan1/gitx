"use client";

import React from "react";
import { motion, Variants } from "framer-motion";

interface TextRevealProps {
  children: string | React.ReactNode;
  as?: "h1" | "h2" | "h3" | "p" | "span" | "div";
  className?: string;
  splitBy?: "lines" | "words" | "chars";
  stagger?: number;
  delay?: number;
}

export function TextReveal({
  children,
  as: Tag = "h2",
  className = "",
  stagger = 0.04,
  delay = 0,
}: TextRevealProps) {
  if (typeof children !== "string") {
    return <Tag className={className}>{children}</Tag>;
  }

  const words = children.split(" ");

  const containerVariants: Variants = {
    hidden: { opacity: 0 },
    visible: {
      opacity: 1,
      transition: {
        staggerChildren: stagger,
        delayChildren: delay,
      },
    },
  };

  const itemVariants: Variants = {
    hidden: { opacity: 0, y: 14 },
    visible: {
      opacity: 1,
      y: 0,
      transition: { duration: 0.5, ease: [0.25, 1, 0.5, 1] as const },
    },
  };

  return (
    <Tag className={className}>
      <motion.span
        variants={containerVariants}
        initial="hidden"
        whileInView="visible"
        viewport={{ once: true, margin: "-10% 0px" }}
        className="inline-block"
      >
        {words.map((word, i) => (
          <span key={i} className="inline-block overflow-hidden mr-[0.28em] align-top">
            <motion.span variants={itemVariants} className="inline-block">
              {word}
            </motion.span>
          </span>
        ))}
      </motion.span>
    </Tag>
  );
}
