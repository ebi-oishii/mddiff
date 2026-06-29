<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { EditorState } from "@codemirror/state";
  import { EditorView, keymap, highlightActiveLine } from "@codemirror/view";
  import { defaultKeymap, history, historyKeymap } from "@codemirror/commands";
  import { markdown } from "@codemirror/lang-markdown";
  import { livePreviewExtension } from "./livepreview";
  import { doc } from "$lib/stores/doc.svelte";
  import { mddiffCmTheme } from "./cm-theme";
  import FindBar from "$lib/components/FindBar.svelte";
  import { findExtension } from "./find-cm.svelte";
  import { useCmFind } from "./use-find.svelte";
  import { attachScrollTracker, type ScrollTracker } from "./scroll-tracker";
  import { restoreCmToLine } from "./cm-editor";
  import { imagePaste } from "./image-paste";

  let {
    text,
    onchange,
    onerror,
  }: {
    text: string;
    onchange: (t: string) => void;
    onerror?: (msg: string) => void;
  } = $props();

  let container: HTMLDivElement;
  let view: EditorView | null = null;
  let lastEmitted = "";
  let scrollTracker: ScrollTracker | null = null;

  function topVisibleLine(): number | null {
    if (!view) return null;
    try {
      const rect = view.scrollDOM.getBoundingClientRect();
      if (rect.width === 0 && rect.height === 0) return null;
      const pos = view.posAtCoords({ x: rect.left + 8, y: rect.top + 4 });
      if (pos != null) return view.state.doc.lineAt(pos).number;
    } catch {}
    return null;
  }

  const find = useCmFind(() => {
    void find.query;
    void find.open;
  });

  onMount(() => {
    const state = EditorState.create({
      doc: text,
      extensions: [
        history(),
        highlightActiveLine(),
        findExtension(find.syncFromData),
        imagePaste((msg) => onerror?.(msg)),
        keymap.of([...defaultKeymap, ...historyKeymap]),
        markdown(),
        EditorView.lineWrapping,
        mddiffCmTheme,
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
    find.bind(view);

    // Focus on mount so the caret is visible immediately on mode switch.
    view.focus();

    scrollTracker = attachScrollTracker(view.scrollDOM, {
      computeLine: topVisibleLine,
    });

    const restore = doc.currentLine;
    requestAnimationFrame(() => {
      if (view) restoreCmToLine(view, restore);
    });
  });

  onDestroy(() => {
    scrollTracker?.captureNow();
    scrollTracker?.detach();
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
{#if find.open}
  <FindBar
    bind:query={find.query}
    bind:replaceQuery={find.replaceQuery}
    bind:replaceVisible={find.replaceVisible}
    matchCount={find.matchCount}
    currentIndex={find.currentIndex}
    focusVersion={find.focusVersion}
    enableReplace={true}
    onnext={find.next}
    onprev={find.prev}
    onreplace={find.replace}
    onreplaceAll={find.replaceAll}
    onclose={find.close}
  />
{/if}

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
  :global(.mddiff-lp-h1) {
    font-size: 1.85em;
    font-weight: 700;
    line-height: 1.25;
  }
  :global(.mddiff-lp-h2) {
    font-size: 1.5em;
    font-weight: 700;
    line-height: 1.25;
  }
  :global(.mddiff-lp-h3) {
    font-size: 1.3em;
    font-weight: 600;
    line-height: 1.3;
  }
  :global(.mddiff-lp-h4) {
    font-size: 1.15em;
    font-weight: 600;
  }
  :global(.mddiff-lp-h5) {
    font-size: 1.05em;
    font-weight: 600;
  }
  :global(.mddiff-lp-h6) {
    font-size: 1em;
    font-weight: 600;
    color: light-dark(#666, #aaa);
  }

  /* Inline styles. */
  :global(.mddiff-lp-bold) {
    font-weight: 700;
  }
  :global(.mddiff-lp-italic) {
    font-style: italic;
  }
  :global(.mddiff-lp-code) {
    background: light-dark(#f3f3f3, #2a2a2a);
    padding: 0.1em 0.35em;
    border-radius: 3px;
    font-family: ui-monospace, "SF Mono", Menlo, monospace;
    font-size: 0.92em;
  }
  :global(.mddiff-lp-link) {
    color: light-dark(#0969da, #58a6ff);
    text-decoration: underline;
    text-underline-offset: 0.15em;
  }
  /* Image syntax pill — Live Preview doesn't render the actual <img> inline,
     so we collapse the entire `![alt](src)` to a small filename badge when
     the cursor isn't on that line. When the cursor IS on the line the raw
     syntax shows again (Decoration.mark, not replace) so the user can edit. */
  :global(.mddiff-lp-image-pill) {
    display: inline-block;
    padding: 0.05em 0.5em;
    margin: 0 0.1em;
    background: light-dark(#eff6ff, #1e3a5f);
    color: light-dark(#0969da, #79b8ff);
    border: 1px solid light-dark(#bcd8fa, #2a3a55);
    border-radius: 4px;
    font-family: ui-monospace, "SF Mono", Menlo, monospace;
    font-size: 0.85em;
    line-height: 1.4;
    cursor: text;
  }
  :global(.mddiff-lp-image) {
    color: light-dark(#0969da, #79b8ff);
  }
</style>
