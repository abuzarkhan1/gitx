# Benchmarks

Criterion benchmarks live inside the crate that owns the hot code path
(criterion requires a package context), rather than at the workspace root:

- `crates/gitx-analysis/benches/` — hotspot scoring, commit classification

Run all benchmarks with:

```bash
cargo bench --workspace
```
