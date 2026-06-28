<script lang="ts">
  import { settings } from "$lib/stores/settings.svelte";
  import type {
    EditorTheme,
    FontSize,
    TabWidth,
    Theme,
  } from "$lib/stores/settings.svelte";
  import type { DiffSubmode, Mode } from "$lib/types";

  let { onClose }: { onClose: () => void } = $props();

  function persistChange<K extends keyof typeof settings>(key: K, value: unknown) {
    // @ts-expect-error narrow on key — store fields are heterogeneous
    settings[key] = value;
    settings.persist();
  }
</script>

<div class="overlay" role="presentation" onclick={onClose}>
  <div
    class="modal"
    role="dialog"
    tabindex="-1"
    aria-modal="true"
    aria-labelledby="settings-title"
    onclick={(e) => e.stopPropagation()}
    onkeydown={(e) => e.key === "Escape" && onClose()}
  >
    <h2 id="settings-title">Settings</h2>

    <div class="section-title">Appearance</div>

    <div class="row">
      <label for="theme">Theme</label>
      <select
        id="theme"
        value={settings.theme}
        onchange={(e) =>
          persistChange("theme", (e.currentTarget as HTMLSelectElement).value as Theme)}
      >
        <option value="auto">Auto (follow OS)</option>
        <option value="light">Light</option>
        <option value="dark">Dark</option>
      </select>
    </div>

    <div class="row">
      <label for="fontsize">Editor font size</label>
      <select
        id="fontsize"
        value={settings.editorFontSize}
        onchange={(e) =>
          persistChange(
            "editorFontSize",
            (e.currentTarget as HTMLSelectElement).value as FontSize,
          )}
      >
        <option value="small">Small (12 px)</option>
        <option value="medium">Medium (14 px)</option>
        <option value="large">Large (17 px)</option>
      </select>
    </div>

    <div class="row">
      <label for="defmode">Default mode on open</label>
      <select
        id="defmode"
        value={settings.defaultMode}
        onchange={(e) =>
          persistChange("defaultMode", (e.currentTarget as HTMLSelectElement).value as Mode)}
      >
        <option value="source">Source</option>
        <option value="live">Live Preview</option>
        <option value="wysiwyg">WYSIWYG</option>
        <option value="preview">Preview</option>
        <option value="diff">Diff (when Git available)</option>
      </select>
    </div>

    <div class="row">
      <label for="editortheme">Editor syntax theme</label>
      <select
        id="editortheme"
        value={settings.editorTheme}
        onchange={(e) =>
          persistChange(
            "editorTheme",
            (e.currentTarget as HTMLSelectElement).value as EditorTheme,
          )}
      >
        <option value="github">GitHub</option>
        <option value="solarized">Solarized</option>
        <option value="dracula">Dracula</option>
      </select>
    </div>

    <div class="section-title">File</div>

    <div class="row">
      <label for="autoreload">Auto-reload on external change</label>
      <input
        id="autoreload"
        type="checkbox"
        checked={settings.autoReload}
        onchange={(e) =>
          persistChange("autoReload", (e.currentTarget as HTMLInputElement).checked)}
      />
    </div>
    <p class="row-hint">
      When the file changes on disk and you have no unsaved edits, swap in the
      disk content silently. Off = always confirm via banner.
    </p>

    <div class="section-title">Source view</div>

    <div class="row">
      <label for="softwrap">Soft wrap</label>
      <input
        id="softwrap"
        type="checkbox"
        checked={settings.softWrap}
        onchange={(e) =>
          persistChange("softWrap", (e.currentTarget as HTMLInputElement).checked)}
      />
    </div>

    <div class="row">
      <label for="linenumbers">Line numbers</label>
      <input
        id="linenumbers"
        type="checkbox"
        checked={settings.lineNumbers}
        onchange={(e) =>
          persistChange("lineNumbers", (e.currentTarget as HTMLInputElement).checked)}
      />
    </div>

    <div class="row">
      <label for="tabwidth">Tab width</label>
      <select
        id="tabwidth"
        value={String(settings.tabWidth)}
        onchange={(e) =>
          persistChange(
            "tabWidth",
            Number((e.currentTarget as HTMLSelectElement).value) as TabWidth,
          )}
      >
        <option value="2">2 spaces</option>
        <option value="4">4 spaces</option>
        <option value="8">8 spaces</option>
      </select>
    </div>

    <div class="section-title">Diff view</div>

    <div class="row">
      <label for="diffsubmode">Default sub-mode</label>
      <select
        id="diffsubmode"
        value={settings.diffDefaultSubmode}
        onchange={(e) =>
          persistChange(
            "diffDefaultSubmode",
            (e.currentTarget as HTMLSelectElement).value as DiffSubmode,
          )}
      >
        <option value="highlight">Highlight Only</option>
        <option value="full">Full</option>
        <option value="sidebyside">Side-by-Side</option>
      </select>
    </div>

    <div class="row">
      <label for="diffdebounce">Recompute delay</label>
      <select
        id="diffdebounce"
        value={String(settings.diffDebounceMs)}
        onchange={(e) =>
          persistChange(
            "diffDebounceMs",
            Number((e.currentTarget as HTMLSelectElement).value),
          )}
      >
        <option value="100">100 ms (snappy)</option>
        <option value="250">250 ms (default)</option>
        <option value="500">500 ms (light CPU)</option>
        <option value="1000">1000 ms (large files)</option>
      </select>
    </div>

    <p class="hint">
      Settings are saved instantly to local storage. Restoring defaults clears them.
    </p>

    <div class="actions">
      <button type="button" class="link" onclick={() => settings.reset()}>Restore defaults</button>
      <button type="button" class="primary" onclick={onClose}>Done</button>
    </div>
  </div>
</div>

<style>
  .overlay {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.45);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 100;
    padding: 1rem;
  }
  .modal {
    background: var(--mdv-surface-pop);
    color: var(--mdv-text);
    border: 1px solid var(--mdv-border);
    border-radius: 8px;
    padding: 1.25rem 1.5rem;
    max-width: 28em;
    width: 100%;
    box-shadow: 0 12px 40px var(--mdv-shadow);
    max-height: calc(100vh - 2rem);
    overflow-y: auto;
  }
  h2 {
    margin: 0 0 1rem;
    font-size: 1.1rem;
  }
  .section-title {
    margin: 1rem 0 0.4rem;
    font-size: 0.72rem;
    text-transform: uppercase;
    letter-spacing: 0.06em;
    color: var(--mdv-text-mute);
    border-top: 1px solid var(--mdv-border-mute);
    padding-top: 0.7rem;
  }
  .section-title:first-of-type {
    border-top: 0;
    padding-top: 0;
    margin-top: 0;
  }
  .row {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    margin: 0.5rem 0;
    font-size: 0.9rem;
  }
  .row label {
    flex: 1;
  }
  .row select {
    font: inherit;
    padding: 0.3rem 0.5rem;
    background: var(--mdv-bg);
    color: var(--mdv-text);
    border: 1px solid var(--mdv-border);
    border-radius: 4px;
    min-width: 12em;
  }
  .row input[type="checkbox"] {
    width: 1rem;
    height: 1rem;
    margin: 0;
  }
  .hint {
    margin: 1rem 0 0;
    font-size: 0.8rem;
    color: var(--mdv-text-mute);
  }
  .row-hint {
    margin: -0.3rem 0 0.6rem;
    font-size: 0.78rem;
    color: var(--mdv-text-mute);
    line-height: 1.4;
  }
  .actions {
    display: flex;
    justify-content: space-between;
    gap: 0.5rem;
    margin-top: 1rem;
  }
  .actions button {
    background: transparent;
    border: 1px solid var(--mdv-border);
    border-radius: 5px;
    padding: 0.4rem 1rem;
    font: inherit;
    color: var(--mdv-text);
    cursor: pointer;
  }
  .actions button:hover {
    background: var(--mdv-surface-hi);
  }
  .actions button.primary {
    background: var(--mdv-accent);
    color: #fff;
    border-color: transparent;
  }
  .actions button.primary:hover {
    filter: brightness(0.92);
  }
  .actions button.link {
    border: 0;
    color: var(--mdv-accent);
    padding: 0.4rem 0;
  }
  .actions button.link:hover {
    background: transparent;
    text-decoration: underline;
  }
</style>
