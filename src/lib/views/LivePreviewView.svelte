<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { EditorState } from "@codemirror/state";
  import { EditorView, keymap, highlightActiveLine } from "@codemirror/view";
  import { defaultKeymap, history, historyKeymap } from "@codemirror/commands";
  import { markdown } from "@codemirror/lang-markdown";
  import { livePreviewExtension } from "./livepreview";
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
        highlightActiveLine(),
        keymap.of([...defaultKeymap, ...historyKeymap]),
        markdown(),
        EditorView.lineWrapping,
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
    if (view) {
      try {
        const rect = view.scrollDOM.getBoundingClientRect();
        const pos = view.posAtCoords({ x: rect.left + 8, y: rect.top + 4 });
        if (pos != null) {
          doc.currentLine = view.state.doc.lineAt(pos).number;
        }
      } catch {
        // best-effort
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
    /* Live Preview scales the editor font with the user setting but bumps it
       a notch since it's prose-styled rather than monospace. */
    font-size: calc(var(--mdv-editor-font-size, 14px) + 1.5px);
    line-height: 1.7;
  }
  :global(.live .cm-scroller) {
    overflow: auto;
    padding: 1rem 2rem 4rem;
  }
  :global(.live .cm-content) {
    max-width: 80ch;
    margin: 0 auto;
  }
  :global(.live .cm-line) {
    padding: 0 4px;
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
