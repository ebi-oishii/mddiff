<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { EditorState } from "@codemirror/state";
  import { EditorView, keymap, lineNumbers, highlightActiveLine } from "@codemirror/view";
  import { defaultKeymap, history, historyKeymap } from "@codemirror/commands";
  import { markdown } from "@codemirror/lang-markdown";
  import { syntaxHighlighting, defaultHighlightStyle } from "@codemirror/language";
  import { doc } from "$lib/stores/doc.svelte";

  let {
    text,
    onchange,
  }: { text: string; onchange: (t: string) => void } = $props();

  let container: HTMLDivElement;
  let view: EditorView | null = null;
  let lastEmitted = "";

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
    // Save topmost visible source line before tearing down so the next mode
    // can scroll there. posAtCoords is more reliable than lineBlockAtHeight
    // when the editor has padding/margins.
    if (view) {
      try {
        const rect = view.scrollDOM.getBoundingClientRect();
        const pos = view.posAtCoords({ x: rect.left + 8, y: rect.top + 4 });
        if (pos != null) {
          doc.currentLine = view.state.doc.lineAt(pos).number;
        }
      } catch {
        // best-effort; ignore if the layout isn't available
      }
    }
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
  :global(.cm-editor) {
    height: 100%;
    font-family: ui-monospace, "SF Mono", Menlo, Consolas, monospace;
    font-size: var(--mdv-editor-font-size, 14px);
    background: light-dark(#fff, #1a1a1a);
    color: light-dark(#222, #ddd);
  }
  :global(.cm-scroller) {
    overflow: auto;
  }
  /* CodeMirror 6 doesn't honor color-scheme on its own; teach the gutter,
     selection and cursor to follow `data-theme` via `light-dark()`. Affects
     every CodeMirror instance in the app (Source, Live Preview). */
  :global(.cm-gutters) {
    background: light-dark(#fafafa, #1a1a1a);
    border-right: 1px solid light-dark(#eee, #2a2a2a);
    color: light-dark(#888, #666);
  }
  :global(.cm-activeLine) {
    background: light-dark(rgba(0, 0, 0, 0.035), rgba(255, 255, 255, 0.04));
  }
  :global(.cm-activeLineGutter) {
    background: light-dark(rgba(0, 0, 0, 0.06), rgba(255, 255, 255, 0.06));
    color: light-dark(#444, #bbb);
  }
  :global(.cm-cursor) {
    border-left-color: light-dark(#000, #ddd);
  }
  :global(.cm-selectionBackground),
  :global(.cm-content ::selection) {
    background: light-dark(#bcd8fa, #2b4a73) !important;
  }
</style>
