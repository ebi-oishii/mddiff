<script lang="ts">
  import { settings } from "$lib/stores/settings.svelte";
  import type { FontSize, Theme } from "$lib/stores/settings.svelte";
  import type { Mode } from "$lib/types";

  let { onClose }: { onClose: () => void } = $props();

  function update<K extends "theme" | "editorFontSize" | "defaultMode">(
    key: K,
    value: Theme | FontSize | Mode,
  ) {
    // @ts-expect-error narrow on key
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

    <div class="row">
      <label for="theme">Theme</label>
      <select
        id="theme"
        value={settings.theme}
        onchange={(e) => update("theme", (e.currentTarget as HTMLSelectElement).value as Theme)}
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
          update("editorFontSize", (e.currentTarget as HTMLSelectElement).value as FontSize)}
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
        onchange={(e) => update("defaultMode", (e.currentTarget as HTMLSelectElement).value as Mode)}
      >
        <option value="source">Source</option>
        <option value="live">Live Preview</option>
        <option value="wysiwyg">WYSIWYG</option>
        <option value="preview">Preview</option>
        <option value="diff">Diff (when Git available)</option>
      </select>
    </div>

    <div class="row">
      <label for="autoreload">Auto-reload on external change</label>
      <input
        id="autoreload"
        type="checkbox"
        checked={settings.autoReload}
        onchange={(e) => {
          settings.autoReload = (e.currentTarget as HTMLInputElement).checked;
          settings.persist();
        }}
      />
    </div>
    <p class="row-hint">
      When the file changes on disk and you have no unsaved edits, swap in the
      disk content silently. Off = always confirm via banner.
    </p>

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
  }
  h2 {
    margin: 0 0 1rem;
    font-size: 1.1rem;
  }
  .row {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    margin: 0.6rem 0;
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
  .row input[type="checkbox"] {
    width: 1rem;
    height: 1rem;
    margin: 0;
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
