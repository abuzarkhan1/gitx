import Link from "next/link";
import TerminalWindow from "@/components/TerminalWindow";
import Typewriter from "@/components/Typewriter";

const HELP = [
  ["stats", "repository statistics"],
  ["hotspots", "files ranked by maintenance risk"],
  ["health", "composite health score, six sub-scores"],
  ["ownership", "who owns what, and where it concentrates"],
  ["lineage", "the full life of a file, renames included"],
  ["blame", "line-level attribution"],
  ["branches", "divergence, age, shared files, staleness"],
  ["search", "full-text across commits, files, authors, tags"],
  ["recovery", "reflog, unreachable commits, dangling objects"],
  ["dependencies", "declared + lockfile-precise dependencies"],
  ["symbols", "functions and classes extracted from HEAD"],
  ["release diff", "what shipped between releases"],
];

const FEATURES = [
  [
    "Explainable, not black-box",
    "gitx risk src/main.rs prints the formula, the time window, and every input — change frequency, churn, bug-fix rate, ownership concentration, complexity. No hidden scoring.",
  ],
  [
    "Local and private",
    "Everything runs on your machine against your repository. Nothing leaves it. No accounts, no cloud, no telemetry.",
  ],
  [
    "Deterministic",
    "The same repository and configuration produce the same results, bit for bit — safe for CI and reproducible audits.",
  ],
  [
    "Built for archaeology",
    "Rename-following lineage, copy-source tracking, symbol history, and recovery of unreachable work are first-class features, not afterthoughts.",
  ],
  [
    "Fast at scale",
    "A persistent SQLite index means hot queries read in milliseconds, with phased lazy loading in the dashboard on large repositories.",
  ],
];

const DASHBOARD = `┌────────────────────────────────────────────────────────────┐
│ gitx v0.1.0     repo: ~/src/gitx                   main ●  │
├────────────────────────────┬───────────────────────────────────┤
│ HEALTH           87 / 100  │ HOTSPOTS                          │
│   balance      ▓▓▓▓▓▓▓░░░  │   src/analysis.rs        17 cmts  │
│   velocity     ▓▓▓▓▓▓▓▓░░  │   src/indexer.rs         12 cmts  │
│   complexity   ▓▓▓▓▓▓▓▓▓░  │   src/engine/pipeline.rs  9 cmts  │
├────────────────────────────┼───────────────────────────────────┤
│ ACTIVITY         last 30d  │ OWNERSHIP                         │
│   commits             412  │   abuzar         61%  ▓▓▓▓▓▓▓▓░░  │
│   authors               7  │   contributor    23%  ▓▓▓▓▓▓░░░░  │
│   churn            3.2k ++ │   others         16%  ▓▓▓▓░░░░░░  │
└────────────────────────────┴───────────────────────────────────┘`;

export default function Home() {
  return (
    <>
      {/* ------------------------------- hero -------------------------------- */}
      <div className="line">
        <span className="faint">$ whoami</span>
        <span className="out"> — gitx, a tool that reads your git history like an archaeologist.</span>
      </div>
      <h1 className="block-head">GitX</h1>
      <p className="hero-tagline">
        <span className="hl">Local-first, terminal-native</span> Git repository
        intelligence and code archaeology.{" "}
        <span className="hl-a">No network. No accounts. No AI.</span>
      </p>

      <TerminalWindow title="gitx — zsh — 88×24" right="●">
        <Typewriter command="gitx">
          <pre className="out">{DASHBOARD}</pre>
        </Typewriter>
      </TerminalWindow>

      {/* --------------------------- command help ---------------------------- */}
      <section className="block" aria-labelledby="help-head">
        <h2 className="block-head" id="help-head">
          gitx --help
        </h2>
        <TerminalWindow title="gitx --help" right="1.2.0">
          <div className="line">
            <span className="prompt">
              <span className="user">user</span>
              <span className="host">@gitx</span>
              <span className="path">:~$</span>
            </span>{" "}
            <span className="cmd">gitx --help</span>
          </div>
          <ul className="cmdlist mt">
            {HELP.map(([cmd, desc]) => (
              <li key={cmd}>
                <span className="cmd-name">gitx {cmd}</span>
                <span className="faint">{"\u00a0\u00a0\u00a0"}</span>
                <span className="cmd-desc"># {desc}</span>
              </li>
            ))}
          </ul>
          <div className="line mt">
            <span className="faint">
              every command also emits machine-readable output:
            </span>
          </div>
          <div className="line">
            <span className="out">$ gitx --json hotspots</span>
          </div>
          <div className="line">
            <span className="out">$ gitx --csv contributors</span>
          </div>
        </TerminalWindow>
      </section>

      {/* ------------------------------ how-to ------------------------------- */}
      <section className="block" aria-labelledby="howto-head">
        <h2 className="block-head" id="howto-head">
          how to use it
        </h2>
        <div className="steps">
          <div className="step span-all">
            <h3>
              <span className="step-num">01</span> — install
            </h3>
            <TerminalWindow title="zsh — install" right="one line">
              <div className="line">
                <span className="prompt">
                  <span className="user">user</span>
                  <span className="host">@gitx</span>
                  <span className="path">:~$</span>
                </span>{" "}
                <span className="cmd">
                  curl --proto '=https' --tlsv1.2 -LsSf \
                  https://github.com/abuzarkhan1/gitx/releases/latest/download/gitx-installer.sh
                  {" "}| sh
                </span>
              </div>
              <div className="line mt">
                <span className="dim">
                  # or: cargo install gitx-cli --locked
                </span>
              </div>
            </TerminalWindow>
          </div>

          <div className="step">
            <h3>
              <span className="step-num">02</span> — run it in any repository
            </h3>
            <TerminalWindow title="zsh — any repo" right="●">
              <div className="line">
                <span className="prompt">
                  <span className="user">user</span>
                  <span className="host">@gitx</span>
                  <span className="path">:~/src/my-project$</span>
                </span>{" "}
                <span className="cmd">gitx</span>
              </div>
              <div className="line mt dim">
                <span>
                  # first run builds a local SQLite index (one pass),
                  <br />
                  # afterwards it refreshes in milliseconds and opens the
                  dashboard
                </span>
              </div>
            </TerminalWindow>
          </div>

          <div className="step">
            <h3>
              <span className="step-num">03</span> — dig in
            </h3>
            <TerminalWindow title="zsh — archaeology" right="history">
              <div className="line">
                <span className="prompt">
                  <span className="user">user</span>
                  <span className="host">@gitx</span>
                  <span className="path">:~$</span>
                </span>{" "}
                <span className="cmd">gitx hotspots</span>
              </div>
              <div className="line out">
                <span>src/analysis.rs — 17 changes · 6.4k churn · 3 fix commits · risk 0.82</span>
              </div>
              <div className="line">
                <span className="prompt">
                  <span className="user">user</span>
                  <span className="host">@gitx</span>
                  <span className="path">:~$</span>
                </span>{" "}
                <span className="cmd">gitx search "deadlock"</span>
              </div>
              <div className="line out">
                <span>4 commits · 6 files · 2 authors</span>
              </div>
              <div className="line">
                <span className="prompt">
                  <span className="user">user</span>
                  <span className="host">@gitx</span>
                  <span className="path">:~$</span>
                </span>{" "}
                <span className="cmd">gitx lineage src/main.rs</span>
              </div>
              <div className="line out">
                <span>src/main.rs — 14 commits · 3 renames · first seen 2024-03-02</span>
              </div>
              <div className="line">
                <span className="prompt">
                  <span className="user">user</span>
                  <span className="host">@gitx</span>
                  <span className="path">:~$</span>
                </span>{" "}
                <span className="cmd">gitx recovery</span>
                <span className="cursor amber" aria-hidden="true" />
              </div>
            </TerminalWindow>
          </div>
        </div>
      </section>

      {/* ------------------------------ features ----------------------------- */}
      <section className="block" aria-labelledby="why-head">
        <h2 className="block-head" id="why-head">
          gitx --why
        </h2>
        <TerminalWindow title="gitx --why" right="README.md">
          <ul className="features">
            {FEATURES.map(([title, body]) => (
              <li key={title}>
                <b>{title}.</b> <span className="out">{body}</span>
              </li>
            ))}
          </ul>
        </TerminalWindow>
      </section>

      {/* -------------------------------- CTA -------------------------------- */}
      <section className="block" aria-labelledby="cta-head">
        <h2 className="block-head" id="cta-head">
          try it
        </h2>
        <TerminalWindow title="zsh — ready" right="●">
          <div className="line">
            <span className="prompt">
              <span className="user">user</span>
              <span className="host">@gitx</span>
              <span className="path">:~$</span>
            </span>{" "}
            <span className="cmd">gitx</span>
            <span className="cursor" aria-hidden="true" />
          </div>
          <p className="dim mt">
            Your repository, explained. Read the{" "}
            <Link href="/about">about</Link> page or the full docs at{" "}
            <a
              href="https://github.com/abuzarkhan1/gitx/tree/main/docs"
              target="_blank"
              rel="noopener noreferrer"
            >
              docs/
            </a>
            .
          </p>
        </TerminalWindow>
      </section>
    </>
  );
}
