<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import MarkdownIt from "markdown-it";
  import { Editor, defaultValueCtx, rootCtx } from "@milkdown/kit/core";
  import { commonmark } from "@milkdown/kit/preset/commonmark";
  import { gfm } from "@milkdown/kit/preset/gfm";
  import { listener, listenerCtx } from "@milkdown/kit/plugin/listener";
  import { getMarkdown, replaceAll } from "@milkdown/kit/utils";
  import { doc } from "$lib/stores/doc.svelte";

  let {
    text,
    onchange,
    onnormalize,
  }: {
    text: string;
    onchange: (t: string) => void;
    onnormalize?: (orig: string, normalized: string) => void;
  } = $props();

  let container: HTMLDivElement;
  let editor: Editor | null = null;
  let lastEmitted = "";
  let ready = $state(false);
  let scrollTimer: ReturnType<typeof setTimeout> | null = null;

  // Milkdown doesn't expose source-line positions on its rendered nodes, so we
  // reconstruct a top-level-block → source-line mapping by parsing the same
  // markdown with markdown-it. ProseMirror renders top-level doc nodes 1:1 as
  // `.ProseMirror` children, so the array index lines up with the child index.
  const lineMapMd = new MarkdownIt({ html: true, linkify: true, breaks: false });
  let cachedSrc = "";
  let cachedLines: number[] = [];
  function topLevelBlockLines(src: string): number[] {
    if (src === cachedSrc) return cachedLines;
    const tokens = lineMapMd.parse(src, {});
    const lines: number[] = [];
    for (const tok of tokens) {
      if (tok.level !== 0) continue;
      if (!tok.map) continue;
      if (tok.type === "inline") continue;
      if (tok.type.endsWith("_close")) continue;
      lines.push(tok.map[0] + 1);
    }
    cachedSrc = src;
    cachedLines = lines;
    return lines;
  }

  function proseMirrorRoot(): HTMLElement | null {
    return container?.querySelector(".ProseMirror") as HTMLElement | null;
  }

  function scrollToLine(line: number) {
    const root = proseMirrorRoot();
    if (!root || !container) return;
    const lines = topLevelBlockLines(lastEmitted || text);
    if (lines.length === 0) return;
    // largest i with lines[i] <= line; if line < lines[0], stay at the top.
    let target = 0;
    for (let i = 0; i < lines.length; i++) {
      if (lines[i] <= line) target = i;
      else break;
    }
    const children = root.children;
    if (target >= children.length) target = children.length - 1;
    const el = children[target] as HTMLElement | undefined;
    if (el) container.scrollTop = el.offsetTop;
  }

  function topVisibleLine(): number | null {
    const root = proseMirrorRoot();
    if (!root || !container) return null;
    const rect = container.getBoundingClientRect();
    // Detached element returns a zero rect — bail to avoid clobbering
    // currentLine with line 1.
    if (rect.width === 0 && rect.height === 0) return null;
    const top = rect.top;
    const children = Array.from(root.children) as HTMLElement[];
    if (children.length === 0) return null;
    let blockIndex = 0;
    for (let i = 0; i < children.length; i++) {
      const cr = children[i].getBoundingClientRect();
      if (cr.top >= top - 2) {
        blockIndex = i;
        break;
      }
      blockIndex = i;
    }
    const lines = topLevelBlockLines(lastEmitted || text);
    return lines[blockIndex] ?? null;
  }

  function captureTopLine() {
    const line = topVisibleLine();
    if (line != null) doc.currentLine = line;
  }

  function onScroll() {
    if (scrollTimer) clearTimeout(scrollTimer);
    scrollTimer = setTimeout(captureTopLine, 80);
  }

  onMount(async () => {
    const initial = text;
    editor = await Editor.make()
      .config((ctx) => {
        ctx.set(rootCtx, container);
        ctx.set(defaultValueCtx, initial);
        ctx.get(listenerCtx).markdownUpdated((_ctx, markdown) => {
          if (markdown !== lastEmitted) {
            lastEmitted = markdown;
            onchange(markdown);
          }
        });
      })
      .use(commonmark)
      .use(gfm)
      .use(listener)
      .create();

    // After load, query Milkdown's own serialization of the doc to detect
    // round-trip normalization (e.g. `*foo*` <-> `_foo_`, link reference
    // expansion, trailing newline adjustment). If different from what we
    // loaded, surface it via `onnormalize` so the parent can both warn the
    // user AND adopt the normalized form as the new baseline (so the dirty
    // indicator doesn't appear just from opening WYSIWYG).
    //
    // We intentionally do NOT call `onchange` here — that's reserved for
    // genuine user edits via the listener.
    try {
      const serialized = editor.action(getMarkdown());
      lastEmitted = serialized;
      if (serialized.trim() !== initial.trim()) {
        onnormalize?.(initial, serialized);
      }
    } catch {
      // getMarkdown not available in this build; skip detection silently.
    }

    ready = true;

    // Restore scroll position last so Milkdown's render has been committed
    // and lastEmitted (post-normalization) is set for an accurate line map.
    const restore = doc.currentLine;
    requestAnimationFrame(() => {
      try {
        scrollToLine(restore);
      } catch {}
      // Attach scroll listener after the restore frame so the programmatic
      // scroll doesn't race the user's first manual scroll for the debounce.
      container?.addEventListener("scroll", onScroll, { passive: true });
    });
  });

  onDestroy(() => {
    if (scrollTimer) clearTimeout(scrollTimer);
    try {
      captureTopLine();
    } catch {
      // DOM might already be torn down; skip silently.
    }
    container?.removeEventListener("scroll", onScroll);
    editor?.destroy();
    editor = null;
  });

  $effect(() => {
    if (editor && ready && text !== lastEmitted) {
      lastEmitted = text;
      editor.action(replaceAll(text));
    }
  });
</script>

<div bind:this={container} class="wys"></div>

<style>
  .wys {
    height: 100%;
    overflow: auto;
  }
  :global(.wys .milkdown) {
    max-width: 92ch;
    margin: 0 auto;
    padding: 2rem 3rem 4rem;
    font-family:
      system-ui,
      -apple-system,
      "Segoe UI",
      Roboto,
      "Hiragino Sans",
      "Yu Gothic",
      sans-serif;
    font-size: 16px;
    line-height: 1.7;
    color: light-dark(#222, #ddd);
  }
  /* In fullscreen the title overlay (set by +page.svelte) sits at the top
     of the canvas and clips the first paragraph if the content's own 2rem
     padding doesn't quite clear it. Match SourceView by widening to 2.5rem
     just for fullscreen. */
  :global(:root[data-fullscreen] .wys .milkdown) {
    padding-top: 2.5rem;
  }
  :global(.wys .ProseMirror) {
    outline: none;
    min-height: 100%;
  }
  :global(.wys h1) {
    font-size: 2rem;
    font-weight: 700;
    margin: 1.5em 0 0.5em;
    border-bottom: 1px solid light-dark(#eee, #333);
    padding-bottom: 0.3em;
  }
  :global(.wys h2) {
    font-size: 1.5rem;
    font-weight: 700;
    margin: 1.5em 0 0.5em;
    border-bottom: 1px solid light-dark(#eee, #333);
    padding-bottom: 0.3em;
  }
  :global(.wys h3) {
    font-size: 1.25rem;
    font-weight: 600;
    margin: 1.25em 0 0.5em;
  }
  :global(.wys p) {
    margin: 0.75em 0;
  }
  :global(.wys code) {
    background: light-dark(#f3f3f3, #2a2a2a);
    padding: 0.15em 0.4em;
    border-radius: 3px;
    font-size: 0.9em;
    font-family: ui-monospace, "SF Mono", Menlo, monospace;
  }
  :global(.wys pre) {
    background: light-dark(#f5f5f5, #1f1f1f);
    padding: 1em;
    border-radius: 6px;
    overflow: auto;
    margin: 1em 0;
  }
  :global(.wys pre code) {
    background: transparent;
    padding: 0;
  }
  :global(.wys blockquote) {
    margin: 1em 0;
    padding: 0 1em;
    border-left: 4px solid light-dark(#ddd, #444);
    color: light-dark(#666, #aaa);
  }
  :global(.wys a) {
    color: light-dark(#0969da, #58a6ff);
  }
  :global(.wys ul),
  :global(.wys ol) {
    padding-left: 1.5em;
    margin: 0.75em 0;
  }
  :global(.wys li) {
    margin: 0.2em 0;
  }
  /* GFM task list items: Milkdown emits attributes only, draw the box ourselves. */
  :global(.wys li[data-item-type="task"]) {
    list-style: none;
    margin-left: -0.4em;
  }
  :global(.wys li[data-item-type="task"] > p) {
    display: inline;
    margin: 0;
  }
  :global(.wys li[data-item-type="task"]::before) {
    content: "☐ ";
    display: inline-block;
    width: 1.2em;
    color: light-dark(#888, #888);
    font-size: 1.05em;
    vertical-align: -1px;
  }
  :global(.wys li[data-item-type="task"][data-checked="true"]::before) {
    content: "☑ ";
    color: light-dark(#2ea043, #3fb950);
  }
  :global(.wys table) {
    border-collapse: collapse;
    margin: 1em 0;
  }
  :global(.wys th),
  :global(.wys td) {
    border: 1px solid light-dark(#ddd, #444);
    padding: 0.5em 0.8em;
  }
  :global(.wys img) {
    max-width: 100%;
  }
  :global(.wys hr) {
    border: 0;
    border-top: 1px solid light-dark(#ddd, #444);
    margin: 1.5em 0;
  }
</style>
