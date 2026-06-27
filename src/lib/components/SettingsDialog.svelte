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
    background: rgba(0, 0, 0, 0.4);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 100;
    padding: 1rem;
  }
  .modal {
    background: light-dark(#fff, #1e1e1e);
    color: inherit;
    border-radius: 8px;
    padding: 1.25rem 1.5rem;
    max-width: 28em;
    width: 100%;
    box-shadow: 0 12px 40px rgba(0, 0, 0, 0.35);
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
    padding: 0.25rem 0.5rem;
    background: light-dark(#fff, #1a1a1a);
    color: inherit;
    border: 1px solid light-dark(#ccc, #444);
    border-radius: 4px;
    min-width: 12em;
  }
  .hint {
    margin: 1rem 0 0;
    font-size: 0.8rem;
    color: light-dark(#777, #888);
  }
  .actions {
    display: flex;
    justify-content: space-between;
    gap: 0.5rem;
    margin-top: 1rem;
  }
  .actions button {
    background: transparent;
    border: 1px solid light-dark(#ccc, #555);
    border-radius: 5px;
    padding: 0.4rem 1rem;
    font: inherit;
    color: inherit;
    cursor: pointer;
  }
  .actions button:hover {
    background: light-dark(#eee, #2a2a2a);
  }
  .actions button.primary {
    background: light-dark(#16325c, #2b3a55);
    color: #fff;
    border-color: transparent;
  }
  .actions button.primary:hover {
    background: light-dark(#0d2440, #3a4a6b);
  }
  .actions button.link {
    border: 0;
    color: light-dark(#0969da, #58a6ff);
    padding: 0.4rem 0;
  }
  .actions button.link:hover {
    background: transparent;
    text-decoration: underline;
  }
</style>
