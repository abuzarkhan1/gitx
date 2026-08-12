# CLI Specification

## 1. Invocation

```bash
gitx
gitx tui
```

Opening without a subcommand starts the interactive TUI (docs/01 UC-01,
docs/16 §7); `gitx tui` launches it explicitly. When stdout is not a
terminal (pipes, CI), `gitx` prints a compact repository snapshot instead
of launching an interactive session.

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

`--lines` / `blame` expose line-level history (which commit introduced or last changed each line). This is an expensive operation and must be computed lazily and paginated; `gitx blame <PATH> --limit N` (default 500) pages the output.

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
gitx architecture --from <REF> --to <REF>
gitx architecture diff <REF1> <REF2>
```

`--from` and `--to` are equivalent to `architecture diff <from> <to>` and must
be provided together.

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
gitx search <QUERY> --renames
gitx search <QUERY> --code
gitx search <QUERY> --history
gitx search <QUERY> --since <DATE>
gitx search <QUERY> --author <NAME>
```

`--since` accepts RFC3339, `YYYY-MM-DD`, or unix seconds and filters commit
results. `--author` matches author name/email substrings.

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
gitx release show <TAG>
gitx release diff <REF1> <REF2>
```

`gitx release <TAG>` is shorthand for `gitx release show <TAG>`.

## 17.5 Diff

```bash
gitx diff <REF1> <REF2>
gitx diff <REF1> <REF2> --path <PREFIX>
gitx diff <REF1> <REF2> --stat
```

Unified diff between any two refs (branch, tag, or commit id), processed
file-by-file so only one file's hunks are in memory at a time (docs/13 §8).
On a TTY the output pages through `less -R` (docs/25); piped output prints
directly. `--stat` prints the file list with insertions/deletions and pages
like other long output.

## 18. Output contract

Human output is optimized for terminal readability.

JSON output is optimized for automation.

A command that supports JSON must not mix logs or decorative terminal output into stdout.

Use stderr for diagnostics.

## 19. Exit codes

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
