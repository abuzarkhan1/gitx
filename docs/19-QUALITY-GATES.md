# Quality Gates

## 1. Code quality

Before merging:

- `cargo fmt --check`
- `cargo clippy`
- tests pass
- no unexplained warnings
- public APIs documented where appropriate

## 2. Correctness

Required:

- fixture repositories pass
- incremental index matches full index
- merge history tests pass
- rename tests pass
- reflog tests pass
- recovery tests pass

## 3. Performance

No major regression in:

- initial indexing
- incremental indexing
- search
- hotspot calculation
- TUI startup

## 4. UX

TUI must:

- work without mouse
- handle small terminals
- provide useful errors
- avoid blocking on long work
- show progress for expensive operations
- support keyboard navigation

## 5. Explainability

Every score:

- has a documented formula
- has visible evidence
- has a defined time window
- has deterministic behavior

## 6. Privacy

Release must verify:

- no telemetry
- no hidden network calls
- no source upload
- no repository modification during analysis
- no sensitive data in logs

## 7. Documentation

Every released command must have:

- help text
- usage example
- documented output
- JSON behavior if supported

## 8. Definition of Done

A feature is not complete until:

```text
implementation
+ tests
+ CLI behavior
+ TUI behavior where relevant
+ error handling
+ documentation
+ performance consideration
```

are complete.
