#!/usr/bin/env bash
# GitX release verification (docs/18 §5–§6).
#
# Builds release binaries for the current host, smoke-tests the actual
# executables against a real repository, and validates that every artifact has
# a SHA-256 checksum. Run locally before tagging a release; CI runs the same
# steps per platform in .github/workflows/release.yml.
set -euo pipefail
cd "$(dirname "$0")/.."

HOST_TARGET="${CARGO_BUILD_TARGET:-$(rustc -vV | sed -n 's/^host: //p')}"
STAGE="$(mktemp -d)"
trap 'rm -rf "$STAGE"' EXIT

echo "==> cargo build --release --target $HOST_TARGET"
cargo build --release --workspace --target "$HOST_TARGET"

EXE=""
if [[ "$HOST_TARGET" == *windows* ]]; then EXE=".exe"; fi

echo "==> staging binaries"
mkdir -p "$STAGE/dist"
cp "target/$HOST_TARGET/release/gitx${EXE}" "$STAGE/dist/gitx${EXE}"
cp "target/$HOST_TARGET/release/gitx-tui${EXE}" "$STAGE/dist/gitx-tui${EXE}"

echo "==> generating SHA-256 checksums"
(
  cd "$STAGE/dist"
  if command -v sha256sum >/dev/null 2>&1; then
    sha256sum * > checksums.txt
  else
    shasum -a 256 * > checksums.txt
  fi
  cat checksums.txt
)

echo "==> smoke test: CLI against a real repository (docs/18 §6)"
SMOKE_REPO="$STAGE/repo"
mkdir -p "$SMOKE_REPO"
cd "$SMOKE_REPO"
git init -q -b main
git config user.email release@example.com
git config user.name "Release Check"
printf 'fn main() { println!("hi"); }\n' > main.rs
git add -A && git commit -qm "feat: initial"
printf 'name = "smoke"\n\n[dependencies]\nserde = "1.0"\n' > Cargo.toml
git add -A && git commit -qm "build: add manifest"
git tag v0.1.0
GITX="$STAGE/dist/gitx${EXE}"
"$GITX" --repo . info >/dev/null
"$GITX" --repo . stats >/dev/null
"$GITX" --repo . timeline --max 5 >/dev/null
"$GITX" --repo . history main.rs >/dev/null
"$GITX" --repo . blame main.rs >/dev/null
"$GITX" --repo . branches >/dev/null
"$GITX" --repo . contributors >/dev/null
"$GITX" --repo . hotspots >/dev/null
"$GITX" --repo . risk >/dev/null
"$GITX" --repo . health >/dev/null
"$GITX" --repo . dependencies >/dev/null
"$GITX" --repo . dependencies history --max 5 >/dev/null
"$GITX" --repo . search "initial" >/dev/null
"$GITX" --repo . recovery >/dev/null
"$GITX" --repo . release show v0.1.0 >/dev/null
"$GITX" --repo . release diff v0.1.0 HEAD >/dev/null
"$GITX" --repo . completions bash >/dev/null
"$GITX" --repo . hotspots --json >/dev/null
echo "smoke: all CLI commands exited 0"

echo "==> smoke test: checksum integrity"
(
  cd "$STAGE/dist"
  if command -v sha256sum >/dev/null 2>&1; then
    sha256sum -c checksums.txt >/dev/null
  else
    shasum -a 256 -c checksums.txt >/dev/null
  fi
)
echo "smoke: checksums verified"

echo "==> all release checks passed"
echo "    artifacts: $STAGE/dist (gitx, gitx-tui, checksums.txt)"
