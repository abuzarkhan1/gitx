# Recovery Specification

## 1. Objective

Help developers discover historical Git objects that are no longer reachable from normal branches or tags.

## 2. Sources

- reflog
- unreachable commits
- dangling commits
- dangling trees
- dangling blobs
- deleted branch tips

## 3. Commands

```bash
gitx recovery
gitx recovery reflog
gitx recovery unreachable
gitx recovery show <OID>
```

## 4. Read-only default

Recovery inspection must never mutate the repository.

The default behavior is:

```text
inspect
rank
explain
display
```

No reset, branch deletion, checkout, or destructive Git operation should happen automatically.

## 5. Recoverability presentation

Example:

```text
RECOVERABLE COMMIT

OID: a82192f
Age: 4 days
Message: feat: workspace persistence

Last known reference:
  refs/heads/feature/workspaces

Reason currently unreachable:
  branch deleted

Actions:
  [V] View
  [P] Export patch
```

## 6. Export

A safe recovery action may export:

- commit patch
- commit metadata
- selected files

Repository mutation must remain an explicit user action outside the read-only inspection flow.

## 7. Garbage collection warning

Unreachable Git objects may eventually be pruned.

The UI should communicate that recoverability is not permanent.

## 8. Safety

Never automatically run:

```text
git reset
git clean
git gc
git branch -D
git update-ref
```

from recovery inspection.
