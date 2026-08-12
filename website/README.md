# GitX website

A terminal-themed marketing site for [GitX](../README.md) — the
local-first, terminal-native Git repository intelligence tool.

## Pages

- `/` — hero + how to use (typed `gitx` demo, command help, install steps)
- `/about` — what it is, the 11-crate workspace, the pipeline
- `/contact` — open an issue, pre-filled via the GitHub issue form

## Design

Pure terminal aesthetic: green (`#33ff66`) and amber (`#ffb000`) on flat
black. Monospace system fonts, no gradients, no rounded cards, no shadows.

## Development

```bash
npm install
npm run dev      # http://localhost:3000
npm run build    # static export → out/
```

The build produces a fully static site (`output: "export"`) — host the
`out/` directory anywhere.
