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

GitX ships two binaries: `gitx` (CLI) and `gitx-tui` (terminal UI).

### Binary download

Tagged releases publish prebuilt binaries for macOS (arm64/x86_64), Linux
(arm64/x86_64) and Windows, plus a `checksums.txt` with SHA-256 hashes (see
docs/27 §3 for the exact release flow). Download the archive for your
platform from the GitHub Release page, verify it against `checksums.txt`, and
put the binary on your `PATH`, e.g.:

```bash
# macOS (arm64)
curl -sSLo gitx.tar.gz https://github.com/USER/gitx/releases/latest/download/gitx-aarch64-apple-darwin.tar.gz
shasum -a 256 -c checksums.txt   # or: sha256sum -c checksums.txt
sudo tar -xzf gitx.tar.gz -C /usr/local/bin
```

Verify the installation:

```bash
gitx --version
gitx-tui --version
gitx completions bash > /tmp/gitx.bash   # optional, see below
```

### Package-manager installation

Tagged releases publish cargo-dist installers alongside the archives (docs/27
checklist step 4). The exact URLs follow the repository owner; with the
`USER` placeholder used throughout this doc:

- **curl installer (macOS/Linux):**

  ```bash
  curl -LsSf https://github.com/USER/gitx/releases/latest/download/gitx-installer.sh | sh
  ```

- **PowerShell installer (Windows):**

  ```powershell
  irm https://github.com/USER/gitx/releases/latest/download/gitx-installer.ps1 | iex
  ```

- **Homebrew:** cargo-dist publishes a tap at `github.com/USER/homebrew-gitx`;
  install both binaries with:

  ```bash
  brew install USER/gitx/gitx
  ```

- **Cargo (source build):**

  ```bash
  cargo install --path crates/gitx-cli --locked
  cargo install --path crates/gitx-tui --locked
  ```

Verify any install with `gitx --version` and `gitx-tui --version`, and
regenerate the plan before a release with `cargo dist plan` so the installer
URLs stay accurate (docs/27 checklist).

### Source build

Requires a Rust toolchain (stable, per docs/03).

```bash
git clone <repo-url> gitx && cd gitx
cargo build --release --workspace
# binaries land in target/release/gitx and target/release/gitx-tui
cp target/release/gitx target/release/gitx-tui /usr/local/bin/
```

Or install both binaries directly:

```bash
cargo install --path crates/gitx-cli --locked
cargo install --path crates/gitx-tui --locked
```

### Shell completion

`gitx completions <shell>` emits completion scripts for bash, zsh, fish and
PowerShell (docs/07 §20):

```bash
gitx completions bash  > /usr/local/etc/bash_completion.d/gitx   # bash
gitx completions zsh   > "${fpath[1]}/_gitx"                    # zsh
gitx completions fish  > ~/.config/fish/completions/gitx.fish   # fish
```

(Requires the completed command names; the TUI binary `gitx-tui` has no
separate CLI surface to complete.)
