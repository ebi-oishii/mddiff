# Changelog

All notable changes to mddiff go here. Format roughly follows [Keep a Changelog](https://keepachangelog.com/).

## [0.2.1-alpha.1] — 2026-06-29

OS integration phase 1: `.md` / `.markdown` / `.mdown` / `.mkd` / `.mddiff` are now recognized file types on every platform.

### Added

- **File associations** — Installing the bundled dmg / msi / deb / AppImage registers mddiff as a handler for Markdown files. Finder's "Open With" menu, Windows Explorer's "Open With", and Linux's default-app pickers now offer mddiff. (#45)
- **Open-from-OS** — Double-clicking a `.md` file (or `open foo.md` / `xdg-open foo.md` / `mddiff foo.md` from the CLI) opens it in mddiff with the file already loaded. macOS "Open With" on a running mddiff also swaps the buffer to the new file live (via `RunEvent::Opened`). Windows / Linux runtime forwarding still spawns a new window — single-instance plugin is the Phase 2 follow-up.

[0.2.1-alpha.1]: https://github.com/ebi-oishii/mddiff/releases/tag/v0.2.1-alpha.1
[#45]: https://github.com/ebi-oishii/mddiff/pull/45

## [0.2.0-alpha.1] — 2026-06-29

The headline of this release: **diff → time-travel + local fine-grained history + blame**. v0.1 was "a markdown editor with first-class diff"; v0.2 lets you walk through past versions, see your own save-by-save history between commits, and read per-line authorship.

### Highlights

- **History view (time-travel)** — From any commit in the Diff picker, *View at this version* opens a read-only Preview of that revision. `←` / `→` walk through the file's history; *Restore this version* drops the historical text into your buffer (as an edit — `⌘Z` rolls it back, the file on disk isn't touched until you save). While in history view you can switch to Diff to compare the pinned version against *any* other revision. [#39]
- **Save-event snapshots** — Every successful save drops a zstd-compressed snapshot under your app data dir (`<app_data>/mddiff/snapshots/<path-hash>/<ts>.zst`). The Diff picker shows them as a *Recent local saves* group; you can diff against any snapshot or open one in the history view. Capped at 100 per file, dedup'd on identical content. [#42]
- **Blame view** — Fourth sub-mode in Diff. Each line gets a gutter showing `<sha8> · author · date`. Lines that aren't in HEAD are attributed to `local · <latest snapshot timestamp>` instead of git; lines that aren't even in a snapshot show as italic `local` with no date (unsaved buffer edits). [#43]
- **Outline sidebar** — Toggle with `⌘⇧O`. Click-to-jump TOC dockable on the right; works in all view modes. [#37]
- **3-group view consolidation** — The 5 modes (Source / Live / WYSIWYG / Preview / Diff) collapse to 3 top-level groups: **Live Preview** (`⌘1`, sub = Source via `⌘⇧1`), **WYSIWYG** (`⌘2`, sub = Preview via `⌘⇧2`), and **Diff** (`⌘3`). Right pane in split mode gets `⌥⌘1/2/3`. [#40]
- **Native spellcheck** — OS-native red-underline spellcheck for Source / Live Preview / WYSIWYG. Code, URLs, and markdown syntax (`#`, `>`, `**`, etc.) are masked out so they don't get flagged. Toggleable in Settings. [#38]
- **Image clipboard paste** — Paste an image into any editable view; it's written to `<file>.assets/` and a Markdown image reference is inserted. Live Preview shows a `🖼 filename` pill for inactive lines; WYSIWYG renders the actual image. [#31]

### Smaller improvements

- Split pane (`⌘\`) now toggled from the menu with `⌥⌘N` shortcuts for the right pane mode. Per-pane fullscreen title pills. Right pane's mode bar shows the group + sub-toggle + close. [#33]
- ⌘/Ctrl-click follows links in editable views; modifier-down adds a pointer cursor on every link. Link-only Markdown opens in a new mddiff window when it points to another markdown file. [#34]
- Export menu items have shortcuts: `⌘⇧H` (HTML), `⌘⇧P` (PDF), `⌘⇧T` (text), `⌘⇧D` (docx), `⌘⇧M` (.mddiff). [#35]
- Source view scrollbar reaches the right edge; active-line highlight extends to the right padding strip. [#36]
- DiffView picker: history view's Diff mode shows a `[pinned: HEAD~3]` chip next to `vs` so you know which side is fixed. [#39]
- Edit-locked dialog when switching to an editor mode from history view — explains restore is needed and warns if the buffer has unsaved edits. [#39]
- `Esc` exits history view; menu also has a dedicated *Exit history view* entry. [#39]

### Breaking-ish (alpha → alpha, but worth flagging)

- **Keyboard shortcuts changed**: `⌘1` was Source, now it's Live Preview group's main. To get to Source, use `⌘⇧1` (or pick Source from the in-menu group entry).
- **Native View menu reorganized** to match the 3-group structure: Live Preview / [Source sub] / WYSIWYG / [Preview sub] / Diff.

### Settings additions

- `outlineOpen` (default `false`) — outline sidebar visibility persists across sessions
- `spellcheck` (default `false`) — Browser/OS spellcheck on editable views

Both default off; existing settings localStorage is forward-compatible (unknown keys merged with defaults).

### File / format compatibility

- `.mddiff` pack format unchanged from v0.1.
- Snapshot directory layout (`<app_data>/mddiff/snapshots/...`) is new; no migration needed.

### Internal

- Cargo workspace now has a `blame` and a `history` module in `mddiff-core`. TUI binary doesn't surface them yet — GUI-only for v0.2.
- 30+ commits since v0.1.0-alpha.1; see `git log v0.1.0-alpha.1..v0.2.0-alpha.1` for the full picture.

[0.2.0-alpha.1]: https://github.com/ebi-oishii/mddiff/releases/tag/v0.2.0-alpha.1
[#31]: https://github.com/ebi-oishii/mddiff/pull/31
[#33]: https://github.com/ebi-oishii/mddiff/pull/33
[#34]: https://github.com/ebi-oishii/mddiff/pull/34
[#35]: https://github.com/ebi-oishii/mddiff/pull/35
[#36]: https://github.com/ebi-oishii/mddiff/pull/36
[#37]: https://github.com/ebi-oishii/mddiff/pull/37
[#38]: https://github.com/ebi-oishii/mddiff/pull/38
[#39]: https://github.com/ebi-oishii/mddiff/pull/39
[#40]: https://github.com/ebi-oishii/mddiff/pull/40
[#42]: https://github.com/ebi-oishii/mddiff/pull/42
[#43]: https://github.com/ebi-oishii/mddiff/pull/43

## [0.1.0-alpha.1] — 2026-06-28

Initial alpha. 5 view modes (Source / Live / WYSIWYG / Preview / Diff), first-class Git diff with 3 sub-modes, `.mddiff` portable pack format, TUI parity, Tauri 2 GUI on macOS / Windows / Linux.

[0.1.0-alpha.1]: https://github.com/ebi-oishii/mddiff/releases/tag/v0.1.0-alpha.1
