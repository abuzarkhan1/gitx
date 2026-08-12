# Releasing GitX

How to ship a new GitX release. This is the operational companion to
[18-RELEASE-ENGINEERING.md](./18-RELEASE-ENGINEERING.md).

## Versioning

Follow Semantic Versioning (`MAJOR.MINOR.PATCH`). The application version
lives in `Cargo.toml` (`[workspace.package] version`) and is inherited by
every crate (`version.workspace = true`).

**`Cargo.lock` is committed** (binary workspace; `--locked` installs in
docs/18 §9 depend on it). Bump it deliberately with `cargo update` and always
verify `cargo build --locked` in the checklist.

The **SQLite index schema is versioned independently** (`schema_version` in
`index_metadata`, migrations in `migrations/`). Bumping the app version never
forces an index migration, and vice versa. Incompatible index formats must be
detected → explained → migrated or rebuilt, never silently reinterpreted.

## Checklist

1. **Quality gate** — run `./scripts/check.sh` (fmt + check + test). CI runs
   the same plus `clippy -D warnings`, a cross-platform build matrix, and an
   end-to-end CLI smoke test (`.github/workflows/ci.yml`).
2. **Changelog** — move unreleased entries in `CHANGELOG.md` under the new
   version heading.
3. **Bump version** — update `[workspace.package] version` in `Cargo.toml`,
   then `cargo check --workspace` so `Cargo.lock` refreshes.
4. **Tag** — `git tag v0.2.0` (annotated) and push the tag. This triggers
   `.github/workflows/release.yml`: preflight checks → per-platform release
   builds (macOS arm64/x86_64, Linux, Windows) → SHA-256 checksums →
   GitHub Release with notes from `CHANGELOG.md`.
5. **Verify the release** — the GitHub Release contains, per platform:
   the `gitx` and `gitx-tui` binaries, `checksums.txt`, and release notes
   (docs/18 §5).
6. **Installers** — run `cargo dist plan` (requires `cargo install
   cargo-dist`) and confirm the shell/powershell/homebrew installers are
   listed; copy the generated install commands into docs/18 §9 if the tap or
   URL shape changed.
7. **Document installation** — keep `README.md`, `docs/18 §9` and
   `docs/22-CONTRIBUTING.md` current: binary download, package-manager
   installers, source build, shell completions.

## Local dry run

cargo-dist can preview the plan without uploading:

```bash
cargo dist plan
```

Requires `cargo install cargo-dist` (see
[cargo-dist docs](https://opensource.axo.dev/cargo-dist/)).

## 0.1.0 runbook

The first public release (crates.io publishes + installers). Concrete steps:

1. Confirm `scripts/check.sh` and `scripts/verify-tui.sh` are green, and
   `cargo package` succeeds for the two dependency leaves (gitx-core,
   gitx-index — dependent crates are validated after the leaves publish;
   see the plan in `docs/superpowers/plans/2026-08-13-product-hardening.md`).
2. Publish crates bottom-up (leaf dependencies first; derive the order with
   `cargo tree -p gitx-cli --edges normal | grep gitx-`):
   `cargo publish -p gitx-core` → `gitx-index` → `gitx-storage` → `gitx-git`
   → `gitx-history` → `gitx-analysis` → `gitx-graph` → `gitx-search` →
   `gitx-services` → `gitx-cli` → `gitx-tui`.
3. Tag and push: `git tag -a v0.1.0 -m "GitX 0.1.0" && git push origin v0.1.0`.
4. `.github/workflows/release.yml` builds the four-target installers,
   checksums, and the GitHub Release (notes from `CHANGELOG.md`).
5. Smoke-test the release binaries with `scripts/release-check.sh`.
6. Publish the Homebrew tap formula if the tap repository exists
   (`brew tap abuzarkhan1/tap`), and copy the installer commands into
   `README.md` and `docs/18 §9`.

Steps 2–6 require crates.io API tokens and GitHub push access — they are
maintainer actions, not part of the automated CI.
