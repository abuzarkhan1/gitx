# Release Engineering

## 1. Release goals

GitX should ship as a standalone binary.

Users should not need:

- Rust
- Node.js
- Python
- GitX server
- database server

## 2. Build

Use Cargo release builds with appropriate optimization.

## 3. Platforms

Initial release targets:

- macOS
- Linux
- Windows

Architectures should be selected based on CI and user demand.

## 4. Versioning

Use Semantic Versioning:

```text
MAJOR.MINOR.PATCH
```

Examples:

```text
0.1.0
0.2.0
1.0.0
```

## 5. Release artifacts

Provide:

- executable
- checksum
- release notes

## 6. CI release checks

Before publishing:

- formatting
- clippy
- unit tests
- integration tests
- fixture tests
- build matrix
- smoke tests

## 7. Database compatibility

GitX index formats must be versioned independently from the application version.

If an index is incompatible:

```text
detect
→ explain
→ migrate or rebuild
```

Never silently corrupt or reinterpret an incompatible index.

## 8. Backward compatibility

CLI output intended for humans may evolve.

JSON output should use documented compatibility rules.

Breaking JSON schema changes require a versioning decision.

## 9. Installation

Document at least:

- binary download
- package-manager installation if later available
- source build
- shell completion
