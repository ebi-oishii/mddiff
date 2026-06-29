<script lang="ts">
  import { onDestroy, onMount } from "svelte";
  import { doc } from "$lib/stores/doc.svelte";
  import FindBar from "$lib/components/FindBar.svelte";
  import { useFind } from "./use-find.svelte";
  import { attachScrollTracker, type ScrollTracker } from "./scroll-tracker";
  import { createPreviewMd, renderWithLineMap } from "./markdown-render";
  import { handleLinkClick } from "./link-click";

  let { text }: { text: string } = $props();

  const md = createPreviewMd();
  const html = $derived(renderWithLineMap(md, text, doc.path));

  /**
   * Click delegation for any `<a>` in the rendered article. Routes external
   * URLs to the OS opener, anchor links to in-view scroll, and local
   * markdown files to a new mddiff window. See `link-click.ts` for the full
   * dispatch table.
   */
  function onArticleClick(event: MouseEvent) {
    handleLinkClick(event, {
      getDocPath: () => doc.path,
      onScrollAnchor: (id) => {
        if (!scroller) return;
        const el = scroller.querySelector(`#${CSS.escape(id)}`);
        el?.scrollIntoView({ behavior: "smooth", block: "start" });
      },
    });
  }

  let scroller: HTMLDivElement;
  let scrollTracker: ScrollTracker | null = null;

  function topVisibleLine(): number | null {
    if (!scroller) return null;
    const rect = scroller.getBoundingClientRect();
    // Detached element returns a zero rect, in which case every child also
    // reports zero — bail to avoid clobbering currentLine with line 1.
    if (rect.width === 0 && rect.height === 0) return null;
    const top = rect.top;
    const blocks = scroller.querySelectorAll<HTMLElement>("[data-mddiff-line]");
    let last: number | null = null;
    for (const block of blocks) {
      const br = block.getBoundingClientRect();
      if (br.top >= top - 2) {
        const n = Number(block.dataset.mddiffLine);
        return Number.isFinite(n) ? n : last;
      }
      const n = Number(block.dataset.mddiffLine);
      if (Number.isFinite(n)) last = n;
    }
    return last;
  }

  function scrollToLine(line: number) {
    if (!scroller) return;
    const blocks = Array.from(
      scroller.querySelectorAll<HTMLElement>("[data-mddiff-line]"),
    );
    if (blocks.length === 0) return;
    // Find the block whose data-mddiff-line is the largest value <= `line`.
    let target: HTMLElement = blocks[0];
    for (const block of blocks) {
      const n = Number(block.dataset.mddiffLine);
      if (Number.isFinite(n) && n <= line) target = block;
      else break;
    }
    scroller.scrollTop = target.offsetTop;
  }

  const find = useFind(() => {
    void html;
    void find.query;
    void find.open;
  });

  onMount(() => {
    find.bind(scroller);
    // Wait one frame for the rendered HTML to land in the DOM.
    const line = doc.currentLine;
    requestAnimationFrame(() => {
      scrollToLine(line);
      // Attach the scroll tracker AFTER the initial restore so the very
      // first frame of restoration scroll doesn't overwrite currentLine.
      if (scroller) {
        scrollTracker = attachScrollTracker(scroller, {
          computeLine: topVisibleLine,
        });
      }
    });
  });

  onDestroy(() => {
    scrollTracker?.captureNow();
    scrollTracker?.detach();
  });
</script>

<div class="preview-scroller" bind:this={scroller}>
  <article class="preview" onclick={onArticleClick} role="presentation">
    {@html html}
  </article>
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
