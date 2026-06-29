<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { Editor, defaultValueCtx, editorViewCtx, rootCtx } from "@milkdown/kit/core";
  import { commonmark } from "@milkdown/kit/preset/commonmark";
  import { gfm } from "@milkdown/kit/preset/gfm";
  import { listener, listenerCtx } from "@milkdown/kit/plugin/listener";
  import { getMarkdown, replaceAll } from "@milkdown/kit/utils";
  // ProseMirror's view module logs a console warning if `white-space: pre-wrap`
  // (and a few other base styles) aren't on the contenteditable. Without
  // those styles PM degrades to a code path that may misbehave on layout
  // edges. Importing the stylesheet adds the rules globally.
  import "prosemirror-view/style/prosemirror.css";
  import FindBar from "$lib/components/FindBar.svelte";
  import { useFind } from "./use-find.svelte";
  import { attachScrollTracker, type ScrollTracker } from "./scroll-tracker";
  import { createLineMapMd } from "./markdown-render";
  import { rewriteRelativeImageSrc } from "./image-path";
  import { handleLinkClick } from "./link-click";
  import { doc } from "$lib/stores/doc.svelte";
  import { settings } from "$lib/stores/settings.svelte";

  let {
    text,
    onchange,
    onnormalize,
  }: {
    text: string;
    onchange: (t: string) => void;
    onnormalize?: (orig: string, normalized: string) => void;
  } = $props();

  let container: HTMLDivElement;
  let editor: Editor | null = null;
  let lastEmitted = "";
  let ready = $state(false);
  let scrollTracker: ScrollTracker | null = null;
  let imageObserver: MutationObserver | null = null;

  // The previous version of this view also ran a MutationObserver that
  // stamped `spellcheck="false"` on code / pre nodes (so prose got spell-
  // checked but identifiers in code didn't). That triggered an interaction
  // with ProseMirror's own DOM observer: PM treats external attribute
  // mutations as content changes and re-renders the node, which produces
  // childList mutations, which fires our observer again, which sets the
  // attribute, which PM re-renders... infinite loop, freezes the main
  // thread, breaks the entire app after WYSIWYG mounts on any non-empty
  // doc. See <internal commit history>. For v0.2.2 we drop the mask
  // entirely — when the user enables spellcheck, code identifiers may
  // also get red underlines. Acceptable trade-off until we move the mask
  // into a Milkdown plugin that owns the attribute via PM's node spec.

  /**
   * Walk every `<img>` in the editor and rewrite relative src to Tauri's
   * `asset://` URL. Idempotent (resolved URLs aren't relative anymore).
   */
  function rewriteImages() {
    if (!container) return;
    for (const img of container.querySelectorAll<HTMLImageElement>("img")) {
      const src = img.getAttribute("src");
      if (!src) continue;
      const rewritten = rewriteRelativeImageSrc(src, doc.path);
      if (rewritten !== src) img.setAttribute("src", rewritten);
    }
  }

  // Milkdown doesn't expose source-line positions on its rendered nodes, so we
  // reconstruct a top-level-block → source-line mapping by parsing the same
  // markdown with markdown-it. ProseMirror renders top-level doc nodes 1:1 as
  // `.ProseMirror` children, so the array index lines up with the child index.
  const lineMapMd = createLineMapMd();
  let cachedSrc = "";
  let cachedLines: number[] = [];
  function topLevelBlockLines(src: string): number[] {
    if (src === cachedSrc) return cachedLines;
    const tokens = lineMapMd.parse(src, {});
    const lines: number[] = [];
    for (const tok of tokens) {
      if (tok.level !== 0) continue;
      if (!tok.map) continue;
      if (tok.type === "inline") continue;
      if (tok.type.endsWith("_close")) continue;
      lines.push(tok.map[0] + 1);
    }
    cachedSrc = src;
    cachedLines = lines;
    return lines;
  }

  function proseMirrorRoot(): HTMLElement | null {
    return container?.querySelector(".ProseMirror") as HTMLElement | null;
  }

  function scrollToLine(line: number) {
    const root = proseMirrorRoot();
    if (!root || !container) return;
    const lines = topLevelBlockLines(lastEmitted || text);
    if (lines.length === 0) return;
    // largest i with lines[i] <= line; if line < lines[0], stay at the top.
    let target = 0;
    for (let i = 0; i < lines.length; i++) {
      if (lines[i] <= line) target = i;
      else break;
    }
    const children = root.children;
    if (target >= children.length) target = children.length - 1;
    const el = children[target] as HTMLElement | undefined;
    if (el) container.scrollTop = el.offsetTop;
  }

  function topVisibleLine(): number | null {
    const root = proseMirrorRoot();
    if (!root || !container) return null;
    const rect = container.getBoundingClientRect();
    // Detached element returns a zero rect — bail to avoid clobbering
    // currentLine with line 1.
    if (rect.width === 0 && rect.height === 0) return null;
    const top = rect.top;
    const children = Array.from(root.children) as HTMLElement[];
    if (children.length === 0) return null;
    let blockIndex = 0;
    for (let i = 0; i < children.length; i++) {
      const cr = children[i].getBoundingClientRect();
      if (cr.top >= top - 2) {
        blockIndex = i;
        break;
      }
      blockIndex = i;
    }
    const lines = topLevelBlockLines(lastEmitted || text);
    return lines[blockIndex] ?? null;
  }

  // DOM-based find — same pattern as Preview / Diff. The scope is `.wys`
  // (the bound container) which wraps `.ProseMirror`, so matches in the
  // rendered Markdown are highlighted in place. Editing in WYSIWYG
  // re-renders nodes (wiping the marks), so the $effect re-applies on
  // every text update.
  const find = useFind(() => {
    void text;
    void find.query;
    void find.open;
  });

  // Click anywhere on the rendered `☐` / `☑` glyph (drawn via .cm-line's
  // ::before, so the click target is the `<li>` itself within ~24px from
  // its left edge) toggles the task's checked state on the underlying
  // ProseMirror node. The markdown-updated listener propagates the change
  // to doc.text and the dirty flag.
  function handleWysiwygLinkClick(e: MouseEvent) {
    handleLinkClick(e, {
      getDocPath: () => doc.path,
      // WYSIWYG is an editor — plain clicks must place the caret so users
      // can edit link text. Require ⌘/Ctrl to navigate, matching VSCode /
      // Typora / JetBrains conventions.
      requireModifier: true,
      // No in-view anchor scroll for WYSIWYG yet — Milkdown's editor view
      // doesn't expose heading ids in the rendered DOM the same way
      // markdown-it does. `#anchor` clicks become no-ops here.
    });
  }

  function handleTaskClick(e: MouseEvent) {
    if (!editor) return;
    const target = e.target as HTMLElement;
    const li = target.closest<HTMLElement>('li[data-item-type="task"]');
    if (!li) return;
    // Limit to the left ~24px (matches the ::before width) so clicks on
    // task body text still place the caret as usual.
    const rect = li.getBoundingClientRect();
    if (e.clientX - rect.left > 24) return;
    e.preventDefault();
    editor.action((ctx) => {
      const view = ctx.get(editorViewCtx);
      const pos = view.posAtDOM(li, 0);
      if (pos < 0) return;
      const resolved = view.state.doc.resolve(pos);
      // Milkdown's GFM preset doesn't introduce a separate `task_list_item`
      // node — it extends the base `list_item` schema with a nullable
      // `checked` attr. A regular bullet/ordered item has `checked === null`;
      // a task item has `checked: true | false`. Walk ancestors and toggle
      // the first `list_item` that has a non-null `checked`.
      for (let d = resolved.depth; d >= 0; d--) {
        const node = resolved.node(d);
        if (node.type.name === "list_item" && node.attrs.checked != null) {
          const nodePos = resolved.before(d);
          view.dispatch(
            view.state.tr.setNodeMarkup(nodePos, null, {
              ...node.attrs,
              checked: !node.attrs.checked,
            }),
          );
          return;
        }
      }
    });
  }

  onMount(async () => {
    find.bind(container);

    const initial = text;
    try {
      editor = await Editor.make()
        .config((ctx) => {
          ctx.set(rootCtx, container);
          ctx.set(defaultValueCtx, initial);
          ctx.get(listenerCtx).markdownUpdated((_ctx, markdown) => {
            if (!ready) return;
            try {
              if (markdown !== lastEmitted) {
                lastEmitted = markdown;
                onchange(markdown);
              }
            } catch (err) {
              console.error("[mddiff] WYSIWYG markdownUpdated handler", err);
            }
          });
        })
        .use(commonmark)
        .use(gfm)
        .use(listener)
        .create();
    } catch (err) {
      console.error("[mddiff] WYSIWYG editor build failed", err);
      return;
    }
    if (!editor) return;

    container.addEventListener("click", handleTaskClick);

    // Link click delegation. ProseMirror also handles link clicks internally
    // (positions the caret) — we listen at the container so we fire before
    // bubbling reaches PM, and `event.preventDefault` blocks PM from
    // following the link inside the editor view.
    container.addEventListener("click", handleWysiwygLinkClick);

    // Milkdown renders `<img>` with the verbatim Markdown src (a relative
    // path for pasted images). The Tauri webview can't load `<doc-dir>/x.png`
    // directly — it needs `convertFileSrc()`'s asset:// URL. We can't
    // override Milkdown's image schema cleanly here, so we watch the
    // ProseMirror DOM and rewrite img src on the fly. The rewrite is
    // idempotent: a resolved asset:// URL is no longer "relative" so the
    // observer's re-fire after our setAttribute is a no-op.
    imageObserver = new MutationObserver(() => {
      try { rewriteImages(); } catch (err) {
        console.error("[mddiff] WYSIWYG rewriteImages", err);
      }
    });
    imageObserver.observe(container, {
      childList: true,
      subtree: true,
      attributes: true,
      attributeFilter: ["src"],
    });
    rewriteImages();

    // After load, query Milkdown's own serialization of the doc to detect
    // round-trip normalization (e.g. `*foo*` <-> `_foo_`, link reference
    // expansion, trailing newline adjustment). If different from what we
    // loaded, surface it via `onnormalize` so the parent can both warn the
    // user AND adopt the normalized form as the new baseline (so the dirty
    // indicator doesn't appear just from opening WYSIWYG).
    //
    // We intentionally do NOT call `onchange` here — that's reserved for
    // genuine user edits via the listener.
    try {
      const serialized = editor.action(getMarkdown());
      lastEmitted = serialized;
      if (serialized.trim() !== initial.trim()) {
        onnormalize?.(initial, serialized);
      }
    } catch {
      // getMarkdown not available in this build; skip detection silently.
    }

    ready = true;
    // Note: previous versions ran a MutationObserver here to mask
    // spellcheck="false" on code/pre as Milkdown re-rendered. It deadlocked
    // with ProseMirror's own DOM observer (PM revert-cycles foreign attrs,
    // we re-stamp, PM reverts...). See the comment near `imageObserver`.

    // Restore scroll position last so Milkdown's render has been committed
    // and lastEmitted (post-normalization) is set for an accurate line map.
    const restore = doc.currentLine;
    requestAnimationFrame(() => {
      try {
        scrollToLine(restore);
      } catch {}
      // Attach scroll tracker after the restore frame so the programmatic
      // scroll doesn't race the user's first manual scroll for the debounce.
      if (container) {
        scrollTracker = attachScrollTracker(container, {
          computeLine: topVisibleLine,
        });
      }
    });
  });

  onDestroy(() => {
    // Block any markdownUpdated callback fired during teardown from
    // propagating into doc.text — Milkdown's destroy can emit one final
    // update as ProseMirror disposes.
    ready = false;

    imageObserver?.disconnect();
    imageObserver = null;

    container?.removeEventListener("click", handleTaskClick);
    container?.removeEventListener("click", handleWysiwygLinkClick);

    try {
      scrollTracker?.captureNow();
    } catch {
      // DOM might already be torn down; skip silently.
    }
    scrollTracker?.detach();
    scrollTracker = null;

    try {
      editor?.destroy();
    } catch (err) {
      console.error("[mddiff] WYSIWYG editor.destroy", err);
    }
    editor = null;
  });

  $effect(() => {
    if (editor && ready && text !== lastEmitted) {
      lastEmitted = text;
      editor.action(replaceAll(text));
    }
  });

  // Outline sidebar jumps surface as `doc.pendingScrollLine`. Defer to next
  // frame so any pending Milkdown re-render is committed before scrollToLine
  // queries .ProseMirror children.
  $effect(() => {
    const line = doc.pendingScrollLine;
    if (line == null || !ready) return;
    requestAnimationFrame(() => {
      try {
        scrollToLine(line);
      } catch {}
      doc.pendingScrollLine = null;
    });
  });

  // Browser/OS-native spellcheck on the .ProseMirror contenteditable.
  // Milkdown doesn't expose a first-class API for editor-host attributes,
  // so we set it directly. Gate on `ready` so we wait for the element
  // (Milkdown creates .ProseMirror inside our container only after init).
  $effect(() => {
    const on = settings.spellcheck;
    if (!ready) return;
    const pm = container?.querySelector(".ProseMirror") as HTMLElement | null;
    if (pm) pm.setAttribute("spellcheck", on ? "true" : "false");
  });

</script>

<div bind:this={container} class="wys"></div>
{#if find.open}
  <FindBar
    bind:query={find.query}
    matchCount={find.matchCount}
    currentIndex={find.currentIndex}
    focusVersion={find.focusVersion}
    onnext={find.next}
    onprev={find.prev}
    onclose={find.close}
  />
{/if}

<style>
  .wys {
    height: 100%;
    overflow: auto;
  }
  :global(.wys .milkdown) {
    max-width: 92ch;
    margin: 0 auto;
    padding: 2rem 3rem 4rem;
    font-family:
      system-ui,
      -apple-system,
      "Segoe UI",
      Roboto,
      "Hiragino Sans",
      "Yu Gothic",
      sans-serif;
    font-size: 16px;
    line-height: 1.7;
    color: light-dark(#222, #ddd);
  }
  /* In fullscreen the title overlay (set by +page.svelte) sits at the top
     of the canvas and clips the first paragraph if the content's own 2rem
     padding doesn't quite clear it. Match SourceView by widening to 2.5rem
     just for fullscreen. */
  :global(:root[data-fullscreen] .wys .milkdown) {
    padding-top: 2.5rem;
  }
  :global(.wys .ProseMirror) {
    outline: none;
    min-height: 100%;
  }
  :global(.wys h1) {
    font-size: 2rem;
    font-weight: 700;
    margin: 1.5em 0 0.5em;
    border-bottom: 1px solid light-dark(#eee, #333);
    padding-bottom: 0.3em;
  }
  :global(.wys h2) {
    font-size: 1.5rem;
    font-weight: 700;
    margin: 1.5em 0 0.5em;
    border-bottom: 1px solid light-dark(#eee, #333);
    padding-bottom: 0.3em;
  }
  :global(.wys h3) {
    font-size: 1.25rem;
    font-weight: 600;
    margin: 1.25em 0 0.5em;
  }
  :global(.wys p) {
    margin: 0.75em 0;
  }
  :global(.wys code) {
    background: light-dark(#f3f3f3, #2a2a2a);
    padding: 0.15em 0.4em;
    border-radius: 3px;
    font-size: 0.9em;
    font-family: ui-monospace, "SF Mono", Menlo, monospace;
  }
  :global(.wys pre) {
    background: light-dark(#f5f5f5, #1f1f1f);
    padding: 1em;
    border-radius: 6px;
    overflow: auto;
    margin: 1em 0;
  }
  :global(.wys pre code) {
    background: transparent;
    padding: 0;
  }
  :global(.wys blockquote) {
    margin: 1em 0;
    padding: 0 1em;
    border-left: 4px solid light-dark(#ddd, #444);
    color: light-dark(#666, #aaa);
  }
  :global(.wys a) {
    color: light-dark(#0969da, #58a6ff);
  }
  :global(.wys ul),
  :global(.wys ol) {
    padding-left: 1.5em;
    margin: 0.75em 0;
  }
  :global(.wys li) {
    margin: 0.2em 0;
  }
  /* GFM task list items: Milkdown emits attributes only, draw the box ourselves. */
  :global(.wys li[data-item-type="task"]) {
    list-style: none;
    margin-left: -0.4em;
  }
  :global(.wys li[data-item-type="task"] > p) {
    display: inline;
    margin: 0;
  }
  :global(.wys li[data-item-type="task"]::before) {
    content: "☐ ";
    display: inline-block;
    width: 1.2em;
    color: light-dark(#888, #888);
    font-size: 1.05em;
    vertical-align: -1px;
    cursor: pointer;
  }
  :global(.wys li[data-item-type="task"][data-checked="true"]::before) {
    content: "☑ ";
    color: light-dark(#2ea043, #3fb950);
  }
  :global(.wys table) {
    border-collapse: collapse;
    margin: 1em 0;
  }
  :global(.wys th),
  :global(.wys td) {
    border: 1px solid light-dark(#ddd, #444);
    padding: 0.5em 0.8em;
  }
  :global(.wys img) {
    max-width: 100%;
  }
  :global(.wys hr) {
    border: 0;
    border-top: 1px solid light-dark(#ddd, #444);
    margin: 1.5em 0;
  }
</style>
