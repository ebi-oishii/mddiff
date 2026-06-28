<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { EditorState } from "@codemirror/state";
  import { EditorView, keymap, lineNumbers, highlightActiveLine } from "@codemirror/view";
  import { defaultKeymap, history, historyKeymap } from "@codemirror/commands";
  import { markdown } from "@codemirror/lang-markdown";
  import { syntaxHighlighting, defaultHighlightStyle } from "@codemirror/language";
  import { doc } from "$lib/stores/doc.svelte";
  import { mdvCmTheme } from "./cm-theme";
  import FindBar from "$lib/components/FindBar.svelte";
  import { CmFindState, findExtension } from "./find-cm.svelte";

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

  const find = new CmFindState();

  // Active-line extension overlay: paints `--mdv-active-line-bg` across the
  // full .source width at the current line's y. The right 3rem padding
  // strip lies outside cm-editor, so without this the highlight visibly
  // stops 3rem before the viewport edge. We position the .source::before
  // by writing CSS vars (--mdv-source-active-y / --mdv-source-active-h);
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
        container.style.setProperty("--mdv-source-active-y", "-9999px");
        container.style.setProperty("--mdv-source-active-h", "0");
        return;
      }
      const lineRect = lineEl.getBoundingClientRect();
      const sourceRect = container.getBoundingClientRect();
      container.style.setProperty(
        "--mdv-source-active-y",
        `${lineRect.top - sourceRect.top}px`,
      );
      container.style.setProperty(
        "--mdv-source-active-h",
        `${lineRect.height}px`,
      );
    } catch {
      // DOM might be mid-teardown; ignore.
    }
  }

  onMount(() => {
    const state = EditorState.create({
      doc: text,
      extensions: [
        history(),
        lineNumbers(),
        highlightActiveLine(),
        syntaxHighlighting(defaultHighlightStyle, { fallback: true }),
        findExtension(find.syncFromData),
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
          if (u.selectionSet || u.docChanged || u.geometryChanged) {
            updateActiveLine();
          }
        }),
      ],
    });
    view = new EditorView({ state, parent: container });
    lastEmitted = text;

    find.bind(view);
    window.addEventListener("keydown", find.onKeydown);

    // Move focus into the editor so the caret is visible immediately on mode
    // switch. Without this the user sees no caret on entry to the view.
    view.focus();

    // Paint the active-line extension once on mount; future updates come from
    // the updateListener and the scrollDOM scroll handler below.
    requestAnimationFrame(updateActiveLine);
    view.scrollDOM.addEventListener("scroll", updateActiveLine, { passive: true });

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
    window.removeEventListener("keydown", find.onKeydown);
    find.destroy();
    if (scrollTimer) clearTimeout(scrollTimer);
    // Last-chance capture in case a scroll event happened in the final ~80ms
    // and the debounce hasn't fired yet. Best-effort; guarded inside.
    captureTopLine();
    view?.scrollDOM.removeEventListener("scroll", onScroll);
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

  $effect(() => {
    void find.query;
    void find.open;
    find.refresh();
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
    font-size: var(--mdv-editor-font-size, 14px);
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
    top: var(--mdv-source-active-y, -9999px);
    height: var(--mdv-source-active-h, 0);
    background: var(--mdv-active-line-bg);
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
