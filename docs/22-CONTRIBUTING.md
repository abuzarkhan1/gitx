# Contributing

## 1. Development setup

Required:

- Rust stable toolchain
- Git
- SQLite tooling optional for debugging
- a terminal supporting standard ANSI features

## 2. Workflow

```text
issue/problem
→ design
→ ADR/spec update if needed
→ implementation
→ tests
→ benchmark if performance-sensitive
→ documentation
→ review
```

## 3. Code organization

Keep business logic outside:

- CLI argument parsing
- TUI rendering
- database-specific code

Prefer domain services and explicit interfaces.

## 4. Error handling

Library crates should return typed errors.

Application boundaries may add contextual errors.

Errors should explain:

- what failed
- why it failed
- what the user can do

## 5. Logging

Use `tracing`.

Do not use ad-hoc debug printing in production paths.

## 6. Tests

Every non-trivial algorithm requires tests.

History-related behavior should use fixture repositories rather than mocks wherever possible.

## 7. Commit conventions

Prefer clear commits:

```text
feat: add incremental ref indexing
fix: handle renamed file lineage
refactor: isolate SQLite storage
test: add merge-history fixture
docs: document hotspot scoring
```

Avoid vague messages such as:

```text
update
changes
fix stuff
work
```

## 8. Pull requests

A PR should explain:

- what changed
- why
- affected modules
- tests
- performance impact
- documentation impact

## 9. New dependency policy

Before adding a dependency, document:

- problem solved
- why standard library is insufficient
- maintenance status
- license compatibility
- binary/compile impact

## 10. Breaking changes

For changes to:

- CLI syntax
- JSON output
- database schema
- analysis formulas
- configuration

update the relevant documentation and ADR before implementation.
