# Configuration

## 1. Principles

Configuration should be:

- optional
- local
- human-readable
- versioned
- backward-compatible where practical

GitX should work with zero configuration.

## 2. Suggested location

Use the platform-appropriate user configuration directory.

Do not hardcode one operating system path.

## 3. Example configuration

```toml
[general]
default_limit = 50
color = "auto"

[index]
enabled = true
auto_refresh = true

[analysis]
hotspot_change_frequency_weight = 0.25
hotspot_recent_churn_weight = 0.20
hotspot_bug_fix_weight = 0.20
hotspot_ownership_weight = 0.15
hotspot_complexity_weight = 0.20

[ui]
theme = "default"
vim_keys = true

[search]
case_sensitive = false
```

Exact keys should be finalized during implementation.

## 4. Configuration precedence

Recommended:

```text
built-in defaults
    ↓
global config
    ↓
repository config
    ↓
environment variables where necessary
    ↓
CLI flags
```

## 5. Repository configuration

If repository-local configuration is supported, it must be clearly distinguished from Git's own configuration.

Do not silently modify `.git/config`.

## 6. Cache

Cache location should be configurable.

Provide commands:

```bash
gitx index status
gitx index clear
gitx index rebuild
```

## 7. No configuration required

A fresh user should be able to:

```bash
cd repository
gitx
```

and immediately use the tool.
