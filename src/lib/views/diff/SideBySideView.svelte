<script lang="ts">
  import MarkdownIt from "markdown-it";
  import DOMPurify from "dompurify";
  import taskLists from "markdown-it-task-lists";
  import type { HunkSummary, SideBySidePayload } from "$lib/types";
  import { mapNewToOld, mapOldToNew } from "./line-map";

  let {
    payload,
    baseLabel,
  }: { payload: SideBySidePayload; baseLabel: string } = $props();

  let oldScroller: HTMLDivElement;
  let newScroller: HTMLDivElement;
  let syncedScroll = $state(true);
  // Guard so programmatic scrollTop writes don't echo back through the sibling
  // pane's scroll handler and create a feedback loop.
  let suppressSync = false;

  /** Find the source line at the top of the viewport (per `data-mdv-line`). */
  function topVisibleLine(scroller: HTMLDivElement): number | null {
    const top = scroller.getBoundingClientRect().top;
    const blocks = scroller.querySelectorAll<HTMLElement>("[data-mdv-line]");
    let last: number | null = null;
    for (const block of blocks) {
      const rect = block.getBoundingClientRect();
      if (rect.top >= top - 2) {
        const n = Number(block.dataset.mdvLine);
        return Number.isFinite(n) ? n : last;
      }
      const n = Number(block.dataset.mdvLine);
      if (Number.isFinite(n)) last = n;
    }
    return last;
  }

  /** Scroll the pane so that the block for `line` is at the top. */
  function scrollToLine(scroller: HTMLDivElement, line: number) {
    const blocks = Array.from(
      scroller.querySelectorAll<HTMLElement>("[data-mdv-line]"),
    );
    if (blocks.length === 0) return;
    let target = blocks[0];
    for (const block of blocks) {
      const n = Number(block.dataset.mdvLine);
      if (Number.isFinite(n) && n <= line) target = block;
      else break;
    }
    scroller.scrollTop = target.offsetTop;
  }

  /** Sync `dst` to `src` using line-based mapping through the diff hunks. */
  function syncLine(side: "old" | "new") {
    if (!syncedScroll || suppressSync) return;
    const src = side === "old" ? oldScroller : newScroller;
    const dst = side === "old" ? newScroller : oldScroller;
    if (!src || !dst) return;
    const srcLine = topVisibleLine(src);
    if (srcLine == null) return;
    const dstLine =
      side === "old"
        ? mapOldToNew(srcLine, payload.hunks)
        : mapNewToOld(srcLine, payload.hunks);
    suppressSync = true;
    scrollToLine(dst, dstLine);
    setTimeout(() => {
      suppressSync = false;
    }, 0);
  }

  function onOldScroll() {
    syncLine("old");
  }
  function onNewScroll() {
    syncLine("new");
  }

  function toggleSync() {
    syncedScroll = !syncedScroll;
    if (syncedScroll) {
      // Snap the OLD pane to follow NEW right now — the user's "now" is
      // wherever they last interacted, and NEW is the editing target.
      syncLine("new");
    }
  }

  const md = new MarkdownIt({
    html: true,
    linkify: true,
    breaks: false,
    typographer: true,
  });
  md.use(taskLists, { enabled: false, label: false });

  type Side = "old" | "new";

  function rangeOverlaps(
    a1: number,
    a2: number,
    b1: number,
    b2: number,
  ): boolean {
    return a1 <= b2 && b1 <= a2;
  }

  /**
   * Two-stage markdown-it pipeline:
   *   1. parse to tokens (block-level tokens carry `token.map = [start, end_exclusive]`)
   *   2. for each block_open token: tag with `data-mdv-line` (for scroll sync)
   *      and, if its range overlaps any visible hunk on this side, inject
   *      `class="mdv-changed mdv-changed-{kind}"` (for the colored overlay)
   *   3. render
   */
  function highlightedHtml(
    text: string,
    hunks: HunkSummary[],
    side: Side,
  ): string {
    const env: Record<string, unknown> = {};
    const tokens = md.parse(text, env);

    for (const token of tokens) {
      if (!token.map || !token.type.endsWith("_open")) continue;
      const tStart = token.map[0] + 1;
      const tEnd = token.map[1];

      token.attrJoin("data-mdv-line", String(tStart));

      for (const h of hunks) {
        let hStart: number;
        let hEnd: number;
        if (side === "new") {
          if (h.kind === "removed") continue;
          hStart = h.new_start;
          hEnd = h.new_end;
        } else {
          if (h.kind === "added") continue;
          hStart = h.old_start;
          hEnd = h.old_end;
        }
        if (rangeOverlaps(tStart, tEnd, hStart, hEnd)) {
          token.attrJoin("class", `mdv-changed mdv-changed-${h.kind}`);
          break;
        }
      }
    }

    return DOMPurify.sanitize(md.renderer.render(tokens, md.options, env), {
      ADD_ATTR: ["data-mdv-line"],
    });
  }

  const oldHtml = $derived(
    highlightedHtml(payload.old_text, payload.hunks, "old"),
  );
  const newHtml = $derived(
    highlightedHtml(payload.new_text, payload.hunks, "new"),
  );
</script>

<div class="sbs">
  <div class="pane">
    <div class="pane-header">
      <span class="side-label">old</span>
      <span class="base-label">{baseLabel}</span>
    </div>
    <div class="pane-scroller" bind:this={oldScroller} onscroll={onOldScroll}>
      <article class="preview">{@html oldHtml}</article>
    </div>
  </div>
  <div class="pane">
    <div class="pane-header">
      <span class="side-label">new</span>
      <span class="base-label">current buffer</span>
    </div>
    <div class="pane-scroller" bind:this={newScroller} onscroll={onNewScroll}>
      <article class="preview">{@html newHtml}</article>
    </div>
  </div>
  <button
    type="button"
    class="sync-toggle"
    class:on={syncedScroll}
    onclick={toggleSync}
    aria-pressed={syncedScroll}
    aria-label={syncedScroll ? "Disable synced scroll" : "Enable synced scroll"}
    title={syncedScroll
      ? "Synced scrolling — click to make panes independent"
      : "Independent scrolling — click to sync the panes now"}
  >
    {syncedScroll ? "🔗" : "⛓"}
  </button>
</div>

<style>
  .sbs {
    display: grid;
    grid-template-columns: 1fr 1fr;
    height: 100%;
    min-height: 0;
    position: relative; /* anchor the sync-toggle */
  }
  .sync-toggle {
    position: absolute;
    left: 50%;
    top: 50%;
    transform: translate(-50%, -50%);
    z-index: 5;
    width: 38px;
    height: 38px;
    border-radius: 999px;
    border: 1px solid var(--mdv-border);
    background: var(--mdv-surface-pop);
    color: var(--mdv-text-mute);
    cursor: pointer;
    font-size: 1.05rem;
    line-height: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    box-shadow: 0 2px 8px var(--mdv-shadow);
    transition:
      background-color 0.12s,
      color 0.12s,
      transform 0.08s;
  }
  .sync-toggle:hover {
    transform: translate(-50%, -50%) scale(1.06);
  }
  .sync-toggle.on {
    background: var(--mdv-accent-bg);
    color: var(--mdv-accent-fg);
    border-color: transparent;
  }
  .pane {
    display: flex;
    flex-direction: column;
    min-height: 0;
    min-width: 0;
    border-right: 1px solid light-dark(#ddd, #333);
  }
  .pane:last-child {
    border-right: 0;
  }
  /* Narrow window / mobile: stack panes vertically so each gets full width. */
  @media (max-width: 760px) {
    .sbs {
      grid-template-columns: 1fr;
      grid-template-rows: 1fr 1fr;
    }
    .pane {
      border-right: 0;
      border-bottom: 1px solid light-dark(#ddd, #333);
    }
    .pane:last-child {
      border-bottom: 0;
    }
  }
  .pane-header {
    flex-shrink: 0;
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.35rem 1rem;
    background: light-dark(#f4f4f4, #1e1e1e);
    border-bottom: 1px solid light-dark(#eee, #2a2a2a);
    font-size: 0.8rem;
    color: light-dark(#555, #aaa);
  }
  .side-label {
    text-transform: uppercase;
    font-weight: 600;
    letter-spacing: 0.05em;
    font-size: 0.7rem;
    padding: 0.05rem 0.4rem;
    border-radius: 3px;
    background: light-dark(#e3eaf5, #2b3a55);
    color: light-dark(#16325c, #b9d0ff);
  }
  .base-label {
    font-family: ui-monospace, "SF Mono", Menlo, monospace;
  }
  .pane-scroller {
    flex: 1;
    overflow: auto;
    min-height: 0;
  }
  .preview {
    max-width: 80ch;
    margin: 0 auto;
    padding: 1.5rem 2rem 4rem;
    line-height: 1.7;
    font-size: 15px;
  }
  .preview :global(h1),
  .preview :global(h2) {
    border-bottom: 1px solid light-dark(#eee, #333);
    padding-bottom: 0.3em;
  }
  .preview :global(h1) {
    font-size: 1.8rem;
    margin: 1.5em 0 0.5em;
  }
  .preview :global(h2) {
    font-size: 1.4rem;
    margin: 1.5em 0 0.5em;
  }
  .preview :global(h3) {
    font-size: 1.15rem;
    margin: 1.25em 0 0.5em;
  }
  .preview :global(p) {
    margin: 0.75em 0;
  }
  .preview :global(code) {
    background: light-dark(#f5f5f5, #2a2a2a);
    padding: 0.15em 0.4em;
    border-radius: 3px;
    font-size: 0.9em;
    font-family: ui-monospace, monospace;
  }
  .preview :global(pre) {
    background: light-dark(#f5f5f5, #1f1f1f);
    padding: 1em;
    border-radius: 6px;
    overflow: auto;
  }
  .preview :global(pre code) {
    background: transparent;
    padding: 0;
  }
  .preview :global(blockquote) {
    margin: 1em 0;
    padding: 0 1em;
    border-left: 4px solid light-dark(#ddd, #444);
    color: light-dark(#666, #aaa);
  }
  .preview :global(a) {
    color: light-dark(#0969da, #58a6ff);
  }
  .preview :global(ul),
  .preview :global(ol) {
    padding-left: 1.5em;
  }
  .preview :global(li.task-list-item) {
    list-style: none;
    margin-left: -1.5em;
  }
  .preview :global(li.task-list-item input.task-list-item-checkbox) {
    margin-right: 0.5em;
    cursor: default;
    vertical-align: middle;
  }
  .preview :global(img) {
    max-width: 100%;
  }

  /* Highlight overlays injected by the markdown-it pipeline. */
  .preview :global(.mdv-changed) {
    border-left: 3px solid transparent;
    padding-left: 0.6rem;
    margin-left: -0.9rem;
  }
  .preview :global(.mdv-changed-added) {
    border-left-color: #2ea043;
    background: light-dark(#e6ffec, rgba(46, 160, 67, 0.12));
  }
  .preview :global(.mdv-changed-modified) {
    border-left-color: #d29922;
    background: light-dark(#fff8c5, rgba(210, 153, 34, 0.12));
  }
  .preview :global(.mdv-changed-removed) {
    border-left-color: #cf222e;
    background: light-dark(#ffebe9, rgba(207, 34, 46, 0.12));
  }
</style>
