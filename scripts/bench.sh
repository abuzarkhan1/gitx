#!/usr/bin/env bash
# Run the full benchmark suite and append a timestamped section to
# benches/RESULTS.md (docs/13 §10 regression gate). Tune the criterion
# warm-up/measurement for a quick dev run; CI stays compile-only
# (cargo bench --no-run) per the current workflow.
set -u

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT" || exit 1
OUT="benches/RESULTS.md"

echo "== running benches (services + analysis) =="
LOG="$(mktemp)"
# Target the criterion bench binaries explicitly: `--` flags are rejected by
# the lib unit-test binaries that `cargo bench --workspace` also runs.
if cargo bench --workspace --no-run 2>&1 | tee -a "$LOG"; then
  :
else
  echo "bench compile failed (see log above)"
  exit 1
fi
if cargo bench --workspace --bench operations --bench hotspots -- --warm-up-time 1 --measurement-time 3 2>&1 | tee -a "$LOG"; then
  :
else
  echo "bench run failed (see log above)"
  exit 1
fi

HOST="$(hostname 2>/dev/null || uname -n 2>/dev/null || echo unknown)"
DATE="$(date +%Y-%m-%d)"
{
  echo ""
  echo "## $DATE — $HOST"
  echo ""
  echo "| Crate | Bench | Mean |"
  echo "|---|---|---|"
  # One row per criterion bench: pull the mean from "time: [lo mean hi]".
  # Analysis benches are named without a crate/ prefix (e.g. "calculate_hotspot_score").
  awk '
    /^Benchmarking / {
      split($0, a, "/");
      crate = a[1]; sub(/^Benchmarking /, "", crate); sub(/:.*/, "", crate);
      bench = a[2]; sub(/:.*/, "", bench);
      if (bench == "") { bench = crate; crate = "gitx-analysis"; }
      next
    }
    /time:.*\[/ {
      # Criterion prints "time:   [lo unit mean unit hi unit]"; some benches
      # inline the bench name before "time:". Grab the tokens after "[" and
      # emit the middle (mean) value + its unit.
      line = $0
      sub(/^.*\[/, "", line)
      gsub(/[\[\]]/, "", line)
      n = split(line, t, / +/)
      if (n >= 4) print "| " crate " | " bench " | " t[3] " " t[4] " |"
    }
  ' "$LOG"
  echo ""
} >> "$OUT"

echo "== results appended to $OUT =="
tail -12 "$OUT"
