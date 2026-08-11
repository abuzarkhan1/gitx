# CLI Specification

## 1. Invocation

```bash
gitx
```

Opening without a subcommand starts the interactive TUI.

## 2. Global options

```text
--repo <PATH>
--json
--no-color
--quiet
--verbose
--config <PATH>
--no-cache
--refresh
--version
--help
```

## 3. Repository

```bash
gitx info
gitx status
gitx stats
```

## 4. Indexing

```bash
gitx scan
gitx refresh
gitx index status
gitx index rebuild
gitx index clear
```

`scan` may perform an initial build.

`refresh` should prefer incremental processing.

## 5. Timeline

```bash
gitx timeline
gitx timeline --author <NAME>
gitx timeline --since <DATE>
gitx timeline --until <DATE>
gitx timeline --branch <BRANCH>
gitx timeline --path <PATH>
```

## 6. Commit

```bash
gitx commit <OID>
```

Should expose:

- metadata
- parents
- message
- changed files
- diff statistics
- classification
- affected contributors
- related history

## 7. File history

```bash
gitx history <PATH>
gitx history <PATH> --follow
gitx history <PATH> --since <DATE>
gitx history <PATH> --lines
gitx blame <PATH>
```

`--lines` / `blame` expose line-level history (which commit introduced or last changed each line). This is an expensive operation and must be computed lazily and paginated.

## 8. Branches

```bash
gitx branches
gitx branch <NAME>
```

Include:

- tip
- age
- ahead
- behind
- divergence
- activity
- shared files
- merge-risk indicators

## 9. Contributors

```bash
gitx contributors
gitx contributor <NAME>
gitx ownership
gitx ownership <PATH>
```

## 10. Hotspots

```bash
gitx hotspots
gitx hotspots --limit 20
gitx hotspots --path src/
gitx hotspots --json
```

## 11. Architecture

```bash
gitx architecture
gitx architecture --from <REF>
gitx architecture --to <REF>
gitx architecture diff <REF1> <REF2>
```

## 12. Dependencies

```bash
gitx dependencies
gitx dependencies history
gitx dependencies diff <REF1> <REF2>
```

## 13. Risk

```bash
gitx risk
gitx risk <PATH>
```

Risk output must show evidence.

## 14. Health

```bash
gitx health
gitx health --json
```

Shows the composite repository health score with all six sub-scores (code hotspots, ownership risk, branch hygiene, change volatility, architecture stability, recovery risk).

Every sub-score must display its evidence, not just a number.

## 15. Search

```bash
gitx search <QUERY>
gitx search <QUERY> --commits
gitx search <QUERY> --files
gitx search <QUERY> --authors
gitx search <QUERY> --branches
```

## 16. Recovery

```bash
gitx recovery
gitx recovery reflog
gitx recovery unreachable
gitx recovery show <OID>
```

Recovery commands must default to read-only behavior.

## 17. Releases

```bash
gitx release <TAG>
gitx release diff <REF1> <REF2>
```

## 18. Output contract

Human output is optimized for terminal readability.

JSON output is optimized for automation.

A command that supports JSON must not mix logs or decorative terminal output into stdout.

Use stderr for diagnostics.

## 19. Exit codes

Suggested:

```text
0  success
1  general error
2  invalid arguments
3  repository not found
4  not a Git repository
5  index unavailable/corrupt
6  unsupported operation
7  analysis incomplete
```

Exact codes should be stabilized before V1.
