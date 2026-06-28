# Refactor Audit Synthesis — branch `test/p1-p8`

調査日: 2026-06-28
方法: 21 agents による並列 audit (10 dimension finders → 10 adversarial verifiers → 1 synthesizer)
対象: `test/p1-p8` ブランチ (14 in-flight PRs locally merged の状態)

## TL;DR

**Recommendation: `refactor-selectively`.**

This audit covers a post-feature-rush codebase that's in good shape. Most findings are genuine duplication from sprint pace, not architectural decay. The signal-to-noise ratio is high — the auditor was honest about cost and dropped speculative findings (CSS dimension consolidation, IS_MAC extraction, ModeBar revival).

The 5 highest-leverage moves all reduce friction for the **next three features** (image paste, TOC, Mermaid):

1. **`useFind` composable** — 7 sites collapse to 1 line each; image paste preview will want find.
2. **`scroll-tracker` helper** — 4 sites collapse; TOC needs precise scroll-line tracking.
3. **`createPreviewMd` factory** — Mermaid is a markdown-it plugin; doing the factory before adding it means 1 extension point not 3 drift opportunities.
4. **MODE_ENTRIES dispatch consolidation** — 3 sites to 1; bonus: closes the `mode=diff` guard inconsistency.
5. **`humanizeError` on the 4 leaking catches** — user-visible quality fix; 4-line mechanical change.

We defer: the external-change watcher store extraction (real but no near-term feature pulls on it), the SettingsDialog schema-driven rewrite (working code, risky rewrite, no pressure), and the Rust mdv-core decomposition (clean wins but modest risk on hot paths with no third consumer yet).

---

## Refactoring (shell)

| Title | Sev | Eff | Risk | Decision |
|---|---|---|---|---|
| External-change watcher is a self-contained feature wrongly hosted in the shell | med | M | med | **Later** |
| Native menu dispatch is a 50-line switch — replace with a lookup table | med | S | low | **Now (PR 4)** |
| Keyboard shortcuts duplicate mode-switching logic and the menu shortcut hints | med | S | low | **Now (PR 4)** |
| Effect at 536-542 enforcing diff-mode validity should be `$derived` | low | S | low | Later (covered by setMode helper in PR 4) |
| Banners section is 5 near-identical structures — extract a Banner component | low | S | low | Later |
| Helper functions `basename` and `modeLabel` belong in `$lib` | low | S | low | **Now (PR 5)** |
| Dead-code: ModeBar.svelte is never imported | low | S | low | **Now (PR 5)** |
| Window-chrome concerns (fullscreen, title sync, close-confirm) | low | M | med | **Leave** — not growing, no test wins |
| Effect at 544-547 silently clearing normalization is partially dead reactivity | low | S | low | Later |
| isMac / MOD / SHIFT constants belong in `$lib/platform.ts` | low | S | low | **Leave** — single consumer today |

**Notes.** The window-chrome extraction is the kind of refactor that moves complexity without reducing it — kept out. Banner component is worth doing when banner count grows past 6, not on the strength of today's 5.

---

## Duplication

| Title | Sev | Eff | Risk | Decision |
|---|---|---|---|---|
| Scroll-debounce + captureTopLine + onDestroy last-chance: 4× | high | M | low | **Now (PR 2)** |
| currentLine restore on mount: identical CM block in Source and LivePreview | med | S | low | **Now (PR 2)** |
| CM6 EditorView mount/teardown + lastEmitted echo gate: Source + LivePreview | med | M | med | Later (echo-gate invariants need care) |
| FindState wiring duplicated in 7 views | med | M | low | **Now (PR 1)** |
| `$effect` for find refresh × 7 | low | S | low | **Now (PR 1, subsumed)** |
| data-mdv-line markdown-it pipeline: Preview + SideBySide + Wysiwyg | med | M | low | **Now (PR 3)** |
| Compartment dispatch effects in SourceView: 3 near-identical `$effect`s | low | S | low | Later (local-only cleanup) |
| Fullscreen 2.5rem top-padding × 5 | low | S | low | **Leave** — selectors differ, value-only consolidation |
| Preview-block CSS rules × 3 | low | M | med | **Leave** — dimensions diverge legitimately |
| Keep: `livepreview.ts` correctly isolated | — | — | — | Documentation |
| Keep: DiffView is single-use shell — do not abstract | — | — | — | Documentation |

**Notes.** Find + scroll + markdown render are the three abstractions paying back the fastest. The CSS findings were already correctly flagged as "lowest priority" — going further and dropping them entirely is the right call.

---

## Find/Replace audit

| Title | Sev | Eff | Risk | Decision |
|---|---|---|---|---|
| DOM finder case-insensitive, CM finder case-sensitive — opposite semantics | high | S | low | **Now (PR 5)** |
| DomFinder highlights gutter line numbers in diff views | med | S | low | **Now (PR 5)** |
| No shared FindController interface — two classes mirror by convention | med | S | low | Later |
| Window keydown install/teardown repeated in 7 hosts | low | S | low | Later (subsumed by `useFind` in PR 1) |
| FindBar's `enableReplace` + `?.()` chaining duplicate intent | low | S | low | Later |

**Notes.** The case-sensitivity mismatch is a user-visible bug fixed in one line — easy win. The shared interface is genuinely useful but lower-priority than the composables that consume it.

---

## Settings store + dialog

| Title | Sev | Eff | Risk | Decision |
|---|---|---|---|---|
| 10 individual `$state` fields force 5-touch-points per setting | med | S | low | Later (paired with type-safety fix) |
| `persistChange` uses `@ts-expect-error` + generic admits methods | med | S | low | Later (depends on first) |
| Dialog body is ~160 lines of identical row markup | low | M | med | **Leave** — defer until 15+ settings |

**Notes.** F4-F8 in docs/issues.md are TUI features — no near-term pressure to grow the Settings dialog. Wait for the Lua-hook config story to actually start landing.

---

## IPC and error handling

| Title | Sev | Eff | Risk | Decision |
|---|---|---|---|---|
| Raw Rust error strings leak to UI in 4 catch blocks | med | S | low | **Now (PR 5)** |
| ExternalChange path filter races on rapid file swap | low | S | low | **Now (PR 5)** |
| `readFile` and `readPath` duplicate identical post-read logic | low | S | low | Later |

**Notes.** humanizeError on the 4 sites is the highest user-visible quality fix in the audit. The race window in handleExternalChange is small but real; fix is surgical.

---

## Svelte 5 runes & state

| Title | Sev | Eff | Risk | Decision |
|---|---|---|---|---|
| Direct buffer mutation from +page.svelte bypasses DocStore methods | med | S | low | **Now (PR 5)** |
| `setTimeout` closures for banner auto-dismiss don't cancel previous timers | low | S | low | Later |
| Global keyboard-shortcuts `$effect` has no reactive deps; should be onMount/onDestroy | low | S | low | Later |
| DiffView reaches into doc store directly while peers receive props | low | S | low | Later (comment-only fix) |

**Notes.** The DocStore method additions are the right architectural move and small enough to bundle with PR 5.

---

## Rust src-tauri organization

| Title | Sev | Eff | Risk | Decision |
|---|---|---|---|---|
| Repeated `cfg(not(any(target_os = "android"...)))` × 10 — use `cfg(desktop)` | low | S | low | Later |
| `write_binary_file` doesn't mark self-write — latent bug | low | S | low | Later |
| Async signatures on CPU/blocking commands have no `.await` | low | S | low | Later |

**Notes.** All low-risk but no near-term pressure. Bundle into a "rust hygiene" PR when someone next touches the IPC layer.

---

## mdv-core library

| Title | Sev | Eff | Risk | Decision |
|---|---|---|---|---|
| `DocState` is dead code — re-exported but never consumed | med | S | low | Later (one-line delete) |
| GitError and PackError are 80% identical — factor RepoError | med | M | med | Later |
| `canonicalize_lossy` duplicated verbatim | low | S | low | Later (paired) |
| Repo-relative-path resolution open-coded in three places | med | M | med | Later (paired) |
| `git.rs` mixes three concerns in 401 lines — split picker out | med | M | low | Later |
| `SideBySidePayload` lives in git.rs but is git-agnostic | low | S | low | Later (paired) |
| `pack.rs` welds bundle build to extract_body — split for mobile | low | S | low | Later |
| **Hardcoded author 'ebi-oishii' in `pack()`** | med | S | low | **Now (PR 6)** |
| `DiffMarker::Unknown` doc and behavior disagree | low | S | low | **Now (PR 7)** |
| `git.rs` public API has no rustdoc | med | M | low | Later |

**Notes.** The hardcoded author leak is a real bug shipping to every `.mdv` file — fix before more users adopt the format. The DiffMarker doc fix is trivial and can ride with any git.rs touch. The bigger Rust reorganization (RepoError, git split, pack split) is a clean wins-pile but no third consumer (CLI/LSP) is pulling on it.

---

## TypeScript safety

| Title | Sev | Eff | Risk | Decision |
|---|---|---|---|---|
| `@ts-expect-error` in `persistChange` enables lying casts | med | S | low | Later (paired with settings consolidation) |
| Local `interface Token` shadows real markdown-it Token | low | S | low | Later |
| `JSON.parse(raw) as Partial<Settings>` trusts localStorage | med | M | low | Later |
| Triple cast on `@turbodocx/html-to-docx` | low | S | low | Later |
| `Array.from(root.children)` cast to `HTMLElement[]` in Wysiwyg | low | S | low | Later |
| Enable `noImplicitReturns` and `noFallthroughCasesInSwitch` | low | S | low | Later |

**Notes.** All low/med severity, none blocking. Bundle into a "type hygiene" PR when settings consolidation lands.

---

## Dead/unused code

| Title | Sev | Eff | Risk | Decision |
|---|---|---|---|---|
| ModeBar.svelte fully orphaned — delete it | med | S | low | **Now (PR 5)** |
| `addedCount()` in types.ts never called — delete | low | S | low | **Now (PR 5)** |
| Duplicate `.row input[type="checkbox"]` CSS rule | low | S | low | **Now (PR 5)** |
| Indirection-only `label()` helper in MdvExportDialog | low | S | low | Later |

**Notes.** Three trivial deletions to ride in PR 5. The MdvExportDialog `label()` indirection is so trivial it can wait — and might earn its keep if a marker glyph gets added like DiffView's `optionLabel`.

---

## Suggested PR sequence

| # | PR | Effort | Risk | What lands |
|---|---|---|---|---|
| 1 | `chore/useFind-composable` | M | low | 7-site collapse, no more keydown leak risk |
| 2 | `chore/scroll-tracker` | M | low | 4-site collapse, restore-rAF invariant documented once |
| 3 | `chore/markdown-render-factory` | M | low | Single MarkdownIt config seam (ready for Mermaid) |
| 4 | `chore/mode-registry-dispatch` | S | low | Menu + shortcuts driven from MODE_ENTRIES |
| 5 | `chore/quick-cleanups` | S | low | humanizeError + DOM finder fixes + dead code + small bugs |
| 6 | `chore/mdv-pack-author` | S | low | Real bug fix: no more hardcoded maintainer name |
| 7 | `chore/diff-marker-doc` | S | low | Trivial doc alignment |

After PR 1-3 the next features (image paste, TOC, Mermaid) have a much cleaner runway. Stop here unless one of the "later" items becomes a blocker.

## Items explicitly left as-is (with reasoning)

- **Window-chrome extraction**: ~25 lines across 3 effects, each guarded against non-Tauri, not growing. Extraction moves complexity without reducing it. No tests would benefit.
- **`IS_MAC` / `MOD_KEY` to `$lib/platform.ts`**: only one consumer today. Pull out when a second appears — listed only to pair with that future work.
- **Fullscreen 2.5rem top-padding CSS-vars consolidation**: the 5 selectors genuinely differ. Consolidation collapses a magic number, not real duplication. Defer until the title overlay height changes or a 6th view is added.
- **Preview-block CSS rules across 3 views**: dimensions diverge in 8+ ways. A CSS-vars extraction would create an abstraction whose every call site overrides 4-6 variables — that's not abstraction earning its keep. Revisit only during a forced cross-cutting theme change.
- **`livepreview.ts`**: explicitly recorded as already correct. Future refactors must not drag it into LivePreviewView.svelte "because that's where it's used."
- **DiffView submode shell**: top-level shell with view-mode-specific logic that doesn't replicate elsewhere. Generalizing would create an abstraction with one caller.
- **SettingsDialog schema-driven rewrite**: working visual component on a stable surface. 10 settings is below the rewrite threshold. F4-F8 are TUI features — no near-term growth pressure on this dialog.

---

## 実行ガイダンス（next steps）

Run these as **separate PRs** in the order below. Each is a single-purpose change so reviewing and reverting stay cheap.

### PR 1 — `chore/useFind-composable` (effort: M, risk: low)
1. Create `src/lib/views/use-find.svelte.ts` exporting `useFind(getScope, depsFn)` and `useCmFind(getView, depsFn)`. Internally call `onMount` to `bind()` + register `window` keydown + return cleanup that removes the listener and calls `destroy()`. Internally run `$effect(() => { depsFn(); find.refresh(); })`.
2. Replace the bind/keydown/destroy/refresh quartet in all 7 hosts: SourceView, LivePreviewView, PreviewView, WysiwygView, SideBySideView, FullDiffView, HighlightView.
3. Manual smoke-test: open Find in each view, type a query, ↑/↓ navigate, close with Esc. Then re-open the same file in another view and confirm no listener leak (open DevTools "Event Listeners" tab on `window`).

### PR 2 — `chore/scroll-tracker` (effort: M, risk: low)
1. Create `src/lib/views/scroll-tracker.ts` exposing `attach(scroller, { computeLine, debounceMs?: 80, onScrollExtra? })` returning `{ captureNow(), detach() }`. Owns: scrollTimer, 80ms debounce, captureNow finalizer, zero-rect guard.
2. Migrate SourceView, LivePreviewView, PreviewView, WysiwygView. Preview/Wysiwyg attach AFTER their rAF restore so the restore frame doesn't write to currentLine.
3. Pull `restoreCmToLine(view, line)` helper into `src/lib/views/cm-editor.ts` (new file). Source and LivePreview both call it.
4. Manual smoke-test: scroll in each view → switch modes → confirm scroll position restores correctly. Open a file, scroll halfway, close and re-open → confirm currentLine persisted.

### PR 3 — `chore/markdown-render-factory` (effort: M, risk: low)
1. Create `src/lib/views/markdown-render.ts` with `createPreviewMd()` and `renderWithLineMap(md, text, perTokenHook?)`.
2. PreviewView consumes the factory directly. SideBySideView passes a per-token hook that injects `mdv-changed` for hunk overlap. WysiwygView's `lineMapMd` uses the configured parser instance only.
3. Visual regression test: take screenshots of the same file in Preview/SideBySide/Wysiwyg before and after, confirm byte-identical render.

### PR 4 — `chore/mode-registry-dispatch` (effort: S, risk: low)
1. Build a single `Record<string, () => void>` dispatch table for menu events, populated from MODE_ENTRIES (mode IDs) + imported handlers (file ops).
2. `handleMenuEvent(id)` becomes `dispatch[id]?.()`.
3. Replace the 45-line keydown `$effect` mode branch with `MODE_ENTRIES.forEach(m => if (e.key === m.key && allowed(m)) setMode(m.id))`.
4. Introduce a single `setMode(target)` helper that validates `diff` requires `gitAvailable || pendingDiskCompare`. Replace the three setters at 188, 294, 586. The demote `$effect` at 536-542 can stay as a belt-and-suspenders or be deleted.

### PR 5 — `chore/quick-cleanups` (effort: S, risk: low)
Batch all small mechanical fixes into one PR with clear sub-commits:
1. Add `humanizeError(e, op)` to the 4 leaking catch sites.
2. Fix DOM finder case-sensitivity mismatch — pass `{ normalize: (s) => s.toLowerCase() }` to `SearchCursor` and lowercase the query in `find-cm.svelte.ts:42`.
3. Fix DomFinder gutter-highlight bug — add `parent.closest('.ln, .sign')` rejection in the TreeWalker filter.
4. Move `MODE_ENTRIES`, `modeLabel`, `basename` to `$lib`.
5. Delete `ModeBar.svelte` (unreferenced).
6. Delete `addedCount()` in `types.ts`.
7. Delete duplicate `.row input[type="checkbox"]` rule in SettingsDialog.
8. Fix ExternalChange race — capture `expectedPath` before await and re-check.
9. Add `reloadFromDisk(text)` and `setPath(path, gitAvailable)` methods to DocStore. Migrate the 3 direct-mutation sites.

### PR 6 — `chore/mdv-pack-author` (effort: S, risk: low)
1. Add `author_name: &str` (and optionally email) to `pack()`'s signature in `crates/mdv-core/src/pack.rs`.
2. Tauri command (`src-tauri/src/commands/mdv.rs`) reads user.name/user.email from git config, with fallback to "Unknown" — **never** the hardcoded `ebi-oishii`.
3. Wire through the JS-side `mdvPack` IPC if the signature requires it.
4. Add a small unit test that pack() output's `author.name` matches what was passed in.

### PR 7 — `chore/diff-marker-doc` (effort: S, risk: low)
Pick the doc-fix option for `DiffMarker::Unknown` in `git.rs:33-34`: update the doc comment to say "Unknown means current buffer was not provided." Leave the code branch alone. Trivial; pair with any other git.rs touch.

## What to defer

- **External-change watcher store extraction**: real refactor but ~90 lines, mode-coupling needs care, no test coverage, no near-term feature pulls on it. Revisit if/when watcher gains a third reason-to-change (e.g. project-level watching for image paste).
- **Settings store consolidation (single `values` object)**: pairs with type-safety fix. ~30 LOC win but touches an actively-edited surface and SettingsDialog rewrite is risky. Defer until the dialog crosses 15 settings or a Lua-hook config story (MEMORY note) starts landing.
- **Rust mdv-core decomposition (git split, RepoError, pack split)**: clean theoretical wins with real-but-modest risk on hot paths. No CLI/LSP consumer pulling on it yet. Defer.
- **CSS dimension consolidation**: already correctly flagged as low-priority — the dimensions genuinely differ. Wait for a theme/contrast pass to force the issue.

## After each PR

- Run `pnpm check && cargo check` plus the existing tests.
- Manual smoke-test the diff between PRs (Find in every view → scroll position restore → MD rendering byte-identical → menu+shortcut still dispatch).
- Don't squash-merge PR 1-3 into one branch. Keep them separate so a regression in (say) scroll-tracker can be reverted without losing the find-composable win.
