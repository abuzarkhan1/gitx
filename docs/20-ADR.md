# Architecture Decision Records

This document contains the initial architectural decisions. Future decisions should use the same format.

---

## ADR-001: Rust as the primary language

### Status

Accepted

### Decision

GitX will be implemented primarily in Rust.

### Reason

GitX requires:

- filesystem operations
- Git object processing
- concurrency
- graph analysis
- local database access
- cross-platform TUI
- low overhead

Rust fits these requirements while providing strong memory safety.

---

## ADR-002: Local-first architecture

### Status

Accepted

### Decision

GitX operates locally and does not require a server.

### Reason

The product is a developer CLI for repository analysis. Repository history and source metadata can be highly sensitive.

---

## ADR-003: No AI

### Status

Accepted

### Decision

AI/LLM functionality is not part of GitX.

### Reason

The core value is deterministic repository intelligence. Every insight should be reproducible and explainable from Git/repository data.

---

## ADR-004: SQLite for local indexing

### Status

Accepted

### Decision

Use SQLite for persistent local indexing.

### Reason

GitX needs fast repeated queries and incremental indexing without requiring a database server.

---

## ADR-005: TUI as primary interactive interface

### Status

Accepted

### Decision

Use a terminal UI powered by ratatui.

### Reason

The product is intentionally a CLI tool. A TUI provides much richer exploration while retaining terminal-native behavior.

---

## ADR-006: gix as primary Git library

### Status

Accepted

### Decision

Use gix as the primary Git object/repository access layer.

### Reason

GitX should understand Git natively rather than repeatedly shelling out to the Git executable.

---

## ADR-007: JSON output

### Status

Accepted

### Decision

Major analytical commands provide machine-readable JSON.

### Reason

CLI tools become significantly more useful when their results can be consumed by scripts and other local developer tooling.

---

## ADR-008: Explainable scoring

### Status

Accepted

### Decision

All derived scores expose their underlying signals.

### Reason

A deterministic score without evidence is difficult to trust and debug.

---

## ADR-009: Incremental indexing

### Status

Accepted

### Decision

GitX uses persistent incremental indexing rather than full analysis on every invocation.

### Reason

Large repositories make repeated full scans unacceptable for an interactive tool.

---

## ADR-010: Read-only analysis

### Status

Accepted

### Decision

Normal GitX analysis does not modify the Git repository or working tree.

### Reason

Repository intelligence should be safe by default.

---

## ADR-011: Tree-sitter as optional structural layer

### Status

Proposed

### Decision

Use Tree-sitter adapters for language-aware analysis when the base repository/file model is mature.

### Reason

Language-aware symbols and structural complexity are valuable, but should not complicate the core Git engine.
