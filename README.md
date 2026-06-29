# mddiff

> **Alpha · v0.2.1-alpha** — APIs / file format / settings may still change before v1. Feedback welcome via issues.

**A Markdown editor with first-class Git diff and time-travel.** Like Typora, but with `git diff` and `git checkout` living next to your text.

🇯🇵 日本語版: [README.ja.md](README.ja.md)

![mddiff main view](docs/img/hero.png)

## What it is

For people who write docs (READMEs, design docs, notes, ADRs) inside Git repos. mddiff keeps the writing experience close to Typora / Obsidian but makes the two questions you ask most often — *"what did I change?"* and *"what did this look like last month?"* — one click away from the cursor.

## Why mddiff

### Diff is first-class, not a plugin

![Side-by-side diff](docs/img/diff-sbs.png)

- Pick **any Git revision** as the comparison base (branch, tag, commit, custom revspec) — or compare against the on-disk version
- Three sub-modes: **Highlight Only**, **Full unified**, **Side-by-Side** with synchronized sub-block scroll
- The commit picker hides commits that didn't touch this file (toggleable)
- External-change detection: when the file changes outside the editor, jump into Diff straight from a banner

### Time-travel through versions

![History view](docs/img/history-view.png)

- **"View at this version"** from any commit in the picker → read-only Preview of the historical file
- `←` / `→` walk through the file's history one commit at a time
- **`Restore this version`** drops the historical text back into your buffer (as an edit — `⌘Z` undoes, the file on disk isn't touched until you save)

While viewing history, switch to **Diff** to compare the pinned version against *any* other revision — useful for "did this section change between v1 and v2?"

![History + Diff](docs/img/history-diff.png)

### Three view groups, one keyboard

Three top-level modes, each with a read-only / raw sub-variant:

| Group | Main mode | Sub-mode | Shortcut |
|---|---|---|---|
| Live Preview | Live Preview (Typora-style) | Source (raw markdown) | `⌘1` / `⌘⇧1` |
| WYSIWYG | WYSIWYG (Milkdown) | Preview (read-only HTML) | `⌘2` / `⌘⇧2` |
| Diff | Diff | — | `⌘3` |

Plus an **outline / TOC sidebar** (`⌘⇧O`) and **split-pane** (`⌘\`) for editing one mode while previewing another.

![Outline sidebar](docs/img/outline.png)

### GUI + TUI parity

Both share the same `mddiff-core` crate. The TUI ships as a **single ~2.3 MB binary** with vim-style `:w :q :wq` and the same Diff sub-modes.

### `.mddiff` portable bundle

Pack a Markdown file's Git history into one zstd-compressed HTML comment. The receiver sees plain Markdown without Git installed; mddiff users replay the history.

### Lightweight

Tauri 2 — no bundled Chromium / Node. **~80 MB idle RAM on macOS, ~0.5 s cold start.** Roughly 1.5–3× lighter than Electron-based competitors.

## Install

### Pre-built binaries

Grab the latest from [Releases](https://github.com/ebi-oishii/mddiff/releases):

| Platform | File |
|---|---|
| macOS (Apple Silicon) | `mddiff_0.2.1_aarch64.dmg` |
| macOS (Intel) | `mddiff_0.2.1_x64.dmg` |
| Windows | `mddiff_0.2.1_x64-setup.msi` |
| Linux | `mddiff_0.2.1_amd64.AppImage` |
| TUI (any platform) | `mddiff-tui_0.2.1_<target>` |

### From source

Requires Rust 1.75+, Node 22+, and [Tauri prerequisites](https://tauri.app/start/prerequisites/).

```sh
git clone https://github.com/ebi-oishii/mddiff
cd mddiff
npm install
npm run tauri build              # → target/release/bundle/
cargo build --release -p mddiff-tui   # → target/release/mddiff-tui
```

## Keyboard reference

| Shortcut | Action |
|---|---|
| `⌘1` / `⌘⇧1` | Live Preview / Source |
| `⌘2` / `⌘⇧2` | WYSIWYG / Preview |
| `⌘3` | Diff |
| `⌘\` | Toggle split pane |
| `⌘⇧O` | Toggle outline sidebar |
| `⌘O` / `⌘S` / `⌘⇧S` | Open / Save / Save As |
| `⌘⇧R` | Reload from disk |
| `⌘,` | Settings |
| `Esc` (in history view) | Exit history view |

On Linux / Windows, `Ctrl` replaces `⌘`.

## Status

**Alpha** — feature-complete enough for daily use, but expect format / settings drift before v1.

Known limitations:
- `.mddiff` file format may change incompatibly before v1
- Settings localStorage schema may reset between releases
- iOS / Android targets are scaffold-only
- Diff base picker labels are English-only

Roadmap candidates (see [docs/issues.md](docs/issues.md)):
- Mermaid diagrams, KaTeX math, spell check
- Find & Replace: regex / case-sensitivity
- git blame gutter
- Image clipboard paste / drag-drop, outline sidebar, link-click navigation, time-travel history — **done**

## Architecture

Cargo workspace, 3 crates:

```
mddiff/
├── crates/mddiff-core/   # UI-independent logic (diff, git, fs, pack format)
├── crates/mddiff-tui/    # ratatui-based terminal UI
└── src-tauri/            # Tauri 2 shell + Svelte 5 + CodeMirror / Milkdown
```

Details: [docs/design.md](docs/design.md) · ADRs: [docs/decisions.md](docs/decisions.md) · Competitive positioning: [docs/competitive-analysis.md](docs/competitive-analysis.md)

## Development

```sh
npm install
npm run tauri dev                  # GUI desktop dev
npm run tauri android dev          # Android (NDK + Android Studio + JAVA_HOME)
npm run tauri ios dev              # iOS (Xcode + cocoapods)

cargo run -p mddiff-tui -- README.md            # TUI
cargo run -p mddiff-tui -- --diff-base HEAD~3 README.md

npm run check              # Svelte + TypeScript
cargo check --workspace    # Rust
cargo test  --workspace    # Rust tests
```

## License

MIT — see [LICENSE](LICENSE).

## More docs

- [docs/design.md](docs/design.md) — architecture & per-mode design
- [docs/decisions.md](docs/decisions.md) — technical ADRs
- [docs/roadmap.md](docs/roadmap.md) — phases & status
- [docs/competitive-analysis.md](docs/competitive-analysis.md) — competitive landscape
- [docs/mddiff-protocol.md](docs/mddiff-protocol.md) — `.mddiff` pack format
- [docs/issues.md](docs/issues.md) — open issues / polish candidates
