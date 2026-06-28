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
  /* CodeMirror styling lives in $lib/views/cm-theme.ts (delivered via
     EditorView.theme so it beats CM defaults on specificity). Only
     editor-instance-specific bits remain here. */
  :global(.cm-editor) {
    font-family: ui-monospace, "SF Mono", Menlo, Consolas, monospace;
    font-size: var(--mdv-editor-font-size, 14px);
  }
  /* Reserve a right strip so long lines don't slide under the floating ☰
     button (top-right, ~46px wide). Padding goes on cm-scroller (not
     cm-content): cm-scroller is the actual flex container that holds the
     gutters + content, so shrinking its content-box also shrinks the
     wrap width that the browser uses for cm-content. Padding on
     cm-content alone didn't take effect — CM's flex layout was treating
     cm-content as flex-grow with content-box padding that didn't bind
     the line wrap.
     Skipped in fullscreen: there's already a 2.5rem top padding pushing
     content below the title overlay, and the ☰ menu sits next to that
     overlay in the OS-title-bar-free area, not on top of text. */
  :global(:root:not([data-fullscreen]) .source .cm-scroller) {
    padding-right: 3rem;
    box-sizing: border-box;
  }
  /* The active-line decoration is a `<div class="cm-activeLine">` whose
     box is bounded by cm-content's content-area, so by default it stops
     3rem short of the editor's right edge, leaving a dead strip where
     the highlight is missing. A negative right margin pushes its box
     into the padding area on the right (still inside cm-scroller), so
     the background color reaches the visual edge while the line's text
     and caret stay in the original content rectangle. */
  :global(:root:not([data-fullscreen]) .source .cm-activeLine) {
    margin-right: -3rem;
  }
</style>
