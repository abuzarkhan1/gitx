# Technical Stack

## 1. Core language

### Rust

Rust is the primary implementation language.

Reasons:

- excellent filesystem performance
- strong concurrency model
- low runtime overhead
- cross-platform binaries
- strong CLI ecosystem
- suitable for Git object processing
- suitable for TUI development
- good memory safety for long-running repository analysis

## 2. Git engine

### gix

Use the `gix` ecosystem as the primary Git implementation layer.

Responsibilities:

- repository discovery
- object access
- commit traversal
- trees
- blobs
- refs
- branches
- tags
- diffs
- object IDs
- repository state

Avoid making the entire application a wrapper around shell commands.

Git CLI fallback should only be introduced when a capability is demonstrably missing or significantly more reliable through Git itself.

## 3. CLI

### clap

Use `clap` for:

- command parsing
- subcommands
- flags
- arguments
- generated help
- shell completion integration

## 4. TUI

### ratatui

Use `ratatui` for:

- layout
- widgets
- tables
- lists
- tabs
- gauges
- charts
- scrollable panels
- dialogs

### crossterm

Use `crossterm` for:

- terminal setup
- raw mode
- keyboard input
- mouse input where needed
- terminal events
- cross-platform terminal control

## 5. Storage

### SQLite

SQLite is the local analytical index.

Reasons:

- zero-server architecture
- transactional
- portable
- mature
- excellent for structured local queries
- supports FTS5
- easy backup/deletion
- suitable for incremental indexing

### rusqlite

Rust database driver.

The storage layer must hide SQLite details from analysis engines.

## 6. Serialization

### serde

Use `serde` for domain serialization.

### serde_json

Use for:

```bash
gitx hotspots --json
gitx timeline --json
gitx branches --json
```

JSON schemas should remain documented and versionable.

## 7. Graph processing

### petgraph

Use for:

- commit relationships
- file dependency graphs
- module relationships
- ownership graphs
- architecture graphs

Do not force every graph problem into one abstraction. Keep graph models domain-specific.

## 8. Concurrency

### rayon

Use for CPU-heavy independent operations:

- commit analysis
- diff/stat calculation
- file metric calculation
- historical aggregation

Database writes must remain controlled and transactional.

## 9. Filesystem traversal

### ignore

Use for efficient filesystem traversal and ignore-rule awareness where source-tree scanning is needed.

Do not accidentally scan `.git` as normal source content.

## 10. Errors

### anyhow

Use at application boundaries for contextual errors.

### thiserror

Use in library crates for typed domain errors.

## 11. Logging

### tracing

Use structured logging.

Normal CLI/TUI output must not be polluted by logs.

Logs should be directed to a diagnostic file or explicitly enabled stderr mode.

## 12. Testing

Use:

- Rust unit tests
- integration tests
- fixture repositories
- snapshot tests for TUI/CLI output where appropriate
- property tests where valuable
- criterion benchmarks

## 13. Performance

### criterion

Use for:

- initial indexing
- incremental indexing
- commit traversal
- hotspot calculation
- search
- graph construction

## 14. Distribution

### cargo-dist

Use for repeatable release packaging.

Targets should include:

- macOS
- Linux
- Windows

Architectures should be expanded based on CI/release needs.

## 15. Optional structural analysis

### Tree-sitter

Use later for language-aware structural analysis.

The architecture should define a language analyzer abstraction so GitX can support:

- Rust
- TypeScript
- JavaScript
- Python
- Go
- other languages

without coupling the core Git engine to a specific parser.

## 16. Dependency policy

Dependencies must be:

- actively maintained where possible
- permissively licensed or otherwise compatible with project licensing
- justified by a concrete need
- reviewed before addition
- pinned through `Cargo.lock` for application builds

Do not add a package just because it makes a small utility convenient.

## 17. Hashing

Use Git object IDs (via gix) as the primary content/change identifiers.

Use Rust standard-library hashing only for internal non-persistent purposes (e.g. in-memory maps), never as a substitute for Git's own object identity.

Do not add a bespoke hashing dependency without a documented need.
