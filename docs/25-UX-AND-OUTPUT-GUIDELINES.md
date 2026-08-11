# UX and Output Guidelines

## 1. Terminal-native identity

GitX should feel like a serious Unix/developer CLI.

Avoid excessive decoration.

## 2. Information hierarchy

Use:

```text
summary
→ key signals
→ evidence
→ raw detail
```

## 3. Colors

Color is supplemental.

Every status must remain understandable without color.

Examples:

```text
[OK]
[WARN]
[HIGH]
[CRITICAL]
```

can accompany color.

## 4. Tables

Tables should:

- have stable columns
- support sorting where interactive
- avoid unnecessary precision
- truncate safely
- offer detail views

## 5. Dates

Human output can use relative dates:

```text
2h ago
3d ago
```

Detail views should provide exact timestamps.

## 6. Paths

Use repository-relative paths by default.

Absolute paths should appear only when necessary.

## 7. OIDs

Default to abbreviated OIDs.

Detail views can show the full OID.

## 8. Long output

Use pagination or TUI detail views.

Never dump thousands of rows by default.

## 9. Errors

Bad:

```text
error
```

Good:

```text
Could not open repository.

Path:
  /path/to/project

Reason:
  no Git repository was found.

Try:
  cd into a Git repository and run gitx again.
```

## 10. JSON

JSON should be:

- valid
- deterministic
- stable
- free from ANSI
- free from progress messages
- documented

## 11. TUI loading

Expensive work should never freeze the interface without explanation.

Use:

```text
Indexing commits… 42%
```

or:

```text
Analyzing hotspots…
```

## 12. Destructive-looking actions

Recovery screens must make it obvious when an action could modify the repository.

Read-only inspection should be the default.
