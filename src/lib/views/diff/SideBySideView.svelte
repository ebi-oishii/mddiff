<script lang="ts">
  import { onDestroy, onMount } from "svelte";
  import MarkdownIt from "markdown-it";
  import DOMPurify from "dompurify";
  import taskLists from "markdown-it-task-lists";
  import type { HunkSummary, SideBySidePayload } from "$lib/types";
  import { mapNewToOld, mapOldToNew } from "./line-map";
  import FindBar from "$lib/components/FindBar.svelte";
  import { FindState } from "../find.svelte";

  let {
    payload,
    baseLabel,
  }: { payload: SideBySidePayload; baseLabel: string } = $props();

  let oldScroller: HTMLDivElement;
  let newScroller: HTMLDivElement;
  let sbsWrap: HTMLDivElement;
  let syncedScroll = $state(true);

  // Find scopes both panes — the outer .sbs wrapper contains both. A match
  // in either pane is reachable; scrollIntoView walks the parent chain and
  // scrolls the right .pane-scroller into position.
  const find = new FindState();

  onMount(() => {
    find.bind(sbsWrap);
    window.addEventListener("keydown", find.onKeydown);
  });

  onDestroy(() => {
    window.removeEventListener("keydown", find.onKeydown);
    find.destroy();
  });

  $effect(() => {
    void payload;
    void find.query;
    void find.open;
    find.refresh();
  });

  /**
   * Track the last scrollTop we programmatically wrote to each pane (and
   * when). Incoming `scroll` events whose scrollTop matches a recent write
   * within a small px / ms window are treated as echoes of our own command
   * and consumed without propagating, while genuine user scrolls go
   * through. This is more robust than counting Set entries — the browser
   * sometimes coalesces multiple scrollTop writes into a single event, and
   * the order of `scroll` event firing vs. setTimeout / microtask draining
   * isn't fixed across engines.
   */
  const lastCommand = {
    old: { top: -1, at: 0 },
    new: { top: -1, at: 0 },
  };
  const COMMAND_TOLERANCE_PX = 2;
  const COMMAND_TIMEOUT_MS = 250;

  // rAF throttle so a smooth-scroll burst doesn't fire N driveSyncs per frame.
  let pendingSrc: "old" | "new" | null = null;
  let rafToken: number | null = null;

  /**
   * Sub-block-precision: returns the source line of the block that contains
   * the viewport top, plus how far the viewport top is into that block
   * (0 = block top is at viewport top, 1 = block bottom is at viewport top).
   * Block-only mapping (returning just `line`) caused round-trip drift
   * because each sync snapped the dst to the start of a block, losing the
   * "I'm halfway through this paragraph" information.
   */
  function topVisiblePos(
    scroller: HTMLDivElement,
  ): { line: number; fraction: number } | null {
    const topY = scroller.getBoundingClientRect().top;
    const blocks = scroller.querySelectorAll<HTMLElement>("[data-mdv-line]");
    let active: HTMLElement | null = null;
    for (const block of blocks) {
      const rect = block.getBoundingClientRect();
      // Pick the first block whose bottom is still below the viewport top —
      // that's the block currently containing (or at) the top edge.
      if (rect.bottom > topY + 1) {
        active = block;
        break;
      }
      active = block;
    }
    if (!active) return null;
    const rect = active.getBoundingClientRect();
    const line = Number(active.dataset.mdvLine);
    if (!Number.isFinite(line)) return null;
    const fraction =
      rect.height > 0
        ? Math.max(0, Math.min(1, (topY - rect.top) / rect.height))
        : 0;
    return { line, fraction };
  }

  /**
   * Scroll so that `line`'s block is at the viewport top, then nudge by
   * `fraction * blockHeight` so the same "% into block" position is shown.
   * Uses getBoundingClientRect rather than offsetTop because offsetTop is
   * relative to offsetParent (which may not be the scroller).
   */
  function scrollToLinePos(
    scroller: HTMLDivElement,
    line: number,
    fraction: number,
  ) {
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
    const scrollerRect = scroller.getBoundingClientRect();
    const targetRect = target.getBoundingClientRect();
    const offsetInsideScroller =
      scroller.scrollTop + (targetRect.top - scrollerRect.top);
    scroller.scrollTop = offsetInsideScroller + fraction * targetRect.height;
  }

  /** Drive `dst` from `src` via the line mapping and record what we wrote. */
  function driveSync(srcSide: "old" | "new") {
    const src = srcSide === "old" ? oldScroller : newScroller;
    const dst = srcSide === "old" ? newScroller : oldScroller;
    const dstSide = srcSide === "old" ? "new" : "old";
    if (!src || !dst) return;

    const before = dst.scrollTop;
    const srcMax = Math.max(0, src.scrollHeight - src.clientHeight);
    const dstMax = Math.max(0, dst.scrollHeight - dst.clientHeight);

    // Edge anchoring: when the source is pinned to the very top (or bottom),
    // pin the destination too. Without this, line-based mapping snaps `dst`
    // to the offsetTop of its first block, which is `padding-top` (not 0) —
    // so the user can scroll src to literal 0 but the other pane stays
    // ~24px down and "won't fully scroll to the top".
    if (src.scrollTop <= 0) {
      dst.scrollTop = 0;
    } else if (src.scrollTop >= srcMax - 1) {
      dst.scrollTop = dstMax;
    } else {
      const srcPos = topVisiblePos(src);
      if (!srcPos) return;
      const dstLine =
        srcSide === "old"
          ? mapOldToNew(srcPos.line, payload.hunks)
          : mapNewToOld(srcPos.line, payload.hunks);
      scrollToLinePos(dst, dstLine, srcPos.fraction);
    }

    if (dst.scrollTop !== before) {
      lastCommand[dstSide] = { top: dst.scrollTop, at: performance.now() };
    }
  }

  function isOurEcho(side: "old" | "new"): boolean {
    const sc = side === "old" ? oldScroller : newScroller;
    if (!sc) return false;
    const cmd = lastCommand[side];
    if (cmd.top < 0) return false;
    const fresh = performance.now() - cmd.at <= COMMAND_TIMEOUT_MS;
    const matches = Math.abs(sc.scrollTop - cmd.top) <= COMMAND_TOLERANCE_PX;
    if (fresh && matches) {
      // Consume — invalidate so a future user scroll at the same position
      // isn't wrongly absorbed.
      lastCommand[side] = { top: -1, at: 0 };
      return true;
    }
    return false;
  }

  function onScroll(side: "old" | "new") {
    if (!syncedScroll) return;
    if (isOurEcho(side)) return;
    // Throttle: at most one driveSync per animation frame; the latest side to
    // scroll wins (matches user intent — they're focused on that pane).
    pendingSrc = side;
    if (rafToken != null) return;
    rafToken = requestAnimationFrame(() => {
      rafToken = null;
      const src = pendingSrc;
      pendingSrc = null;
      if (src) driveSync(src);
    });
  }

  function onOldScroll() {
    onScroll("old");
  }
  function onNewScroll() {
    onScroll("new");
  }

  function toggleSync() {
    syncedScroll = !syncedScroll;
    if (syncedScroll) {
      // Snap OLD to follow NEW right now — NEW is the editing target, so
      // align the comparison view to where the user currently is.
      driveSync("new");
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

<div class="sbs" bind:this={sbsWrap}>
  <!-- LEFT pane: current buffer. The "primary" side that the user is
       editing reads left-to-right naturally; the comparison base sits to
       the right as the reference. -->
  <div class="pane">
    <div class="pane-header">
      <span class="side-label new">new</span>
      <span class="base-label">current buffer</span>
    </div>
    <div class="pane-scroller" bind:this={newScroller} onscroll={onNewScroll}>
      <article class="preview">{@html newHtml}</article>
    </div>
  </div>
  <div class="pane">
    <div class="pane-header">
      <span class="side-label old">old</span>
      <span class="base-label">{baseLabel}</span>
    </div>
    <div class="pane-scroller" bind:this={oldScroller} onscroll={onOldScroll}>
      <article class="preview">{@html oldHtml}</article>
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
{#if find.open}
  <FindBar
    bind:query={find.query}
    matchCount={find.matchCount}
    currentIndex={find.currentIndex}
    focusVersion={find.focusVersion}
    onnext={find.next}
    onprev={find.prev}
    onclose={find.close}
  />
{/if}

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
