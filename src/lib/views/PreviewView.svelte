<script lang="ts">
  import { onDestroy, onMount } from "svelte";
  import MarkdownIt from "markdown-it";
  import DOMPurify from "dompurify";
  import taskLists from "markdown-it-task-lists";
  import { doc } from "$lib/stores/doc.svelte";

  let { text }: { text: string } = $props();

  const md = new MarkdownIt({
    html: true,
    linkify: true,
    breaks: false,
    typographer: true,
  });
  md.use(taskLists, { enabled: false, label: false });

  /**
   * 2-stage pipeline: parse → annotate each block_open token with its source
   * line range as `data-mdv-line` → render. The attributes let us map between
   * scroll positions in the rendered DOM and source line numbers, which is
   * the basis for cross-mode scroll sync.
   */
  const html = $derived.by(() => {
    // parse and render MUST share the same `env` so plugins (notably
    // markdown-it-task-lists) can hand state between phases. Passing a fresh
    // {} to render dropped the task-list metadata and broke `[x]`/`[ ]`
    // checkbox rendering.
    const env: Record<string, unknown> = {};
    const tokens = md.parse(text, env);
    for (const token of tokens) {
      if (token.map && token.type.endsWith("_open")) {
        token.attrJoin("data-mdv-line", String(token.map[0] + 1));
      }
    }
    return DOMPurify.sanitize(md.renderer.render(tokens, md.options, env), {
      ADD_ATTR: ["data-mdv-line"],
    });
  });

  let scroller: HTMLDivElement;
  let scrollTimer: ReturnType<typeof setTimeout> | null = null;

  function topVisibleLine(): number | null {
    if (!scroller) return null;
    const rect = scroller.getBoundingClientRect();
    // Detached element returns a zero rect, in which case every child also
    // reports zero — bail to avoid clobbering currentLine with line 1.
    if (rect.width === 0 && rect.height === 0) return null;
    const top = rect.top;
    const blocks = scroller.querySelectorAll<HTMLElement>("[data-mdv-line]");
    let last: number | null = null;
    for (const block of blocks) {
      const br = block.getBoundingClientRect();
      if (br.top >= top - 2) {
        const n = Number(block.dataset.mdvLine);
        return Number.isFinite(n) ? n : last;
      }
      const n = Number(block.dataset.mdvLine);
      if (Number.isFinite(n)) last = n;
    }
    return last;
  }

  function captureTopLine() {
    const line = topVisibleLine();
    if (line != null) doc.currentLine = line;
  }

  function onScroll() {
    if (scrollTimer) clearTimeout(scrollTimer);
    scrollTimer = setTimeout(captureTopLine, 80);
  }

  function scrollToLine(line: number) {
    if (!scroller) return;
    const blocks = Array.from(
      scroller.querySelectorAll<HTMLElement>("[data-mdv-line]"),
    );
    if (blocks.length === 0) return;
    // Find the block whose data-mdv-line is the largest value <= `line`.
    let target: HTMLElement = blocks[0];
    for (const block of blocks) {
      const n = Number(block.dataset.mdvLine);
      if (Number.isFinite(n) && n <= line) target = block;
      else break;
    }
    scroller.scrollTop = target.offsetTop;
  }

  onMount(() => {
    // Wait one frame for the rendered HTML to land in the DOM.
    const line = doc.currentLine;
    requestAnimationFrame(() => {
      scrollToLine(line);
      // Attach the scroll listener AFTER the initial restore so the very
      // first frame of restoration scroll doesn't overwrite currentLine.
      scroller?.addEventListener("scroll", onScroll, { passive: true });
    });
  });

  onDestroy(() => {
    if (scrollTimer) clearTimeout(scrollTimer);
    captureTopLine();
    scroller?.removeEventListener("scroll", onScroll);
  });
</script>

<div class="preview-scroller" bind:this={scroller}>
  <article class="preview">
    {@html html}
  </article>
</div>

<style>
  .preview-scroller {
    height: 100%;
    overflow: auto;
  }
  .preview {
    max-width: 92ch;
    margin: 0 auto;
    padding: 2rem 3rem 4rem;
    line-height: 1.7;
    font-size: 16px;
  }
  /* Match SourceView / LivePreview / WYSIWYG: in fullscreen the title
     overlay covers the top, so widen the content's top padding. */
  :global(:root[data-fullscreen]) .preview {
    padding-top: 2.5rem;
  }
  .preview :global(h1) {
    font-size: 2rem;
    margin: 1.5em 0 0.5em;
    border-bottom: 1px solid light-dark(#eee, #333);
    padding-bottom: 0.3em;
  }
  .preview :global(h2) {
    font-size: 1.5rem;
    margin: 1.5em 0 0.5em;
    border-bottom: 1px solid light-dark(#eee, #333);
    padding-bottom: 0.3em;
  }
  .preview :global(h3) {
    font-size: 1.25rem;
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
  .preview :global(table) {
    border-collapse: collapse;
    margin: 1em 0;
  }
  .preview :global(th),
  .preview :global(td) {
    border: 1px solid light-dark(#ddd, #444);
    padding: 0.5em 0.8em;
  }
  .preview :global(img) {
    max-width: 100%;
  }
</style>
