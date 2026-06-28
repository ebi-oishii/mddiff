<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { Editor, defaultValueCtx, editorViewCtx, rootCtx } from "@milkdown/kit/core";
  import { commonmark } from "@milkdown/kit/preset/commonmark";
  import { gfm } from "@milkdown/kit/preset/gfm";
  import { listener, listenerCtx } from "@milkdown/kit/plugin/listener";
  import { getMarkdown, replaceAll } from "@milkdown/kit/utils";

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

  // Click anywhere on the rendered `☐` / `☑` glyph (drawn via .cm-line's
  // ::before, so the click target is the `<li>` itself within ~24px from
  // its left edge) toggles the task's checked state on the underlying
  // ProseMirror node. The markdown-updated listener propagates the change
  // to doc.text and the dirty flag.
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
      // Walk up from the click target's position until we hit the task
      // list item node. Milkdown's GFM preset names it either
      // `task_list_item` (Lezer-style) or `taskListItem` (camelCase
      // depending on version) — accept both.
      for (let d = resolved.depth; d >= 0; d--) {
        const node = resolved.node(d);
        if (
          node.type.name === "task_list_item" ||
          node.type.name === "taskListItem"
        ) {
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
    const initial = text;
    editor = await Editor.make()
      .config((ctx) => {
        ctx.set(rootCtx, container);
        ctx.set(defaultValueCtx, initial);
        ctx.get(listenerCtx).markdownUpdated((_ctx, markdown) => {
          if (markdown !== lastEmitted) {
            lastEmitted = markdown;
            onchange(markdown);
          }
        });
      })
      .use(commonmark)
      .use(gfm)
      .use(listener)
      .create();

    container.addEventListener("click", handleTaskClick);

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
  });

  onDestroy(() => {
    container?.removeEventListener("click", handleTaskClick);
    editor?.destroy();
    editor = null;
  });

  $effect(() => {
    if (editor && ready && text !== lastEmitted) {
      lastEmitted = text;
      editor.action(replaceAll(text));
    }
  });
</script>

<div bind:this={container} class="wys"></div>

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
