import type { Metadata } from "next";
import Link from "next/link";
import TerminalWindow from "@/components/TerminalWindow";

export const metadata: Metadata = { title: "about" };

const CRATES = `crates/
├── gitx-cli/        commands · dispatch · exit codes
├── gitx-core/       domain types · config · result types
├── gitx-git/        objects · refs · diffs · reflog  (gix wrapper)
├── gitx-index/      initial + incremental scans · change detection
├── gitx-storage/    SQLite provider · migrations · transactions
├── gitx-history/    timeline · blame · lineage · renames
├── gitx-analysis/   metrics · hotspots · ownership · risk
├── gitx-graph/      module graph · architecture dependencies
├── gitx-search/     full-text search · symbols · filters
├── gitx-services/   application facade (no business logic)
└── gitx-tui/        views · keymaps · charts · themes

migrations/          versioned SQLite schema migrations
tests/               fixture repos · integration · snapshots
benches/             criterion benchmark suites`;

const VALUES = [
  "no network — everything runs against your local .git",
  "no accounts — no signup, no cloud, no telemetry",
  "no AI — every score is a deterministic formula over raw git signals",
];

export default function AboutPage() {
  return (
    <>
      <div className="line">
        <span className="faint">$ </span>
        <span className="cmd">cat ABOUT.md</span>
      </div>
      <h1 className="block-head">about</h1>

      <TerminalWindow title="about — cat ABOUT.md" right="README.md">
        <div className="line">
          <span className="prompt">
            <span className="user">user</span>
            <span className="host">@gitx</span>
            <span className="path">:~$</span>
          </span>{" "}
          <span className="cmd">cat ABOUT.md</span>
        </div>
        <pre className="out mt">
{`# GitX

Local-first, terminal-native Git repository intelligence and
code archaeology.

GitX turns a Git repository's history, structure, changes,
ownership, branches, dependencies, and recoverable work into a
fast, interactive, explainable terminal experience.

It is:
  - explainable, not black-box
  - local and private
  - deterministic (safe for CI)
  - built for archaeology
  - fast at scale`}
        </pre>
      </TerminalWindow>

      <section className="block" aria-labelledby="crates-head">
        <h2 className="block-head" id="crates-head">
          the workspace
        </h2>
        <TerminalWindow title="tree crates/ — 11 crates" right="structure">
          <div className="line">
            <span className="prompt">
              <span className="user">user</span>
              <span className="host">@gitx</span>
              <span className="path">:~$</span>
            </span>{" "}
            <span className="cmd">tree crates/ -L 1</span>
          </div>
          <pre className="out mt">{CRATES}</pre>
          <div className="line mt dim">
            <span>
              analysis lives in <span className="amber">gitx-analysis</span>,
              graph primitives in <span className="amber">gitx-graph</span>,
              and the CLI and TUI delegate through{" "}
              <span className="amber">gitx-services</span>. No TUI component
              traverses git objects directly.
            </span>
          </div>
        </TerminalWindow>
      </section>

      <div className="grid grid-2">
        <section className="block" aria-labelledby="pipeline-head">
          <h2 className="block-head" id="pipeline-head">
            the pipeline
          </h2>
          <TerminalWindow title="gitx index — data flow" right="docs/04">
            <pre className="out">
{`repository
   │  discover
   ▼
read refs
   │  traverse
   ▼
normalize domain entities
   │  persist
   ▼
build derived indexes
   │  analyze
   ▼
run requested analyses   →   sqlite index in .git/gitx

first run:   one full pass, then cached
every run:   incremental, O(new commits)`}
            </pre>
            <div className="line dim">
              <span>
                hot queries read the local SQLite index in milliseconds — the
                same index that powers{" "}
                <Link href="/contact">every command</Link> and the dashboard.
              </span>
            </div>
          </TerminalWindow>
        </section>

        <section className="block" aria-labelledby="credit-head">
          <h2 className="block-head" id="credit-head">
            credits
          </h2>
          <TerminalWindow title="gitx --credit" right="abuzar">
            <div className="line">
              <span className="prompt">
                <span className="user">user</span>
                <span className="host">@gitx</span>
                <span className="path">:~$</span>
              </span>{" "}
              <span className="cmd">gitx --credit</span>
            </div>
            <ul className="kvlist mt">
              <li>
                <span className="k">project</span>
                <span className="a">→</span>
                <span className="out">GitX</span>
              </li>
              <li>
                <span className="k">author</span>
                <span className="a">→</span>
                <span className="credit-name">Abuzar Khan</span>
              </li>
              <li>
                <span className="k">role</span>
                <span className="a">→</span>
                <span className="out">creator &amp; maintainer</span>
              </li>
              <li>
                <span className="k">license</span>
                <span className="a">→</span>
                <span className="out">MIT</span>
              </li>
              <li>
                <span className="k">github</span>
                <span className="a">→</span>
                <a
                  href="https://github.com/abuzarkhan1"
                  target="_blank"
                  rel="noopener noreferrer"
                >
                  github.com/abuzarkhan1
                </a>
              </li>
            </ul>
            <div className="line mt dim">
              <span>© 2026 Abuzar Khan · MIT licensed</span>
            </div>
          </TerminalWindow>
        </section>
      </div>

      <section className="block" aria-labelledby="values-head">
        <h2 className="block-head" id="values-head">
          the rules
        </h2>
        <TerminalWindow title="gitx --values" right="hard constraints">
          <ul className="features">
            {VALUES.map((v) => (
              <li key={v}>
                <span className="out">{v}</span>
              </li>
            ))}
          </ul>
        </TerminalWindow>
      </section>
    </>
  );
}
