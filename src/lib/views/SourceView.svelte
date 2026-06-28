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
  import { markdownSpellcheckMask } from "./cm-spellcheck-mask";
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
  const spellcheckComp = new Compartment();

  // Compartment-able fragment that toggles browser/OS-native spellcheck on
  // CM's contenteditable (`.cm-content`). When off we explicitly set
  // "false" rather than leaving the attribute absent — some browsers
  // default to true on contenteditable.
  function spellcheckExt(on: boolean) {
    return EditorView.contentAttributes.of({ spellcheck: on ? "true" : "false" });
  }

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
        spellcheckComp.of(spellcheckExt(settings.spellcheck)),
        markdownSpellcheckMask,
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

  // Outline sidebar jumps surface as `doc.pendingScrollLine`. Honor it once,
  // then clear so we don't re-scroll on every doc.text mutation.
  $effect(() => {
    const line = doc.pendingScrollLine;
    if (line == null || !view) return;
    restoreCmToLine(view, line);
    doc.pendingScrollLine = null;
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
  $effect(() => {
    if (!view) return;
    view.dispatch({
      effects: spellcheckComp.reconfigure(spellcheckExt(settings.spellcheck)),
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
  /* Anchor the active-line extension ::before to .source. */
  .source {
    position: relative;
  }
  /* Reserve a right strip so long lines don't slide under the floating ☰
     button (top-right, 34px + 12px inset + shadow ≈ 54px).
     Padding goes on `.cm-content` (the inner content box), NOT on
     `.source` or `.cm-scroller`. Reasoning:
       - On `.cm-scroller`: clientWidth *includes* padding so CM's
         wrap heuristic doesn't shrink the wrap point — long lines
         still render past cm-content's right edge.
       - On `.source` (the outer host): does correctly shrink the wrap
         point, but it also pushes cm-scroller (the scrollbar's host)
         3rem in from the viewport right — leaving the scrollbar
         floating in the middle of the right strip instead of flush
         against the window edge.
       - On `.cm-content`: cm-line wraps within cm-content's content
         box (excluding padding), so wrap fires 3rem early; cm-scroller
         stays full-width so its scrollbar lands on the viewport right
         edge.
     Skipped in fullscreen: the 2.5rem top padding already moves content
     below the title overlay, and the ☰ menu sits next to the overlay
     in the OS-title-bar-free area, not on top of text. */
  :global(:root:not([data-fullscreen]) .source .cm-content) {
    padding-right: 3rem;
  }
  /* Active-line extension: paints the highlight color into the right
     3rem strip (where cm-content's padding lives) at the current line's
     y. CM's own .cm-activeLine handles the cm-line area, so this only
     needs to cover the gap.
     cm-editor's solid background fills that padding region too, and is
     rendered after ::before in document order — so we lift ::before
     with z-index:1 to actually be visible over it.
     The strip uses the same translucent `--mddiff-active-line-bg`
     token; visually identical to cm-activeLine on the same row.
     Position vars are written by updateActiveLine() in the script. */
  :global(:root:not([data-fullscreen])) .source::before {
    content: "";
    position: absolute;
    right: 0;
    width: 3rem;
    top: var(--mddiff-source-active-y, -9999px);
    height: var(--mddiff-source-active-h, 0);
    background: var(--mddiff-active-line-bg);
    pointer-events: none;
    z-index: 1;
  }
  /* In fullscreen the floating "(filename) MODE" overlay (rendered by
     +page.svelte at top-left) covers the first line of source because
     CodeMirror's default content padding is only a few pixels. Live /
     Preview / WYSIWYG have intrinsic 2rem top padding so they're fine. */
  :global(:root[data-fullscreen] .source .cm-scroller) {
    padding-top: 2.5rem;
  }
</style>
