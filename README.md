# mddiff

> **Alpha · v0.1.0-alpha** — APIs / file format / settings may change before v1. Feedback welcome via issues.

A lightweight, cross-platform Markdown editor with **first-class diff**. One Rust core, one Tauri 2 + Svelte 5 GUI, and one ratatui TUI binary.

🇯🇵 日本語版: [README.ja.md](README.ja.md)

## What sets it apart

- **5 editing modes** in the same window:
  - **Source** — CodeMirror 6 with markdown syntax highlight, line numbers, soft wrap, configurable tab width
  - **Live Preview** — Typora-style: the line you're editing stays raw, others render inline
  - **WYSIWYG** — Milkdown / ProseMirror rich editing with click-to-toggle task lists
  - **Preview** — read-only rendered HTML
  - **Diff** — compare against any Git revision, or against the disk version
- **Diff is 1st-class, not a plugin**:
  - 3 sub-modes (Highlight Only / Full unified / Side-by-Side with synchronized scroll)
  - Base picker filters to commits that actually changed this file
  - Switch any time from a banner if the file changed externally
- **GUI + TUI feature parity** — same `mddiff-core` crate. TUI ships as a single ~2.3 MB binary with vim-style `:w :q :wq` commands and the same Diff sub-modes
- **`.mddiff` portable bundle** — pack a Markdown file's Git history into one zstd-compressed HTML comment. Receiver reads it as plain Markdown without Git; the mddiff app can replay the history
- **Light footprint** — Tauri 2 means no bundled Chromium / Node. Idle RAM ≈ 80 MB on macOS, cold start ≈ 0.5 s. 1.5–3× lighter than Electron-based competitors

## Install

### Pre-built binaries (recommended)

Download from the [Releases](https://github.com/ebi-oishii/mddiff/releases) page:

| Platform | File |
|---|---|
| macOS (Apple Silicon) | `mddiff_0.1.0_aarch64.dmg` |
| macOS (Intel) | `mddiff_0.1.0_x64.dmg` |
| Windows | `mddiff_0.1.0_x64-setup.msi` |
| Linux | `mddiff_0.1.0_amd64.AppImage` |
| TUI (any platform) | `mddiff-tui_0.1.0_<target>` |

### From source

Requires: Rust 1.75+, Node 22+, platform deps for Tauri ([Tauri prerequisites](https://tauri.app/start/prerequisites/)).

```sh
git clone https://github.com/ebi-oishii/mddiff
cd mddiff
npm install
npm run tauri build              # produces dmg / msi / AppImage in target/release/bundle/
cargo build --release -p mddiff-tui   # TUI single binary
```

## Status & roadmap

This is **alpha**: feature-complete enough to use daily, but rough edges and format/setting changes are possible before v1.

**Known limitations**:
- `.mddiff` file format may change incompatibly before v1
- Settings localStorage schema may reset between releases
- Mobile (iOS / Android) is scaffold-only; expect bugs
- Diff base picker uses English-only labels (other UI is localized)

**Near-term roadmap** (see [docs/issues.md](docs/issues.md) for details):
- Image clipboard paste / drag-drop to `.assets/`
- TOC / outline sidebar
- Mermaid diagram rendering
- KaTeX inline math
- Spell check
- Find & Replace: regex / case-sensitivity toggles
- git blame gutter

## Architecture

Cargo workspace, 3 crates:

```
mddiff/
├── crates/mddiff-core/   # UI-independent logic (diff, git, fs, pack format)
├── crates/mddiff-tui/    # ratatui-based terminal UI
└── src-tauri/            # Tauri 2 shell + Svelte 5 + CodeMirror / Milkdown
```

See [docs/design.md](docs/design.md) for the full architecture, [docs/decisions.md](docs/decisions.md) for ADRs, and [docs/competitive-analysis.md](docs/competitive-analysis.md) for the competitive positioning.

## Development

### GUI (Tauri)

```sh
npm install
npm run tauri dev                  # Desktop dev mode
npm run tauri android dev          # Android (requires Android Studio + NDK + JAVA_HOME)
npm run tauri ios dev              # iOS (requires Xcode + cocoapods)
```

First-time mobile setup: `npm run tauri ios init` / `android init`. iOS additionally needs `brew install cocoapods`.

### TUI

```sh
cargo run -p mddiff-tui                              # no file
cargo run -p mddiff-tui -- README.md                 # open a file
cargo run -p mddiff-tui -- --diff-base HEAD~3 README.md   # diff against any revspec
cargo run -p mddiff-tui -- --read-only README.md           # read-only mode
```

### Quality checks

```sh
npm run check              # Svelte + TypeScript
cargo check --workspace    # Rust, all crates
cargo test  --workspace    # Rust tests
```

## License

MIT — see [LICENSE](LICENSE).

## Documentation

- [docs/design.md](docs/design.md) — architecture and per-mode design
- [docs/decisions.md](docs/decisions.md) — technical ADRs
- [docs/roadmap.md](docs/roadmap.md) — phases and status
- [docs/competitive-analysis.md](docs/competitive-analysis.md) — competitive landscape and positioning
- [docs/refactor-audit.md](docs/refactor-audit.md) — internal code quality audit
- [docs/mddiff-protocol.md](docs/mddiff-protocol.md) — `.mddiff` portable package format v1
- [docs/issues.md](docs/issues.md) — open issues / polish / feature candidates
