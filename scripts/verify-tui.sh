#!/usr/bin/env bash
# Headless TUI verification (docs/08 polish pass): builds a small fixture
# repo, drives `gitx-tui` in a tmux pane, captures each view, and greps for
# the expected markers. Run:  scripts/verify-tui.sh
set -u

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
BIN="$ROOT/target/debug/gitx-tui"
FIX=/tmp/gitx-tui-verify-repo
OUT=/tmp/gitx-tui-verify
SESS=gitxverify
PASS=0
FAIL=0

rm -rf "$FIX" "$OUT"
mkdir -p "$FIX" "$OUT"

# ── fixture repo ─────────────────────────────────────────────────────────
(
  cd "$FIX" || exit 1
  git init -q -b main
  git config user.email t@example.com
  git config user.name Tester
  git config commit.gpgsign false
  mkdir -p src
  printf 'module example.com/fix\ngo 1.21\n' > go.mod
  printf 'fn main() { println!("hello"); }\n' > src/main.rs
  printf 'pub fn add(a: i32, b: i32) -> i32 { a + b }\n' > src/lib.rs
  printf '# fixture\n' > README.md
  git add -A
  git commit -qm "feat: initial scaffold"
  for i in 2 3 4 5 6 7 8 9 10 11 12; do
    case $i in
      3|7) echo "// churn $i" >> src/lib.rs ;;
      5|9) echo "fn f$i() {}" >> src/main.rs ;;
      11)  echo "fix: $i" >> README.md ;;
      *)   echo "// commit $i" >> src/lib.rs ;;
    esac
    git add -A
    git commit -qm "feat: iteration $i"
  done
  echo "pub fn util() {}" > src/utils.rs
  git add -A
  git commit -qm "feat: add utility module"
  git commit --allow-empty -qm "fix: resolve flaky test"
  git checkout -qb feature/wip
  echo "// feature work" >> src/lib.rs
  git add -A
  git commit -qm "feat: wip feature"
  git checkout -q main
)

[ -x "$BIN" ] || { echo "build gitx-tui first: cargo build -p gitx-tui"; exit 1; }

# ── lazy loading (docs/13 §7), in its own session so the main flow starts
#    from a clean nav state: the Overview must paint real stats from Phase A
#    while the heavy panels still load in the background ────────────────
tmux kill-session -t gitxlazy 2>/dev/null
tmux new-session -d -s gitxlazy -x 140 -y 44
tmux send-keys -t gitxlazy "cd $FIX && TERM=xterm-256color $BIN" Enter
LAZY_OK=0
for _ in $(seq 1 20); do
  tmux capture-pane -t gitxlazy -p > "$OUT/00_lazy_overview.txt"
  if grep -aq "Repository size" "$OUT/00_lazy_overview.txt"; then LAZY_OK=1; break; fi
  sleep 0.2
done
if [ "$LAZY_OK" = 1 ]; then
  echo "  PASS  Overview paints Phase A stats at startup (lazy)"
  PASS=$((PASS + 1))
else
  echo "  FAIL  Overview stats did not paint within ~4s"
  FAIL=$((FAIL + 1))
fi
# If a load stage is still visible in that frame, the Overview must already
# be populated (phased loading) — never the eager-loading placeholder.
if grep -aq "Esc cancel" "$OUT/00_lazy_overview.txt"; then
  if grep -aq "Loading repository data" "$OUT/00_lazy_overview.txt"; then
    echo "  FAIL  Overview shows placeholder while a load stage is running"
    FAIL=$((FAIL + 1))
  else
    echo "  PASS  Overview populated while load still in progress (phased)"
    PASS=$((PASS + 1))
  fi
else
  echo "  PASS  load completed before first Overview frame (tiny fixture)"
  PASS=$((PASS + 1))
fi
# A heavy panel visited early shows its loading placeholder (or real data),
# never the misleading "run gitx refresh" empty state while the load runs.
tmux send-keys -t gitxlazy v; sleep 0.3
tmux capture-pane -t gitxlazy -p > "$OUT/00_lazy_recovery.txt"
if grep -aq "Run: gitx refresh" "$OUT/00_lazy_recovery.txt"; then
  echo "  FAIL  Recovery shows empty-state while loading"
  FAIL=$((FAIL + 1))
else
  echo "  PASS  Recovery visited early: no misleading empty state"
  PASS=$((PASS + 1))
fi
tmux kill-session -t gitxlazy 2>/dev/null

# ── `gitx` (no args) launches the dashboard (docs/01 UC-01, docs/16 §7) ──
# The CLI must open the TUI Overview in a terminal instead of printing a
# "separate binary" hint.
tmux kill-session -t gitxcli 2>/dev/null
tmux new-session -d -s gitxcli -x 140 -y 44
tmux send-keys -t gitxcli "cd $FIX && TERM=xterm-256color $ROOT/target/debug/gitx" Enter
for _ in $(seq 1 20); do
  tmux capture-pane -t gitxcli -p | grep -q "Overview" && break
  sleep 0.5
done
tmux capture-pane -t gitxcli -p > "$OUT/00_cli_noarg.txt"
if grep -q "Overview" "$OUT/00_cli_noarg.txt"; then
  echo "  PASS  gitx (no args) opens the TUI Overview"
  PASS=$((PASS + 1))
else
  echo "  FAIL  gitx (no args) opens the TUI Overview"
  FAIL=$((FAIL + 1))
fi
tmux send-keys -t gitxcli q
sleep 0.4
tmux kill-session -t gitxcli 2>/dev/null

tmux kill-session -t "$SESS" 2>/dev/null
tmux new-session -d -s "$SESS" -x 140 -y 44
tmux send-keys -t "$SESS" "cd $FIX && TERM=xterm-256color $BIN" Enter

# Wait until the header (branding) is on screen and the loader finished.
for _ in $(seq 1 40); do
  sleep 0.5
  tmux capture-pane -t "$SESS" -p | grep -q "GitX" && break
done
# Wait until the loader finished: keep polling while the status bar shows
# the progress/cancel hint (docs/08 §6), break once it is gone.
for _ in $(seq 1 60); do
  sleep 0.5
  tmux capture-pane -t "$SESS" -p | grep -q "Esc cancel" || break
done

snap() { tmux capture-pane -t "$SESS" -p > "$OUT/$1.txt"; }
keys() { tmux send-keys -t "$SESS" "$1"; sleep "${2:-0.5}"; }
# tmux's `send-keys Esc` key-name form gets eaten by its own escape handling;
# a raw ESC byte via -l is delivered reliably to the app.
esc() { tmux send-keys -t "$SESS" -l $'\x1b'; sleep "${1:-0.3}"; }
wait_for() { # wait_for <pattern> <timeout-loops>
  for _ in $(seq 1 "${2:-20}"); do
    tmux capture-pane -t "$SESS" -p | grep -aq "$1" && return 0
    sleep 0.4
  done
  return 1
}
check() { # check <file> <pattern> <label>
  if grep -aq "$2" "$OUT/$1.txt"; then
    echo "  PASS  $3"
    PASS=$((PASS + 1))
  else
    echo "  FAIL  $3  (grep '$2' in $1)"
    FAIL=$((FAIL + 1))
  fi
}

echo "== capturing views =="
snap 00_overview                      # onboarding hint + charts + gauges
check 00_overview "Getting started"   "first-run onboarding hint"
check 00_overview "Activity — commits per week" "activity bar chart"
check 00_overview "Languages — share" "language breakdown bars"
check 00_overview "Repository size"   "repo-size gauge (small/medium/large)"
check 00_overview "Health — six measured" "six health sub-score gauges"
check 00_overview "▸ Overview"        "you-are-here breadcrumb"

# Scroll the Overview down to reach its below-the-fold sections.
keys 'Enter' 0.5                       # enter Overview content (nav_used)
for _ in $(seq 1 55); do keys 'j' 0.1; done
snap 01_overview_scrolled
check 01_overview_scrolled "Overall"         "overall health gauge"
check 01_overview_scrolled "Top hotspots"    "hotspot score bars"
check 01_overview_scrolled "Contributors — share" "contributor share bars"
check 01_overview_scrolled "showing"         "scroll-position indicator (range)"

# ── cursor navigation: j/k moves the selection, not just the window ─────
keys 'k' 0.3
keys 'k' 0.3
keys 'k' 0.3
snap 02_cursor_moved
L1=$(grep -n "▶" "$OUT/01_overview_scrolled.txt" | head -1 | cut -d: -f1)
L2=$(grep -n "▶" "$OUT/02_cursor_moved.txt" | head -1 | cut -d: -f1)
if [ -n "$L1" ] && [ -n "$L2" ] && [ "$L1" != "$L2" ]; then
  echo "  PASS  j/k moves the cursor highlight (row $L1 -> $L2)"
  PASS=$((PASS + 1))
else
  echo "  FAIL  j/k moves the cursor highlight (row $L1 vs $L2)"
  FAIL=$((FAIL + 1))
fi

# ── mouse: wheel scrolls, sidebar click jumps to Timeline (row 5) ───────
tmux send-keys -t "$SESS" -l $'\033[<65;5;10M'
sleep 0.4
snap 03_mouse_scrolled
L3=$(grep -n "▶" "$OUT/03_mouse_scrolled.txt" | head -1 | cut -d: -f1)
if [ -n "$L2" ] && [ -n "$L3" ] && [ "$L2" != "$L3" ]; then
  echo "  PASS  mouse wheel moves the cursor (row $L2 -> $L3)"
  PASS=$((PASS + 1))
else
  echo "  FAIL  mouse wheel moves the cursor (row $L2 vs $L3)"
  FAIL=$((FAIL + 1))
fi
tmux send-keys -t "$SESS" -l $'\033[<0;4;5M'   # click sidebar row 5 → Timeline
if wait_for "Timeline — commit graph" 15; then
  echo "  PASS  mouse click jumps to sidebar view"
  PASS=$((PASS + 1))
else
  echo "  FAIL  mouse click jumps to sidebar view"
  FAIL=$((FAIL + 1))
fi

# ── timeline: commit graph lanes ─────────────────────────────────────────
keys 'j' 0.3
keys 'j' 0.3
snap 04_timeline
check 04_timeline "commit graph"      "timeline commit graph (ASCII lanes)"
check 04_timeline "•"                 "graph glyphs render"

# ── commits: related panel + Enter opens the row under the cursor ───────
esc 0.3
keys 'c' 0.5                           # Commits view
snap 05a_commits_top
keys 'j' 0.3
keys 'j' 0.3
snap 05_commits
check 05_commits "related"            "commit view related-commits panel"
# The commit under the cursor (7-hex short oid) must be the one Enter opens.
# Extract from the frame captured AFTER the two cursor moves (05_commits),
# since Enter opens what the cursor is on at that moment.
SHORT=$(grep "▶" "$OUT/05_commits.txt" | grep -o '[0-9a-f]\{7\}' | head -1)
keys 'Enter' 0.8
snap 06_commit_detail
check 06_commit_detail "insertions(+)" "commit detail diff stats"
check 06_commit_detail "files changed" "commit detail file list"
if [ -n "$SHORT" ] && grep -aq "commit $SHORT" "$OUT/06_commit_detail.txt"; then
  echo "  PASS  Enter opens the row under the cursor ($SHORT)"
  PASS=$((PASS + 1))
else
  echo "  FAIL  Enter opens the row under the cursor (wanted commit $SHORT)"
  FAIL=$((FAIL + 1))
fi

esc 0.3
esc 0.3
keys 'b' 0.5                           # Branches
snap 07_branches
check 07_branches "ahead"             "branch ahead/behind bars"

esc 0.3
keys 'f' 0.5                           # Files
keys 'j' 0.3
keys 'Enter' 0.8
snap 08_file_lineage
check 08_file_lineage "Lineage of"    "file view lineage (first/last/renames)"
check 08_file_lineage "Created by"    "file creation + last-change authors"
esc 0.3
esc 0.3

keys 'u' 0.5                           # Contributors
snap 09_contributors
check 09_contributors "areas:"        "contributor areas / ownership concentration"
check 09_contributors "first"         "contributor first/last activity"

esc 0.3
keys 's' 0.5                           # Hotspots
snap 10_hotspots
check 10_hotspots "sort:"             "hotspots sortable table (sort label)"
keys 's' 0.4
if wait_for "Sort: changes" 10; then
  echo "  PASS  hotspots sort toggled to changes"
  PASS=$((PASS + 1))
else
  echo "  FAIL  hotspots sort toggled to changes"
  FAIL=$((FAIL + 1))
fi

esc 0.3
keys 'w' 0.5                           # Ownership
snap 12_ownership
check 12_ownership "Ownership — per-file" "per-file ownership % bars"

esc 0.3
keys 'a' 0.6                           # Architecture
snap 13_architecture
check 13_architecture "before vs after" "architecture structural before/after"
check 13_architecture "Modules added"  "architecture modules-added list"

esc 0.3
keys 'd' 0.5                           # Dependencies
snap 14_dependencies
check 14_dependencies "Dependencies — declared" "dependencies view"

esc 0.3
keys 'x' 0.5                           # Risk
snap 15_risk
check 15_risk "evidence-backed"       "risk view with colors + evidence"

esc 0.3
keys 'e' 0.5                           # Health
snap 16_health
check 16_health "Plain language:"     "health plain-language verdict"
check 16_health "Evidence:"           "health evidence panel"
keys 'j' 0.4
if wait_for "concentrated" 10; then
  echo "  PASS  health selection switches the evidence panel"
  PASS=$((PASS + 1))
else
  echo "  FAIL  health selection switches the evidence panel"
  FAIL=$((FAIL + 1))
fi

esc 0.3
keys 'v' 0.5                           # Recovery
snap 18_recovery
check 18_recovery "Reflog:"           "recovery view"

esc 0.3
keys 'g' 0.6                           # Graph
snap 19_graph
check 19_graph "Modules — files, import & call edges" "graph module table (dir/file/import/call)"
check 19_graph "Totals"                "graph totals line"

# ── search (async FTS) ──────────────────────────────────────────────────
esc 0.3
keys '/' 0.4
keys 'feat' 1.5
if wait_for "Search (commits · files · authors · branches · tags · renames · symbols · code)" 10; then
  echo "  PASS  search view opens with async FTS"
  PASS=$((PASS + 1))
else
  echo "  FAIL  search view opens with async FTS"
  FAIL=$((FAIL + 1))
fi
if wait_for "commit  *[0-9a-f]" 15; then   # badge 'commit   <oid>' (basic-regex compatible)
  echo "  PASS  search results (commit scope)"
  PASS=$((PASS + 1))
else
  echo "  FAIL  search results (commit scope)"
  FAIL=$((FAIL + 1))
fi

# ── Ctrl+C quits even from nav mode ─────────────────────────────────────
tmux send-keys -t "$SESS" C-c
sleep 0.6
tmux capture-pane -t "$SESS" -p > "$OUT/20_ctrlc.txt"
if grep -aq "gitx-tui" "$OUT/20_ctrlc.txt" && ! grep -aq "GitX" "$OUT/20_ctrlc.txt"; then
  echo "  PASS  Ctrl+C quits from nav mode"
  PASS=$((PASS + 1))
else
  echo "  FAIL  Ctrl+C quits from nav mode"
  FAIL=$((FAIL + 1))
fi

tmux kill-session -t "$SESS" 2>/dev/null

# ── theme support: GITX_THEME=light uses the light palette ──────────────
# ratatui emits 256-color codes: light theme fg is black (38;5;0) and its
# selection bg is cyan (48;5;6); the default theme uses white fg + blue sel.
tmux new-session -d -s "$SESS" -x 140 -y 44
tmux send-keys -t "$SESS" "cd $FIX && TERM=xterm-256color GITX_THEME=light $BIN" Enter
sleep 4
tmux capture-pane -t "$SESS" -p -e > "$OUT/21_theme.txt"
if grep -aq $'\033\[38;5;0m' "$OUT/21_theme.txt" || grep -aq $'\033\[48;5;6m' "$OUT/21_theme.txt"; then
  echo "  PASS  GITX_THEME=light applies the light palette"
  PASS=$((PASS + 1))
else
  echo "  FAIL  GITX_THEME=light applies the light palette"
  FAIL=$((FAIL + 1))
fi
tmux send-keys -t "$SESS" q
sleep 0.4
tmux kill-session -t "$SESS" 2>/dev/null

echo
echo "== result: $PASS passed, $FAIL failed =="
echo "frames in $OUT/"
exit "$FAIL"
