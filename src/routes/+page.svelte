<script lang="ts">
  import { doc } from "$lib/stores/doc.svelte";
  import {
    pickAndReadFile,
    pickAndWriteFile,
    pickSavePath,
    writeBinaryFile,
    writeFile,
  } from "$lib/ipc/fs";
  import { gitIsRepo } from "$lib/ipc/git";
  import {
    printAsPdf,
    renderToDocx,
    renderToHtml,
    renderToPlainText,
  } from "$lib/export";
  import { onMount } from "svelte";
  import ModeBar from "$lib/components/ModeBar.svelte";
  import MdvExportDialog from "$lib/components/MdvExportDialog.svelte";
  import SettingsDialog from "$lib/components/SettingsDialog.svelte";
  import { settings, FONT_SIZE_PX } from "$lib/stores/settings.svelte";
  import SourceView from "$lib/views/SourceView.svelte";
  import LivePreviewView from "$lib/views/LivePreviewView.svelte";
  import WysiwygView from "$lib/views/WysiwygView.svelte";
  import PreviewView from "$lib/views/PreviewView.svelte";
  import DiffView from "$lib/views/DiffView.svelte";
  import { SAMPLE_MD } from "$lib/sample";
  import type { Mode } from "$lib/types";

  let mode = $state<Mode>("source");
  let error = $state<string | null>(null);
  let normalization = $state<string | null>(null);
  let settingsOpen = $state(false);

  onMount(() => {
    settings.hydrate();
    // Once persisted settings are in, switch to the user's preferred default
    // mode. Honor it only if no file has been loaded yet (don't surprise the
    // user mid-flow).
    if (!doc.path && !doc.text) {
      mode = settings.defaultMode;
    }
  });

  // Apply the chosen theme by toggling `color-scheme` on <html>. CSS rules
  // below resolve `light-dark()` based on that. "auto" lets the OS decide.
  $effect(() => {
    if (typeof document === "undefined") return;
    document.documentElement.dataset.theme = settings.theme;
  });

  // Editor font size is propagated via a CSS custom property so CodeMirror's
  // `.cm-editor` and the Live Preview view both pick it up.
  $effect(() => {
    if (typeof document === "undefined") return;
    const px = FONT_SIZE_PX[settings.editorFontSize];
    document.documentElement.style.setProperty("--mdv-editor-font-size", `${px}px`);
  });

  function handleNormalize(_orig: string, normalized: string) {
    normalization =
      "WYSIWYG により表記が正規化されました（例: `*foo*` / `_foo_` の統一、リンクの展開、改行整理など）。Source モードで内容を確認できます。";
    // Adopt the normalized form as the new baseline so we don't show a spurious
    // dirty mark just from opening WYSIWYG. If the user already had unsaved
    // edits, `adoptNormalized` preserves them.
    doc.adoptNormalized(normalized);
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

  function loadSample() {
    error = null;
    normalization = null;
    // No path: untitled, will Save As on first save.
    doc.load(null, SAMPLE_MD, false);
    mode = "preview";
  }

  let exportOpen = $state(false);

  function exportTitle(): string {
    return basename(doc.path).replace(/\.[^.]+$/, "") || "untitled";
  }

  async function exportHtml() {
    exportOpen = false;
    error = null;
    try {
      const path = await pickSavePath("html", "HTML", doc.path);
      if (!path) return;
      await writeFile(path, renderToHtml(doc.text, exportTitle()));
    } catch (e) {
      error = String(e);
    }
  }

  async function exportPdf() {
    exportOpen = false;
    error = null;
    try {
      await printAsPdf(doc.text, exportTitle());
    } catch (e) {
      error = String(e);
    }
  }

  async function exportPlainText() {
    exportOpen = false;
    error = null;
    try {
      const path = await pickSavePath("txt", "Plain text", doc.path);
      if (!path) return;
      await writeFile(path, renderToPlainText(doc.text));
    } catch (e) {
      error = String(e);
    }
  }

  async function exportDocx() {
    exportOpen = false;
    error = null;
    try {
      const path = await pickSavePath("docx", "Word document", doc.path);
      if (!path) return;
      const bytes = await renderToDocx(doc.text, exportTitle());
      await writeBinaryFile(path, bytes);
    } catch (e) {
      error = String(e);
    }
  }

  let mdvDialogOpen = $state(false);

  function openMdvDialog() {
    exportOpen = false;
    error = null;
    if (!doc.path) {
      error = ".mdv export requires a saved file in a Git repository";
      return;
    }
    if (!doc.gitAvailable) {
      error = "this file is not in a Git repository";
      return;
    }
    mdvDialogOpen = true;
  }

  let mdvStatus = $state<string | null>(null);

  function onMdvSaved(msg: string) {
    mdvDialogOpen = false;
    mdvStatus = msg;
    setTimeout(() => {
      if (mdvStatus === msg) mdvStatus = null;
    }, 6000);
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

  // Close the Export dropdown on any click that's not inside it.
  $effect(() => {
    if (!exportOpen) return;
    function onClick(e: MouseEvent) {
      const target = e.target as HTMLElement | null;
      if (!target?.closest(".export-dd")) {
        exportOpen = false;
      }
    }
    window.addEventListener("click", onClick);
    return () => window.removeEventListener("click", onClick);
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
      <div class="export-dd" class:open={exportOpen}>
        <button
          onclick={() => (exportOpen = !exportOpen)}
          aria-haspopup="menu"
          aria-expanded={exportOpen}
        >
          Export ▾
        </button>
        {#if exportOpen}
          <div role="menu" class="export-menu">
            <button role="menuitem" onclick={exportHtml}>HTML</button>
            <button role="menuitem" onclick={exportPdf}>PDF (via print)</button>
            <button role="menuitem" onclick={exportPlainText}>Plain text</button>
            <button role="menuitem" onclick={exportDocx}>DOCX</button>
            <button
              role="menuitem"
              onclick={openMdvDialog}
              disabled={!doc.gitAvailable}
              title={doc.gitAvailable
                ? "Bundle history into a .mdv package"
                : "Requires a Git-managed file"}
            >
              .mdv (with history)
            </button>
          </div>
        {/if}
      </div>
      <button onclick={loadSample} title="Load a built-in sample document">Sample</button>
      <button
        onclick={() => (settingsOpen = true)}
        title="Settings"
        aria-label="Settings"
        class="icon-btn"
      >
        ⚙
      </button>
    </div>
  </header>

  {#if error}
    <div class="error">{error}</div>
  {/if}
  {#if mdvStatus}
    <div class="info">
      {mdvStatus}
      <button class="dismiss" onclick={() => (mdvStatus = null)}>×</button>
    </div>
  {/if}
  {#if normalization && mode === "wysiwyg"}
    <div class="warn">
      {normalization}
      <button class="dismiss" onclick={() => (normalization = null)}>×</button>
    </div>
  {/if}
  {#if !doc.path && !doc.text}
    <div class="hint">
      Tap <strong>Sample</strong> for a quick demo, <strong>Open</strong> to pick a file, or just start typing.
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

  {#if mdvDialogOpen && doc.path}
    <MdvExportDialog
      path={doc.path}
      currentText={doc.text}
      onSaved={onMdvSaved}
      onCancel={() => (mdvDialogOpen = false)}
    />
  {/if}
  {#if settingsOpen}
    <SettingsDialog onClose={() => (settingsOpen = false)} />
  {/if}
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
    --mdv-editor-font-size: 14px;
  }
  /* Theme: when the user picks an explicit value, force the color scheme
     so `light-dark()` resolves in our favor. "auto" leaves both. */
  :global(:root[data-theme="light"]) {
    color-scheme: light;
  }
  :global(:root[data-theme="dark"]) {
    color-scheme: dark;
  }
  :global(:root[data-theme="auto"]) {
    color-scheme: light dark;
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
  .actions .icon-btn {
    padding: 0.35rem 0.55rem;
    font-size: 1.05rem;
    line-height: 1;
  }
  .export-dd {
    position: relative;
    display: inline-block;
  }
  .export-menu {
    position: absolute;
    top: calc(100% + 4px);
    right: 0;
    z-index: 10;
    background: light-dark(#fff, #1f1f1f);
    border: 1px solid light-dark(#ccc, #444);
    border-radius: 6px;
    box-shadow: 0 6px 18px rgba(0, 0, 0, 0.15);
    min-width: 11em;
    padding: 0.25rem;
    display: flex;
    flex-direction: column;
  }
  .export-menu button {
    background: transparent;
    border: 0;
    border-radius: 4px;
    padding: 0.45rem 0.7rem;
    font: inherit;
    color: inherit;
    text-align: left;
    cursor: pointer;
  }
  .export-menu button:hover {
    background: light-dark(#f0f0f0, #2a2a2a);
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
  .info {
    display: flex;
    align-items: flex-start;
    gap: 0.5rem;
    padding: 0.5rem 1rem;
    background: light-dark(#eaf6ed, #1e3023);
    color: light-dark(#1a5d2a, #8edda1);
    border-bottom: 1px solid light-dark(#bfe2c8, #2a4530);
    font-size: 0.85rem;
  }
  .info .dismiss {
    margin-left: auto;
    background: transparent;
    border: 0;
    font-size: 1.1rem;
    line-height: 1;
    color: inherit;
    cursor: pointer;
    padding: 0 0.3em;
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
  .hint {
    padding: 0.5rem 1rem;
    background: light-dark(#f0f6ff, #1a2538);
    color: light-dark(#456, #99aacc);
    border-bottom: 1px solid light-dark(#cde, #2a3a55);
    font-size: 0.85rem;
  }
  .hint strong {
    color: light-dark(#16325c, #b9d0ff);
    font-weight: 600;
  }
  main {
    flex: 1;
    overflow: hidden;
    min-height: 0;
  }
</style>
