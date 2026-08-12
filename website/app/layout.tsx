import type { Metadata } from "next";
import Link from "next/link";
import "./globals.css";

export const metadata: Metadata = {
  title: {
    default: "GitX — terminal-native Git intelligence",
    template: "%s — GitX",
  },
  description:
    "GitX turns a Git repository's history, structure, changes, ownership, branches, dependencies, and recoverable work into a fast, interactive, explainable terminal experience. No network. No accounts. No AI.",
  metadataBase: new URL("https://github.com/abuzarkhan1/gitx"),
  openGraph: {
    title: "GitX — terminal-native Git intelligence",
    description:
      "Local-first, terminal-native Git repository intelligence and code archaeology.",
    type: "website",
  },
};

const NAV = [
  { href: "/", label: "~/", title: "home" },
  { href: "/about", label: "~/about", title: "about" },
  { href: "/contact", label: "~/contact", title: "contact" },
];

export default function RootLayout({
  children,
}: Readonly<{ children: React.ReactNode }>) {
  return (
    <html lang="en">
      <body>
        <div className="site-shell">
          <header className="term-bar">
            <span className="dots" aria-hidden="true">
              <span className="dot dot-green" />
              <span className="dot dot-amber" />
              <span className="dot dot-off" />
            </span>
            <span className="title">
              <b>gitx</b>
              <span className="status"> — zsh — 88×24</span>
            </span>
            <nav className="term-nav" aria-label="Sections">
              {NAV.map((item) => (
                <Link key={item.href} href={item.href} title={item.title}>
                  {item.label}
                </Link>
              ))}
            </nav>
          </header>

          <main className="term-main">{children}</main>

          <footer className="term-footer">
            <span>
              <span className="ok">$</span> gitx --version
              <span className="faint"> → </span>
              <span className="ok">GitX 0.1.0</span>
            </span>
            <span>
              <span className="ok">$</span> git status
              <span className="faint"> → </span>
              <span className="ok">on main · clean</span>
            </span>
          </footer>
        </div>
      </body>
    </html>
  );
}
