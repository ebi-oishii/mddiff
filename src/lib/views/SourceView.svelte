<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { EditorState } from "@codemirror/state";
  import { EditorView, keymap, lineNumbers, highlightActiveLine } from "@codemirror/view";
  import { defaultKeymap, history, historyKeymap } from "@codemirror/commands";
  import { markdown } from "@codemirror/lang-markdown";
  import { syntaxHighlighting, defaultHighlightStyle } from "@codemirror/language";
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
      // Detached element returns a zero rect — bail before posAtCoords maps
      // (0, 0) to position 0 and clobbers currentLine with 1.
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
        lineNumbers(),
        highlightActiveLine(),
        syntaxHighlighting(defaultHighlightStyle, { fallback: true }),
        keymap.of([...defaultKeymap, ...historyKeymap]),
        markdown(),
        EditorView.lineWrapping,
        mdvCmTheme,
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

    // Track scroll continuously so doc.currentLine stays fresh regardless of
    // unmount timing. Svelte 5's onDestroy can fire after the DOM has been
    // detached, at which point getBoundingClientRect returns a zero rect and
    // posAtCoords resolves to position 0 (line 1), silently clobbering the
    // saved line. Continuous capture sidesteps that race.
    view.scrollDOM.addEventListener("scroll", onScroll, { passive: true });

    // Restore scroll position from DocStore so mode switches stay in place.
    // Defer one frame so CodeMirror has measured the layout.
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
    // Last-chance capture in case a scroll event happened in the final ~80ms
    // and the debounce hasn't fired yet. Best-effort; guarded inside.
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

<div bind:this={container} class="source"></div>

<style>
  .source {
    height: 100%;
    overflow: hidden;
  }
  /* CodeMirror styling lives in $lib/views/cm-theme.ts (delivered via
     EditorView.theme so it beats CM defaults on specificity). Only
     editor-instance-specific bits remain here. */
  :global(.cm-editor) {
    font-family: ui-monospace, "SF Mono", Menlo, Consolas, monospace;
    font-size: var(--mdv-editor-font-size, 14px);
  }
  /* In fullscreen the floating "(filename) MODE" overlay (rendered by
     +page.svelte at top-left) covers the first line of source because
     CodeMirror's default content padding is only a few pixels. Live /
     Preview / WYSIWYG have intrinsic 2rem top padding so they're fine. */
  :global(:root[data-fullscreen] .source .cm-scroller) {
    padding-top: 2.5rem;
  }
</style>
