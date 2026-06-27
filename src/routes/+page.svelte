<script lang="ts">
  import { doc } from "$lib/stores/doc.svelte";
  import { pickAndReadFile, pickAndWriteFile, writeFile } from "$lib/ipc/fs";
  import { gitIsRepo } from "$lib/ipc/git";
  import ModeBar from "$lib/components/ModeBar.svelte";
  import SourceView from "$lib/views/SourceView.svelte";
  import LivePreviewView from "$lib/views/LivePreviewView.svelte";
  import WysiwygView from "$lib/views/WysiwygView.svelte";
  import PreviewView from "$lib/views/PreviewView.svelte";
  import DiffView from "$lib/views/DiffView.svelte";
  import type { Mode } from "$lib/types";

  let mode = $state<Mode>("source");
  let error = $state<string | null>(null);
  let normalization = $state<string | null>(null);

  function handleNormalize(_orig: string, _normalized: string) {
    normalization =
      "WYSIWYG により表記が正規化されました（例: `*foo*` / `_foo_` の統一、リンクの展開、改行整理など）。Source モードで内容を確認できます。";
  }

  async function open() {
    error = null;
    try {
      const loaded = await pickAndReadFile();
      if (loaded) doc.load(loaded.path, loaded.text, loaded.gitAvailable);
    } catch (e) {
      error = String(e);
    }
  }

  async function save() {
    error = null;
    try {
      if (doc.path) {
        await writeFile(doc.path, doc.text);
        doc.markSaved();
      } else {
        const path = await pickAndWriteFile(doc.text);
        if (path) {
          doc.path = path;
          doc.gitAvailable = await gitIsRepo(path);
          doc.markSaved();
        }
      }
    } catch (e) {
      error = String(e);
    }
  }

  function basename(p: string | null): string {
    if (!p) return "(untitled)";
    const parts = p.split(/[\\/]/);
    return parts[parts.length - 1] || p;
  }

  $effect(() => {
    if (mode === "diff" && !doc.gitAvailable) {
      mode = "source";
    }
  });

  // Clear the WYSIWYG normalization banner whenever we leave WYSIWYG, or
  // when a new file is loaded.
  $effect(() => {
    void doc.path;
    if (mode !== "wysiwyg") normalization = null;
  });

  $effect(() => {
    function onKey(e: KeyboardEvent) {
      const mod = e.metaKey || e.ctrlKey;
      if (!mod) return;
      if (e.key === "o") {
        e.preventDefault();
        open();
      } else if (e.key === "s") {
        e.preventDefault();
        save();
      } else if (e.key === "1") {
        e.preventDefault();
        mode = "source";
      } else if (e.key === "2") {
        e.preventDefault();
        mode = "live";
      } else if (e.key === "3") {
        e.preventDefault();
        mode = "wysiwyg";
      } else if (e.key === "4") {
        e.preventDefault();
        mode = "preview";
      } else if (e.key === "5" && doc.gitAvailable) {
        e.preventDefault();
        mode = "diff";
      }
    }
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  });
</script>

<svelte:head>
  <title>{basename(doc.path)}{doc.dirty ? " •" : ""} — mdv</title>
</svelte:head>

<div class="app">
  <header>
    <div class="title">
      <span class="filename">{basename(doc.path)}</span>
      {#if doc.dirty}<span class="dirty" title="Unsaved changes">●</span>{/if}
    </div>
    <ModeBar bind:mode gitAvailable={doc.gitAvailable} />
    <div class="actions">
      <button onclick={open}>Open</button>
      <button onclick={save}>Save</button>
    </div>
  </header>

  {#if error}
    <div class="error">{error}</div>
  {/if}
  {#if normalization && mode === "wysiwyg"}
    <div class="warn">
      {normalization}
      <button class="dismiss" onclick={() => (normalization = null)}>×</button>
    </div>
  {/if}

  <main>
    {#if mode === "source"}
      <SourceView text={doc.text} onchange={(t) => doc.setText(t)} />
    {:else if mode === "live"}
      <LivePreviewView text={doc.text} onchange={(t) => doc.setText(t)} />
    {:else if mode === "wysiwyg"}
      <WysiwygView
        text={doc.text}
        onchange={(t) => doc.setText(t)}
        onnormalize={handleNormalize}
      />
    {:else if mode === "preview"}
      <PreviewView text={doc.text} />
    {:else}
      <DiffView />
    {/if}
  </main>
</div>

<style>
  :global(html),
  :global(body) {
    margin: 0;
    height: 100%;
  }
  :global(:root) {
    color-scheme: light dark;
    font-family: system-ui, -apple-system, "Segoe UI", Roboto, sans-serif;
    background: light-dark(#fff, #1a1a1a);
    color: light-dark(#222, #ddd);
  }
  .app {
    display: flex;
    flex-direction: column;
    height: 100vh;
  }
  header {
    display: flex;
    align-items: center;
    gap: 1rem;
    padding: 0.5rem 1rem;
    border-bottom: 1px solid light-dark(#ddd, #333);
    background: light-dark(#fafafa, #222);
    flex-wrap: wrap;
  }
  .title {
    display: flex;
    align-items: baseline;
    gap: 0.4rem;
    min-width: 0;
    flex: 1 1 12ch;
  }
  .filename {
    font-weight: 500;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .dirty {
    color: light-dark(#cc7000, #ffb84d);
  }
  .actions {
    display: flex;
    gap: 0.4rem;
  }
  .actions button {
    background: transparent;
    border: 1px solid light-dark(#ccc, #555);
    border-radius: 5px;
    padding: 0.35rem 0.9rem;
    font: inherit;
    color: inherit;
    cursor: pointer;
  }
  .actions button:hover {
    background: light-dark(#eee, #2a2a2a);
  }
  /* Mobile / narrow window: stack header rows, make tap targets larger. */
  @media (max-width: 640px) {
    header {
      padding: 0.4rem 0.6rem;
      gap: 0.5rem;
    }
    .title {
      flex-basis: 100%;
      order: 1;
    }
    :global(.mode-bar) {
      order: 3;
      flex: 1 1 100%;
      overflow-x: auto;
      -webkit-overflow-scrolling: touch;
    }
    .actions {
      order: 2;
    }
    .actions button {
      padding: 0.5rem 0.9rem;
    }
  }
  .error {
    padding: 0.5rem 1rem;
    background: light-dark(#fff0f0, #4a2222);
    color: light-dark(#a33, #ffb4b4);
    border-bottom: 1px solid light-dark(#fcc, #663);
    font-size: 0.85rem;
  }
  .warn {
    display: flex;
    align-items: flex-start;
    gap: 0.5rem;
    padding: 0.5rem 1rem;
    background: light-dark(#fff8e5, #3a3220);
    color: light-dark(#7a5a00, #e8c97a);
    border-bottom: 1px solid light-dark(#f0d68c, #5a4a20);
    font-size: 0.85rem;
  }
  .warn .dismiss {
    margin-left: auto;
    background: transparent;
    border: 0;
    font-size: 1.1rem;
    line-height: 1;
    color: inherit;
    cursor: pointer;
    padding: 0 0.3em;
  }
  main {
    flex: 1;
    overflow: hidden;
    min-height: 0;
  }
</style>
