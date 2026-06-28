<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { EditorState } from "@codemirror/state";
  import { EditorView, keymap, lineNumbers, highlightActiveLine } from "@codemirror/view";
  import { defaultKeymap, history, historyKeymap } from "@codemirror/commands";
  import { markdown } from "@codemirror/lang-markdown";
  import { syntaxHighlighting, defaultHighlightStyle } from "@codemirror/language";
  import { search, searchKeymap } from "@codemirror/search";
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
        search({ top: true }),
        keymap.of([...defaultKeymap, ...historyKeymap, ...searchKeymap]),
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

    // Move focus into the editor so the caret is visible immediately on mode
    // switch, and so the CodeMirror searchKeymap (⌘F) reaches its handler.
    // Without this, the user sees no caret and ⌘F is a no-op until they
    // click into the editor first.
    view.focus();

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
</style>
