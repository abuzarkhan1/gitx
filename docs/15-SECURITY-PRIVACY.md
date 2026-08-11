# Security and Privacy

## 1. Privacy model

GitX is local-first.

By default:

- no repository data leaves the machine
- no telemetry is required
- no account is required
- no server is required
- no cloud index exists

## 2. Repository safety

GitX must treat the repository as user-owned data.

Normal analysis operations must be read-only.

The tool should not modify:

- source files
- Git refs
- branches
- tags
- index
- working tree

except for explicitly documented maintenance commands such as GitX's own local index/cache.

## 3. Local index

The GitX SQLite index may contain sensitive repository metadata.

Therefore:

- store it in a predictable local cache location
- document how to delete it
- never upload it automatically
- avoid unnecessary duplication of full source contents

## 4. Sensitive data

Commit messages and author information may contain sensitive data.

Do not send them anywhere.

Avoid displaying full raw email addresses in contexts where a normalized display identity is sufficient.

## 5. Symlinks and filesystem traversal

Repository scanning must defend against accidental traversal outside the repository root where path resolution is involved.

## 6. Path handling

Normalize paths safely.

Never construct shell commands from untrusted repository paths without proper argument handling.

## 7. Command execution

Avoid shelling out to Git.

If a Git subprocess is ever required:

- use direct process APIs
- pass arguments structurally
- never build shell strings
- never invoke a shell unnecessarily

## 8. Recovery

Recovery is read-only by default.

Any repository-mutating action must require explicit user intent.

## 9. Logging

Do not log:

- source contents
- full diffs
- secrets
- credentials
- unnecessary personal data

Diagnostic logs should be minimal.

## 10. Network

GitX core must not require network access.

Any future network-related feature would require an explicit scope and security review. It is not part of the current product.
