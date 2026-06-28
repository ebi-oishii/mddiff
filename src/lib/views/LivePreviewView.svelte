<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { EditorState } from "@codemirror/state";
  import { EditorView, keymap, highlightActiveLine } from "@codemirror/view";
  import { defaultKeymap, history, historyKeymap } from "@codemirror/commands";
  import { markdown } from "@codemirror/lang-markdown";
  import { livePreviewExtension } from "./livepreview";
  import { doc } from "$lib/stores/doc.svelte";
  import { mdvCmTheme } from "./cm-theme";

  let {
    text,
    onchange,
  }: { text: string; onchange: (t: string) => void } = $props();

  let container: HTMLDivElement;
  let view: EditorView | null = null;
  let lastEmitted = "";
  let scrollTimer: ReturnType<typeof setTimeout> | null = null;

  function captureTopLine() {
    if (!view) return;
    try {
      const rect = view.scrollDOM.getBoundingClientRect();
      if (rect.width === 0 && rect.height === 0) return;
      const pos = view.posAtCoords({ x: rect.left + 8, y: rect.top + 4 });
      if (pos != null) doc.currentLine = view.state.doc.lineAt(pos).number;
    } catch {}
  }

  function onScroll() {
    if (scrollTimer) clearTimeout(scrollTimer);
    scrollTimer = setTimeout(captureTopLine, 80);
  }

  onMount(() => {
    const state = EditorState.create({
      doc: text,
      extensions: [
        history(),
        highlightActiveLine(),
        keymap.of([...defaultKeymap, ...historyKeymap]),
        markdown(),
        EditorView.lineWrapping,
        mdvCmTheme,
        livePreviewExtension,
        EditorView.updateListener.of((u) => {
          if (u.docChanged) {
            const next = u.state.doc.toString();
            lastEmitted = next;
            onchange(next);
          }
        }),
      ],
    });
    view = new EditorView({ state, parent: container });
    lastEmitted = text;

    view.scrollDOM.addEventListener("scroll", onScroll, { passive: true });

    const restore = doc.currentLine;
    requestAnimationFrame(() => {
      if (!view) return;
      const total = view.state.doc.lines;
      const safe = Math.max(1, Math.min(total, restore));
      const pos = view.state.doc.line(safe).from;
      view.dispatch({ effects: EditorView.scrollIntoView(pos, { y: "start" }) });
    });
  });

  onDestroy(() => {
    if (scrollTimer) clearTimeout(scrollTimer);
    captureTopLine();
    view?.scrollDOM.removeEventListener("scroll", onScroll);
    view?.destroy();
  });

  $effect(() => {
    if (view && text !== lastEmitted) {
      lastEmitted = text;
      view.dispatch({
        changes: { from: 0, to: view.state.doc.length, insert: text },
      });
    }
  });
</script>

<div bind:this={container} class="live"></div>

<style>
  .live {
    height: 100%;
    overflow: hidden;
  }
  :global(.live .cm-editor) {
    height: 100%;
    font-family:
      system-ui,
      -apple-system,
      "Segoe UI",
      Roboto,
      "Hiragino Sans",
      "Yu Gothic",
      sans-serif;
    /* Hard-coded 16px to match PreviewView. `ch` units depend on font-size,
       so reading off a CSS variable (= user's editor setting) here would make
       the 92ch max-width resolve to different actual pixels than Preview's
       92ch. Source mode keeps the configurable font-size — it's the
       monospace editor where users tweak size; Live Preview / Preview are
       reading views. */
    font-size: 16px;
    line-height: 1.7;
  }
  :global(.live .cm-scroller) {
    overflow: auto;
    /* Layout matched to PreviewView: padding on the content block itself so
       the visible "card" of text has the same outer rect (92ch + 6rem) as
       Preview. */
    padding: 0;
  }
  :global(.live .cm-content) {
    max-width: 92ch;
    margin: 0 auto;
    padding: 2rem 3rem 4rem;
  }
  /* In fullscreen the title overlay sits over the top of the canvas;
     widen the top padding to clear it. */
  :global(:root[data-fullscreen] .live .cm-content) {
    padding-top: 2.5rem;
  }
  :global(.live .cm-line) {
    padding: 0;
  }

  /* Heading sizes (mark decorations). */
  :global(.mdv-lp-h1) {
    font-size: 1.85em;
    font-weight: 700;
    line-height: 1.25;
  }
  :global(.mdv-lp-h2) {
    font-size: 1.5em;
    font-weight: 700;
    line-height: 1.25;
  }
  :global(.mdv-lp-h3) {
    font-size: 1.3em;
    font-weight: 600;
    line-height: 1.3;
  }
  :global(.mdv-lp-h4) {
    font-size: 1.15em;
    font-weight: 600;
  }
  :global(.mdv-lp-h5) {
    font-size: 1.05em;
    font-weight: 600;
  }
  :global(.mdv-lp-h6) {
    font-size: 1em;
    font-weight: 600;
    color: light-dark(#666, #aaa);
  }

  /* Inline styles. */
  :global(.mdv-lp-bold) {
    font-weight: 700;
  }
  :global(.mdv-lp-italic) {
    font-style: italic;
  }
  :global(.mdv-lp-code) {
    background: light-dark(#f3f3f3, #2a2a2a);
    padding: 0.1em 0.35em;
    border-radius: 3px;
    font-family: ui-monospace, "SF Mono", Menlo, monospace;
    font-size: 0.92em;
  }
  :global(.mdv-lp-link) {
    color: light-dark(#0969da, #58a6ff);
    text-decoration: underline;
    text-underline-offset: 0.15em;
  }
</style>
