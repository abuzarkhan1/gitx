"use client";

import React, { createContext, useContext, useState } from "react";

export type CursorVariant = "default" | "hover" | "magnetic" | "text" | "listen" | "explore" | "clone" | "hidden";

interface CursorContextType {
  variant: CursorVariant;
  cursorText: string;
  cursorTheme: "light" | "dark";
  setCursorVariant: (variant: CursorVariant, text?: string, theme?: "light" | "dark") => void;
  setCursorTheme: (theme: "light" | "dark") => void;
  resetCursor: () => void;
}

const CursorContext = createContext<CursorContextType>({
  variant: "default",
  cursorText: "",
  cursorTheme: "light",
  setCursorVariant: () => {},
  setCursorTheme: () => {},
  resetCursor: () => {},
});

export const useCursor = () => useContext(CursorContext);

export function CursorProvider({ children }: { children: React.ReactNode }) {
  const [variant, setVariant] = useState<CursorVariant>("default");
  const [cursorText, setCursorText] = useState<string>("");
  const [cursorTheme, setCursorTheme] = useState<"light" | "dark">("light");

  const setCursorVariant = (newVariant: CursorVariant, text = "", theme: "light" | "dark" = "light") => {
    setVariant(newVariant);
    setCursorText(text);
    setCursorTheme(theme);
  };

  const resetCursor = () => {
    setVariant("default");
    setCursorText("");
    setCursorTheme("light");
  };

  return (
    <CursorContext.Provider value={{ variant, cursorText, cursorTheme, setCursorVariant, setCursorTheme, resetCursor }}>
      {children}
    </CursorContext.Provider>
  );
}
