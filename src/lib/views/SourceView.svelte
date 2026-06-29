<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { Compartment, EditorState } from "@codemirror/state";
  import { EditorView, keymap, lineNumbers, highlightActiveLine } from "@codemirror/view";
  import { defaultKeymap, history, historyKeymap } from "@codemirror/commands";
  import { markdown } from "@codemirror/lang-markdown";
  import { syntaxHighlighting, defaultHighlightStyle } from "@codemirror/language";
  import { doc } from "$lib/stores/doc.svelte";
  import { settings } from "$lib/stores/settings.svelte";
  import { mddiffCmTheme } from "./cm-theme";
  import { mddiffSyntaxHighlighting } from "./cm-syntax";
  import FindBar from "$lib/components/FindBar.svelte";
  import { findExtension } from "./find-cm.svelte";
  import { useCmFind } from "./use-find.svelte";
  import { attachScrollTracker, type ScrollTracker } from "./scroll-tracker";
  import { restoreCmToLine } from "./cm-editor";
  import { imagePaste } from "./image-paste";
  import { linkClickCmExtension } from "./link-click-cm";

  let {
    text,
    onchange,
    onerror,
  }: {
    text: string;
    onchange: (t: string) => void;
    /** Surface image-paste failures to the host's error banner. */
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
      // Detached element returns a zero rect — bail before posAtCoords maps
      // (0, 0) to position 0 and clobbers currentLine with 1.
      if (rect.width === 0 && rect.height === 0) return null;
      const pos = view.posAtCoords({ x: rect.left + 8, y: rect.top + 4 });
      if (pos != null) return view.state.doc.lineAt(pos).number;
    } catch {}
    return null;
  }

  const find = useCmFind(() => {
    void text;
    void find.query;
    void find.open;
  });

  // Active-line extension overlay: paints `--mddiff-active-line-bg` across the
  // full .source width at the current line's y. The right 3rem padding
  // strip lies outside cm-editor, so without this the highlight visibly
  // stops 3rem before the viewport edge. We position the .source::before
  // by writing CSS vars (--mddiff-source-active-y / --mddiff-source-active-h);
  // the actual paint happens in style {}.
  //
  // We measure the rendered .cm-activeLine element directly (instead of
  // computing y from lineBlockAt) because CM's BlockInfo.top excludes
  // cm-content's 4px top padding, which would offset our extension strip
  // and make it look misaligned with the in-editor highlight.
  function updateActiveLine() {
    if (!view || !container) return;
    try {
      const lineEl = view.dom.querySelector(
        ".cm-activeLine",
      ) as HTMLElement | null;
      if (!lineEl) {
        // No active line (e.g. multi-line selection mid-drag) — hide the
        // overlay by pushing it off-screen.
        container.style.setProperty("--mddiff-source-active-y", "-9999px");
        container.style.setProperty("--mddiff-source-active-h", "0");
        return;
      }
      const lineRect = lineEl.getBoundingClientRect();
      const sourceRect = container.getBoundingClientRect();
      container.style.setProperty(
        "--mddiff-source-active-y",
        `${lineRect.top - sourceRect.top}px`,
      );
      container.style.setProperty(
        "--mddiff-source-active-h",
        `${lineRect.height}px`,
      );
    } catch {
      // DOM might be mid-teardown; ignore.
    }
  }

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
        // Markdown-tag colors first (heading / code / link / etc.); the
        // default highlight runs as a fallback so non-markdown tags
        // (inside fenced code blocks etc.) still get something sensible.
        mddiffSyntaxHighlighting,
        syntaxHighlighting(defaultHighlightStyle, { fallback: true }),
        findExtension(find.syncFromData),
        imagePaste((msg) => onerror?.(msg)),
        linkClickCmExtension({ getDocPath: () => doc.path }),
        keymap.of([...defaultKeymap, ...historyKeymap]),
        markdown(),
        wrapComp.of(settings.softWrap ? EditorView.lineWrapping : []),
        tabSizeComp.of(EditorState.tabSize.of(settings.tabWidth)),
        mddiffCmTheme,
        EditorView.updateListener.of((u) => {
          if (u.docChanged) {
            const next = u.state.doc.toString();
            lastEmitted = next;
            onchange(next);
          }
          if (u.selectionSet || u.docChanged || u.geometryChanged) {
            updateActiveLine();
          }
        }),
      ],
    });
    view = new EditorView({ state, parent: container });
    lastEmitted = text;

    find.bind(view);

    // Move focus into the editor so the caret is visible immediately on mode
    // switch. Without this the user sees no caret on entry to the view.
    view.focus();

    // Paint the active-line extension once on mount; future updates come from
    // the updateListener and the scrollDOM scroll handler below.
    requestAnimationFrame(updateActiveLine);
    view.scrollDOM.addEventListener("scroll", updateActiveLine, { passive: true });

    scrollTracker = attachScrollTracker(view.scrollDOM, {
      computeLine: topVisibleLine,
    });

    // Restore scroll position from DocStore so mode switches stay in place.
    // Defer one frame so CodeMirror has measured the layout.
    const restore = doc.currentLine;
    requestAnimationFrame(() => {
      if (view) restoreCmToLine(view, restore);
    });
  });

  onDestroy(() => {
    // Last-chance capture in case a scroll happened in the final ~80ms and
    // the debounce hasn't fired yet. Best-effort; guarded inside.
    scrollTracker?.captureNow();
    scrollTracker?.detach();
    view?.scrollDOM.removeEventListener("scroll", updateActiveLine);
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
  .source {
    height: 100%;
    overflow: hidden;
  }
  /* CodeMirror styling lives in $lib/views/cm-theme.ts (delivered via
     EditorView.theme so it beats CM defaults on specificity). Only
     editor-instance-specific bits remain here. */
  :global(.cm-editor) {
    font-family: ui-monospace, "SF Mono", Menlo, Consolas, monospace;
    font-size: var(--mddiff-editor-font-size, 14px);
  }
  /* Reserve a right strip so long lines don't slide under the floating ☰
     button (top-right, 34px + 12px inset + shadow ≈ 54px).
     Padding goes on `.source` (the outer host), NOT on cm-scroller.
     CodeMirror computes line-wrap width from `cm-scroller.clientWidth`,
     and clientWidth *includes* padding — so padding on cm-scroller
     shrinks the visible box but leaves the wrap point unchanged, with
     cm-line happily rendering past cm-content's right edge. Putting the
     padding on `.source` makes cm-scroller itself narrower (its parent
     now reserves space), so clientWidth is the correct (shrunken) value
     and wrap actually fires earlier.
     Skipped in fullscreen: the 2.5rem top padding already moves content
     below the title overlay, and the ☰ menu sits next to the overlay
     in the OS-title-bar-free area, not on top of text. */
  :global(:root:not([data-fullscreen])) .source {
    padding-right: 3rem;
    box-sizing: border-box;
    /* Anchor the active-line extension ::before to .source. */
    position: relative;
  }
  /* Active-line extension: paints the highlight color across the full
     .source width (including the 3rem padding strip) at the current
     line's y. CM's own .cm-activeLine still paints inside cm-editor,
     and cm-editor's solid background sits *above* this ::before (later
     in document order, same stacking context), so the bar is visible
     only where cm-editor doesn't cover — i.e. the right 3rem strip.
     Position vars are written by updateActiveLine() in the script. */
  :global(:root:not([data-fullscreen])) .source::before {
    content: "";
    position: absolute;
    left: 0;
    right: 0;
    top: var(--mddiff-source-active-y, -9999px);
    height: var(--mddiff-source-active-h, 0);
    background: var(--mddiff-active-line-bg);
    pointer-events: none;
  }
  /* In fullscreen the floating "(filename) MODE" overlay (rendered by
     +page.svelte at top-left) covers the first line of source because
     CodeMirror's default content padding is only a few pixels. Live /
     Preview / WYSIWYG have intrinsic 2rem top padding so they're fine. */
  :global(:root[data-fullscreen] .source .cm-scroller) {
    padding-top: 2.5rem;
  }
</style>
