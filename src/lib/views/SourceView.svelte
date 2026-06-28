<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { Compartment, EditorState } from "@codemirror/state";
  import { EditorView, keymap, lineNumbers, highlightActiveLine } from "@codemirror/view";
  import { defaultKeymap, history, historyKeymap } from "@codemirror/commands";
  import { markdown } from "@codemirror/lang-markdown";
  import { syntaxHighlighting, defaultHighlightStyle } from "@codemirror/language";
  import { doc } from "$lib/stores/doc.svelte";
  import { settings } from "$lib/stores/settings.svelte";
  import { mdvCmTheme } from "./cm-theme";

  let {
    text,
    onchange,
  }: { text: string; onchange: (t: string) => void } = $props();

  let container: HTMLDivElement;
  let view: EditorView | null = null;
  let lastEmitted = "";

  // Compartments let us swap extensions at runtime without rebuilding the
  // EditorState. Used to honor settings.softWrap / lineNumbers / tabWidth
  // without losing scroll position or selection when the user changes them.
  const wrapComp = new Compartment();
  const lineNumComp = new Compartment();
  const tabSizeComp = new Compartment();

  onMount(() => {
    const state = EditorState.create({
      doc: text,
      extensions: [
        history(),
        lineNumComp.of(settings.lineNumbers ? lineNumbers() : []),
        highlightActiveLine(),
        syntaxHighlighting(defaultHighlightStyle, { fallback: true }),
        keymap.of([...defaultKeymap, ...historyKeymap]),
        markdown(),
        wrapComp.of(settings.softWrap ? EditorView.lineWrapping : []),
        tabSizeComp.of(EditorState.tabSize.of(settings.tabWidth)),
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

  // Reactively re-configure the CM compartments when their settings change,
  // so toggling Soft wrap / Line numbers / Tab width takes effect live.
  $effect(() => {
    if (!view) return;
    view.dispatch({
      effects: wrapComp.reconfigure(
        settings.softWrap ? EditorView.lineWrapping : [],
      ),
    });
  });
  $effect(() => {
    if (!view) return;
    view.dispatch({
      effects: lineNumComp.reconfigure(
        settings.lineNumbers ? lineNumbers() : [],
      ),
    });
  });
  $effect(() => {
    if (!view) return;
    view.dispatch({
      effects: tabSizeComp.reconfigure(EditorState.tabSize.of(settings.tabWidth)),
    });
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
