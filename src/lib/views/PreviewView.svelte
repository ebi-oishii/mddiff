<script lang="ts">
  import MarkdownIt from "markdown-it";
  import DOMPurify from "dompurify";
  import taskLists from "markdown-it-task-lists";

  let { text }: { text: string } = $props();

  const md = new MarkdownIt({
    html: true,
    linkify: true,
    breaks: false,
    typographer: true,
  });
  md.use(taskLists, { enabled: false, label: false });

  const html = $derived(DOMPurify.sanitize(md.render(text)));
</script>

<div class="preview-scroller">
  <article class="preview">
    {@html html}
  </article>
</div>

<style>
  .preview-scroller {
    height: 100%;
    overflow: auto;
  }
  .preview {
    max-width: 92ch;
    margin: 0 auto;
    padding: 2rem 3rem 4rem;
    line-height: 1.7;
    font-size: 16px;
  }
  .preview :global(h1) {
    font-size: 2rem;
    margin: 1.5em 0 0.5em;
    border-bottom: 1px solid light-dark(#eee, #333);
    padding-bottom: 0.3em;
  }
  .preview :global(h2) {
    font-size: 1.5rem;
    margin: 1.5em 0 0.5em;
    border-bottom: 1px solid light-dark(#eee, #333);
    padding-bottom: 0.3em;
  }
  .preview :global(h3) {
    font-size: 1.25rem;
    margin: 1.25em 0 0.5em;
  }
  .preview :global(p) {
    margin: 0.75em 0;
  }
  .preview :global(code) {
    background: light-dark(#f5f5f5, #2a2a2a);
    padding: 0.15em 0.4em;
    border-radius: 3px;
    font-size: 0.9em;
    font-family: ui-monospace, monospace;
  }
  .preview :global(pre) {
    background: light-dark(#f5f5f5, #1f1f1f);
    padding: 1em;
    border-radius: 6px;
    overflow: auto;
  }
  .preview :global(pre code) {
    background: transparent;
    padding: 0;
  }
  .preview :global(blockquote) {
    margin: 1em 0;
    padding: 0 1em;
    border-left: 4px solid light-dark(#ddd, #444);
    color: light-dark(#666, #aaa);
  }
  .preview :global(a) {
    color: light-dark(#0969da, #58a6ff);
  }
  .preview :global(ul),
  .preview :global(ol) {
    padding-left: 1.5em;
  }
  .preview :global(li.task-list-item) {
    list-style: none;
    margin-left: -1.5em;
  }
  .preview :global(li.task-list-item input.task-list-item-checkbox) {
    margin-right: 0.5em;
    cursor: default;
    vertical-align: middle;
  }
  .preview :global(table) {
    border-collapse: collapse;
    margin: 1em 0;
  }
  .preview :global(th),
  .preview :global(td) {
    border: 1px solid light-dark(#ddd, #444);
    padding: 0.5em 0.8em;
  }
  .preview :global(img) {
    max-width: 100%;
  }
</style>
