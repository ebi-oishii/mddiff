<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { listen, type UnlistenFn } from "@tauri-apps/api/event";
  import { getCurrentWindow } from "@tauri-apps/api/window";
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
  let menuOpen = $state(false);
  let mdvDialogOpen = $state(false);
  let mdvStatus = $state<string | null>(null);

  // Detect Mac at runtime for shortcut hint glyphs. Non-Mac users see "Ctrl+"
  // instead of ⌘ so the menu hint actually matches the key they need to press.
  const isMac =
    typeof navigator !== "undefined" && /mac/i.test(navigator.platform);
  const MOD = isMac ? "⌘" : "Ctrl+";
  const SHIFT = isMac ? "⇧" : "Shift+";

  let menuUnlisten: UnlistenFn | null = null;

  onMount(async () => {
    settings.hydrate();
    if (!doc.path && !doc.text) {
      mode = settings.defaultMode;
    }
    // Native OS menu (desktop only). The Rust side emits a `menu-event`
    // with the item id; route it back into the existing handlers so the
    // top menu and the in-app ☰ menu share behavior.
    try {
      menuUnlisten = await listen<string>("menu-event", (e) =>
        handleMenuEvent(e.payload),
      );
    } catch {
      // listen unavailable (e.g. browser / SSR) — fall back to in-app menu only
    }
  });

  onDestroy(() => {
    menuUnlisten?.();
  });

  function handleMenuEvent(id: string) {
    switch (id) {
      case "open":
        open();
        break;
      case "save":
        save();
        break;
      case "save_as":
        saveAs();
        break;
      case "sample":
        loadSample();
        break;
      case "export_html":
        exportHtml();
        break;
      case "export_pdf":
        exportPdf();
        break;
      case "export_text":
        exportPlainText();
        break;
      case "export_docx":
        exportDocx();
        break;
      case "export_mdv":
        openMdvDialog();
        break;
      case "preferences":
        openSettings();
        break;
      case "mode_source":
        mode = "source";
        break;
      case "mode_live":
        mode = "live";
        break;
      case "mode_wysiwyg":
        mode = "wysiwyg";
        break;
      case "mode_preview":
        mode = "preview";
        break;
      case "mode_diff":
        if (doc.gitAvailable) mode = "diff";
        break;
    }
  }

  $effect(() => {
    if (typeof document === "undefined") return;
    document.documentElement.dataset.theme = settings.theme;
  });

  $effect(() => {
    if (typeof document === "undefined") return;
    const px = FONT_SIZE_PX[settings.editorFontSize];
    document.documentElement.style.setProperty("--mdv-editor-font-size", `${px}px`);
  });

  // Push filename + dirty + mode into the OS window title bar (Mac top bar,
  // Win/Linux window chrome). Quiet failure when not running under Tauri.
  $effect(() => {
    void doc.path;
    void doc.dirty;
    void mode;
    const title = `${basename(doc.path)}${doc.dirty ? " ●" : ""} · ${modeLabel(mode)}`;
    try {
      getCurrentWindow().setTitle(title);
    } catch {
      // not in a Tauri window (browser / SSR); nothing to do
    }
  });

  function handleNormalize(_orig: string, normalized: string) {
    normalization =
      "WYSIWYG により表記が正規化されました（例: `*foo*` / `_foo_` の統一、リンクの展開、改行整理など）。Source モードで内容を確認できます。";
    doc.adoptNormalized(normalized);
  }

  async function open() {
    closeMenu();
    error = null;
    try {
      const loaded = await pickAndReadFile();
      if (loaded) doc.load(loaded.path, loaded.text, loaded.gitAvailable);
    } catch (e) {
      error = String(e);
    }
  }

  async function save() {
    closeMenu();
    error = null;
    try {
      if (doc.path) {
        await writeFile(doc.path, doc.text);
        doc.markSaved();
      } else {
        await saveAs();
      }
    } catch (e) {
      error = String(e);
    }
  }

  async function saveAs() {
    closeMenu();
    error = null;
    try {
      const path = await pickAndWriteFile(doc.text);
      if (path) {
        doc.path = path;
        doc.gitAvailable = await gitIsRepo(path);
        doc.markSaved();
      }
    } catch (e) {
      error = String(e);
    }
  }

  function loadSample() {
    closeMenu();
    error = null;
    normalization = null;
    doc.load(null, SAMPLE_MD, false);
    mode = "preview";
  }

  function exportTitle(): string {
    return basename(doc.path).replace(/\.[^.]+$/, "") || "untitled";
  }

  async function exportHtml() {
    closeMenu();
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
    closeMenu();
    error = null;
    try {
      await printAsPdf(doc.text, exportTitle());
    } catch (e) {
      error = String(e);
    }
  }

  async function exportPlainText() {
    closeMenu();
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
    closeMenu();
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

  function openMdvDialog() {
    closeMenu();
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

  function onMdvSaved(msg: string) {
    mdvDialogOpen = false;
    mdvStatus = msg;
    setTimeout(() => {
      if (mdvStatus === msg) mdvStatus = null;
    }, 6000);
  }

  function openSettings() {
    closeMenu();
    settingsOpen = true;
  }

  function closeMenu() {
    menuOpen = false;
  }

  type ModeEntry = { id: Mode; label: string; key: string; requiresGit?: boolean };
  const MODE_ENTRIES: ModeEntry[] = [
    { id: "source", label: "Source", key: "1" },
    { id: "live", label: "Live Preview", key: "2" },
    { id: "wysiwyg", label: "WYSIWYG", key: "3" },
    { id: "preview", label: "Preview", key: "4" },
    { id: "diff", label: "Diff", key: "5", requiresGit: true },
  ];

  function modeLabel(m: Mode): string {
    return MODE_ENTRIES.find((e) => e.id === m)?.label ?? m;
  }

  function setMode(m: Mode) {
    closeMenu();
    mode = m;
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

  $effect(() => {
    void doc.path;
    if (mode !== "wysiwyg") normalization = null;
  });

  // Close the popover on outside click / Escape.
  $effect(() => {
    if (!menuOpen) return;
    function onClick(e: MouseEvent) {
      const target = e.target as HTMLElement | null;
      if (!target?.closest(".menu-wrap")) closeMenu();
    }
    function onKey(e: KeyboardEvent) {
      if (e.key === "Escape") closeMenu();
    }
    window.addEventListener("click", onClick);
    window.addEventListener("keydown", onKey);
    return () => {
      window.removeEventListener("click", onClick);
      window.removeEventListener("keydown", onKey);
    };
  });

  $effect(() => {
    function onKey(e: KeyboardEvent) {
      const mod = e.metaKey || e.ctrlKey;
      if (!mod) return;
      // Mode shortcuts
      if (e.key === "1") {
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
      // File ops
      else if (e.key === "o") {
        e.preventDefault();
        open();
      } else if (e.key === "s" && e.shiftKey) {
        e.preventDefault();
        saveAs();
      } else if (e.key === "s") {
        e.preventDefault();
        save();
      } else if (e.key === ",") {
        e.preventDefault();
        openSettings();
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
    <!-- Filename / dirty / current mode live in the OS window title bar
         (see the setTitle effect). The in-app header is just chrome for
         the menu trigger now. -->
    <div class="header-spacer"></div>
    <div class="menu-wrap" class:open={menuOpen}>
      <button
        class="menu-trigger"
        onclick={() => (menuOpen = !menuOpen)}
        aria-haspopup="menu"
        aria-expanded={menuOpen}
        aria-label="Menu"
        title="Menu"
      >
        ☰
      </button>
      {#if menuOpen}
        <div role="menu" class="menu">
          <div class="section">Mode</div>
          {#each MODE_ENTRIES as m}
            {@const disabled = m.requiresGit && !doc.gitAvailable}
            <button
              role="menuitem"
              class="mode-item"
              class:active={mode === m.id}
              {disabled}
              onclick={() => setMode(m.id)}
              title={disabled ? "Requires a Git-managed file" : undefined}
            >
              <span>
                <span class="check" aria-hidden="true">{mode === m.id ? "✓" : ""}</span>
                {m.label}
              </span>
              <kbd>{MOD}{m.key}</kbd>
            </button>
          {/each}
          <div class="sep"></div>
          <button role="menuitem" onclick={open}>
            <span>Open…</span><kbd>{MOD}O</kbd>
          </button>
          <button role="menuitem" onclick={save}>
            <span>Save</span><kbd>{MOD}S</kbd>
          </button>
          <button role="menuitem" onclick={saveAs}>
            <span>Save As…</span><kbd>{MOD}{SHIFT}S</kbd>
          </button>
          <div class="sep"></div>
          <div class="section">Export as</div>
          <button role="menuitem" onclick={exportHtml}>HTML</button>
          <button role="menuitem" onclick={exportPdf}>PDF <span class="muted">(via print)</span></button>
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
            .mdv <span class="muted">(with history)</span>
          </button>
          <div class="sep"></div>
          <button role="menuitem" onclick={loadSample}>Load sample</button>
          <button role="menuitem" onclick={openSettings}>
            <span>Settings…</span><kbd>{MOD},</kbd>
          </button>
        </div>
      {/if}
    </div>
  </header>

  {#if error}
    <div class="banner error">
      <span>{error}</span>
      <button class="dismiss" aria-label="Dismiss" onclick={() => (error = null)}>×</button>
    </div>
  {/if}
  {#if mdvStatus}
    <div class="banner info">
      <span>{mdvStatus}</span>
      <button class="dismiss" aria-label="Dismiss" onclick={() => (mdvStatus = null)}>×</button>
    </div>
  {/if}
  {#if normalization && mode === "wysiwyg"}
    <div class="banner warn">
      <span>{normalization}</span>
      <button class="dismiss" aria-label="Dismiss" onclick={() => (normalization = null)}>×</button>
    </div>
  {/if}
  {#if !doc.path && !doc.text}
    <div class="banner hint">
      Press <kbd>{MOD}O</kbd> to open a file, or use the <strong>☰</strong> menu for Sample and other actions.
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

  /* ---------- Design tokens ---------- */
  :global(:root) {
    color-scheme: light dark;
    font-family: system-ui, -apple-system, "Segoe UI", Roboto, "Hiragino Sans", "Yu Gothic", sans-serif;

    /* Surfaces — dark mode unified on terminal-clear dark (#1e1e1e) so the
       canvas, editor and all view backgrounds stay the same as the user
       switches between modes. Header / popovers / hover use a slightly
       lighter step for chrome separation, never the canvas going dark
       further. */
    --mdv-bg:           light-dark(#ffffff, #1e1e1e);
    --mdv-surface:      light-dark(#f6f8fa, #252526);
    --mdv-surface-hi:   light-dark(#eaeef2, #2d2d2e);
    --mdv-surface-pop:  light-dark(#ffffff, #2a2a2b);
    --mdv-editor-bg:    light-dark(#ffffff, #1e1e1e);
    --mdv-editor-gutter:light-dark(#f6f8fa, #252526);

    /* Text */
    --mdv-text:         light-dark(#1f2328, #d4d4d4);
    --mdv-text-mute:    light-dark(#656d76, #9d9d9d);
    --mdv-text-subtle:  light-dark(#8c959f, #6e6e6e);

    /* Borders */
    --mdv-border:       light-dark(#d0d7de, #3c3c3c);
    --mdv-border-mute:  light-dark(#eaeef2, #2d2d2d);

    /* Accent */
    --mdv-accent:       light-dark(#0969da, #58a6ff);
    --mdv-accent-bg:    light-dark(#ddf4ff, #1f3551);
    --mdv-accent-fg:    light-dark(#0a3069, #b9d4ff);

    /* Status colors */
    --mdv-warn-fg:      light-dark(#9a6700, #d4a72c);
    --mdv-warn-bg:      light-dark(#fff8c5, #2c241a);
    --mdv-warn-border:  light-dark(#f0d68c, #3d3214);
    --mdv-danger-fg:    light-dark(#cf222e, #f85149);
    --mdv-danger-bg:    light-dark(#ffebe9, #2c1a1a);
    --mdv-danger-border:light-dark(#f8b4ad, #3d2020);
    --mdv-success-fg:   light-dark(#1a7f37, #3fb950);
    --mdv-success-bg:   light-dark(#dafbe1, #1a2e1f);
    --mdv-success-border:light-dark(#a4d9b1, #2a4530);
    --mdv-info-fg:      light-dark(#16325c, #b9d4ff);
    --mdv-info-bg:      light-dark(#ddf4ff, #1a2538);
    --mdv-info-border:  light-dark(#bcd8fa, #2a3a55);

    --mdv-shadow:       light-dark(rgba(0, 0, 0, 0.1), rgba(0, 0, 0, 0.5));

    --mdv-editor-font-size: 14px;

    background: var(--mdv-bg);
    color: var(--mdv-text);
  }
  :global(:root[data-theme="light"]) {
    color-scheme: light;
  }
  :global(:root[data-theme="dark"]) {
    color-scheme: dark;
  }
  :global(:root[data-theme="auto"]) {
    color-scheme: light dark;
  }

  /* ---------- Shell ---------- */
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
    border-bottom: 1px solid var(--mdv-border);
    background: var(--mdv-surface);
    flex-wrap: wrap;
  }
  .header-spacer {
    flex: 1;
  }

  /* ---------- Menu ---------- */
  .menu-wrap {
    position: relative;
    display: inline-block;
  }
  .menu-trigger {
    background: transparent;
    border: 1px solid transparent;
    border-radius: 5px;
    padding: 0.25rem 0.55rem;
    font: inherit;
    font-size: 1.1rem;
    line-height: 1;
    color: var(--mdv-text);
    cursor: pointer;
  }
  .menu-trigger:hover,
  .menu-wrap.open .menu-trigger {
    background: var(--mdv-surface-hi);
    border-color: var(--mdv-border);
  }
  .menu {
    position: absolute;
    top: calc(100% + 6px);
    right: 0;
    z-index: 50;
    background: var(--mdv-surface-pop);
    border: 1px solid var(--mdv-border);
    border-radius: 6px;
    box-shadow: 0 8px 24px var(--mdv-shadow);
    min-width: 16em;
    padding: 0.25rem;
    display: flex;
    flex-direction: column;
    font-size: 0.88rem;
  }
  .menu button {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 1rem;
    width: 100%;
    padding: 0.4rem 0.7rem;
    text-align: left;
    background: transparent;
    border: 0;
    border-radius: 4px;
    color: var(--mdv-text);
    font: inherit;
    cursor: pointer;
  }
  .menu button:hover:not(:disabled) {
    background: var(--mdv-surface-hi);
  }
  .menu button:disabled {
    opacity: 0.45;
    cursor: not-allowed;
  }
  .menu .section {
    padding: 0.4rem 0.7rem 0.15rem;
    font-size: 0.7rem;
    color: var(--mdv-text-mute);
    text-transform: uppercase;
    letter-spacing: 0.06em;
  }
  .menu .sep {
    height: 1px;
    background: var(--mdv-border-mute);
    margin: 0.25rem 0;
  }
  .menu .muted {
    color: var(--mdv-text-mute);
    font-size: 0.88em;
  }
  .menu .mode-item .check {
    display: inline-block;
    width: 1em;
    margin-right: 0.25em;
    color: var(--mdv-accent);
  }
  .menu .mode-item.active {
    color: var(--mdv-accent-fg);
  }
  .menu kbd,
  .banner kbd {
    font: inherit;
    font-size: 0.76em;
    color: var(--mdv-text-mute);
    background: var(--mdv-surface-hi);
    padding: 0.05em 0.4em;
    border-radius: 3px;
    border: 1px solid var(--mdv-border-mute);
  }

  /* ---------- Banners ---------- */
  .banner {
    display: flex;
    align-items: flex-start;
    gap: 0.5rem;
    padding: 0.5rem 1rem;
    border-bottom: 1px solid transparent;
    font-size: 0.85rem;
  }
  .banner.error {
    background: var(--mdv-danger-bg);
    color: var(--mdv-danger-fg);
    border-bottom-color: var(--mdv-danger-border);
  }
  .banner.warn {
    background: var(--mdv-warn-bg);
    color: var(--mdv-warn-fg);
    border-bottom-color: var(--mdv-warn-border);
  }
  .banner.info {
    background: var(--mdv-success-bg);
    color: var(--mdv-success-fg);
    border-bottom-color: var(--mdv-success-border);
  }
  .banner.hint {
    background: var(--mdv-info-bg);
    color: var(--mdv-info-fg);
    border-bottom-color: var(--mdv-info-border);
  }
  .banner.hint strong {
    color: var(--mdv-accent-fg);
  }
  .banner .dismiss {
    margin-left: auto;
    background: transparent;
    border: 0;
    font-size: 1.1rem;
    line-height: 1;
    color: inherit;
    cursor: pointer;
    padding: 0 0.3em;
  }

  /* ---------- Mobile ---------- */
  @media (max-width: 640px) {
    header {
      padding: 0.4rem 0.6rem;
      gap: 0.5rem;
    }
    .menu-trigger {
      padding: 0.45rem 0.7rem;
    }
  }

  main {
    flex: 1;
    overflow: hidden;
    min-height: 0;
    /* Anchor the canvas color here so every mode (Preview / WYSIWYG / Diff /
       editors) sits on the same background. Editors override via CM theme
       but use the same token, so they read as one continuous surface. */
    background: var(--mdv-bg);
    color: var(--mdv-text);
  }
</style>
