<script lang="ts">
  import MarkdownIt from "markdown-it";
  import DOMPurify from "dompurify";
  import taskLists from "markdown-it-task-lists";
  import type { HunkSummary, SideBySidePayload } from "$lib/types";

  let {
    payload,
    baseLabel,
  }: { payload: SideBySidePayload; baseLabel: string } = $props();

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
   *   1. parse text into tokens (block-level tokens carry `token.map = [start, end_exclusive]`)
   *   2. for each block_open token whose source range overlaps any hunk on
   *      this side, inject `class="mdv-changed mdv-changed-{kind}"`
   *   3. render
   *
   * On the "new" side we ignore pure Removed hunks (those lines aren't in
   * the new buffer); on the "old" side we ignore pure Added hunks.
   * For Modified hunks, the respective range applies on each side.
   */
  function highlightedHtml(
    text: string,
    hunks: HunkSummary[],
    side: Side,
  ): string {
    const tokens = md.parse(text, {});

    for (const token of tokens) {
      if (!token.map || !token.type.endsWith("_open")) continue;
      const tStart = token.map[0] + 1;
      const tEnd = token.map[1]; // markdown-it's end is exclusive 0-based ≡ inclusive 1-based last line

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

    return DOMPurify.sanitize(md.renderer.render(tokens, md.options, {}));
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
    <div class="pane-scroller">
      <article class="preview">{@html oldHtml}</article>
    </div>
  </div>
  <div class="pane">
    <div class="pane-header">
      <span class="side-label">new</span>
      <span class="base-label">current buffer</span>
    </div>
    <div class="pane-scroller">
      <article class="preview">{@html newHtml}</article>
    </div>
  </div>
</div>

<style>
  .sbs {
    display: grid;
    grid-template-columns: 1fr 1fr;
    height: 100%;
    min-height: 0;
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
