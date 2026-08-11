#!/usr/bin/env bash
# Build the edge-case fixture repository (docs/14 §5, tests/fixtures/README.md).
#
# Deterministic: fixed dates and content, so the resulting history is stable.
# Usage: ./build.sh [target-dir]   (default: ./edge-cases)
set -euo pipefail

TARGET="${1:-$(dirname "$0")/edge-cases}"
rm -rf "$TARGET"
mkdir -p "$TARGET"
cd "$TARGET"

git init -q -b main
git config user.email fixtures@example.com
git config user.name "Fixture Bot"
git config commit.gpgsign false

commit() { # message
  git add -A
  GIT_AUTHOR_DATE="2026-01-0${1}T10:00:00 +0000" \
  GIT_COMMITTER_DATE="2026-01-0${1}T10:00:00 +0000" \
    git commit -qm "$2"
}

# c1: initial files (text + binary).
printf 'alpha\n' > alpha.txt
printf '\x00\x01\x02\xff\xfe binary payload' > blob.bin
commit 1 "add: initial files"

# c2: modify the text file.
printf 'alpha\nbeta\n' > alpha.txt
commit 2 "fix: extend alpha"

# c3: empty commit (no tree change).
git commit -q --allow-empty -m "chore: empty commit"

# c4: revert of c2.
git revert --no-edit HEAD~1

# c5: merge commit from a side branch.
git checkout -q -b feature/experiment
printf 'gamma\n' > gamma.txt
commit 5 "feat: experiment module"
git checkout -q main
printf 'mainline\n' > mainline.txt
commit 6 "feat: mainline change"
git merge -q --no-ff feature/experiment -m "merge: bring in experiment"

# c6: rewritten HEAD (amend after soft reset) — leaves the pre-rewrite
# commit unreachable, exercising rewritten-history detection.
git reset -q --soft HEAD~1
git commit -q --amend -m "feat: mainline change (rewritten)"

echo "built fixture at $TARGET:"
git log --oneline --graph | head -12
