import type { Metadata } from "next";
import "./globals.css";
import { CursorProvider } from "@/components/providers/CursorProvider";
import { CustomCursor } from "@/components/motion/CustomCursor";

export const metadata: Metadata = {
  title: "GitX — Local-First Git Repository Intelligence & Code Archaeology",
  description: "Terminal-native Git repository intelligence in Rust. Explore commit history, maintenance risk hotspots, ownership concentration, rename lineage, and recoverable work with sub-second SQLite indexing. 100% local, zero network, zero AI.",
  keywords: [
    "GitX",
    "Git CLI",
    "Code Archaeology",
    "Git Repository Intelligence",
    "Rust CLI",
    "Ratatui TUI",
    "Git Hotspots",
    "Git Reflog Recovery",
    "Git Lineage",
    "Local Git Analytics"
  ],
  authors: [{ name: "GitX Contributors" }],
  creator: "GitX Project",
  metadataBase: new URL("https://gitx.dev"),
  openGraph: {
    title: "GitX — Local-First Git Repository Intelligence",
    description: "Explainable Git history, maintenance hotspots, ownership, and lost work in an interactive terminal experience.",
    url: "https://gitx.dev",
    siteName: "GitX Observatory",
    locale: "en_US",
    type: "website",
  },
  twitter: {
    card: "summary_large_image",
    title: "GitX — Terminal-Native Git Repository Intelligence",
    description: "Explore Git history, hotspots, ownership, and recoverable work in Rust. 100% local, zero network, zero AI.",
  },
  robots: {
    index: true,
    follow: true,
  },
};

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html lang="en">
      <head>
        <meta name="viewport" content="width=device-width, initial-scale=1, maximum-scale=5" />
        <meta name="theme-color" content="#ffffff" />
      </head>
      <body className="antialiased selection:bg-[#ff682c] selection:text-white">
        <a href="#main-content" className="skip-to-content">
          Skip to main content
        </a>
        <CursorProvider>
          <CustomCursor />
          {children}
        </CursorProvider>
      </body>
    </html>
  );
}
