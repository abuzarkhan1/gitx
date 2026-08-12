# GitX

GitX is a **local-first, terminal-native Git repository intelligence and code archaeology CLI**. 

It turns your Git repository's history, structure, changes, ownership, branches, dependencies, and recovery information into a fast, interactive, explainable terminal experience.

## Overview

Unlike standard Git commands which just show you raw history, GitX acts as an intelligent layer on top of your local repository. By building a local, lightning-fast SQLite index of your codebase, GitX provides instant answers to complex code archaeology questions:

- **Where are the maintenance hotspots?** (Which files change the most and contain the most bugs?)
- **Who actually owns this module?** (Beyond just `git blame`, who has historically owned this code?)
- **What is the repository health?** (Analyzes churn, risk, and test coverage regressions).
- **How has the architecture evolved?** (Structural dependency graphing using tree-sitter).
- **What did we lose?** (Advanced reflog and unreachable commit recovery).

## Features

- ⚡️ **Lightning Fast**: Built in Rust. Scans and indexes your repository into a local SQLite database for instant querying.
- 📊 **Advanced Analytics**: Calculates Risk Scores, Change Frequencies, and Churn metrics using deterministic algorithms. 
- 🖥️ **Interactive TUI**: Comes with a gorgeous Terminal User Interface (`gitx-tui`) to explore the repository graphically without leaving your terminal.
- 🔍 **FTS5 Search**: Perform powerful full-text searches across commits, files, tags, and authors instantly.
- 🧠 **Explainable Intelligence**: No AI black boxes. Every score or warning exposes the raw git signals behind it.
- 🔧 **Machine Readable**: Every CLI command can output structured JSON for CI/CD pipeline integrations.

## Installation

GitX is split into two applications: the primary CLI (`gitx`) and the interactive dashboard (`gitx-tui`).

```bash
# Clone the repository
git clone https://github.com/abuzarkhan1/gitx.git
cd gitx

# Install the CLI tool
cargo install --path crates/gitx-cli

# Install the TUI dashboard
cargo install --path crates/gitx-tui
```

## Quick Start

### Using the CLI

To get started, simply navigate to any Git repository and build the local index:

```bash
cd my-project/
gitx scan
```

Once the repository is indexed, you can run any of the powerful analytical commands:

```bash
# Get high-level repository stats
gitx stats

# Find the highest-risk files in the codebase
gitx hotspots

# See the line-level historical lineage of a file
gitx lineage src/main.rs

# Get a composite health score for the repository
gitx health

# Output analytics as JSON for a script
gitx --json risk
```

### Using the TUI

If you prefer a visual dashboard, simply run the TUI in your terminal:

```bash
gitx-tui
```

Use `j` and `k` to navigate through the dashboard tabs (Overview, Timeline, Commits, Hotspots, Architecture, etc.), and hit `Enter` to select.

## License

MIT
