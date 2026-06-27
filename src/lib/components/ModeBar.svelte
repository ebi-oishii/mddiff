<script lang="ts">
  import type { Mode } from "$lib/types";

  let {
    mode = $bindable(),
    gitAvailable = false,
  }: { mode: Mode; gitAvailable?: boolean } = $props();

  const modes: { id: Mode; label: string; requiresGit?: boolean }[] = [
    { id: "source", label: "Source" },
    { id: "live", label: "Live Preview" },
    { id: "wysiwyg", label: "WYSIWYG" },
    { id: "preview", label: "Preview" },
    { id: "diff", label: "Diff", requiresGit: true },
  ];
</script>

<div class="mode-bar" role="tablist" aria-label="Editor mode">
  {#each modes as m}
    {@const disabled = m.requiresGit && !gitAvailable}
    <button
      role="tab"
      aria-selected={mode === m.id}
      class:active={mode === m.id}
      {disabled}
      title={disabled ? "File is not in a Git repository" : undefined}
      onclick={() => (mode = m.id)}
    >
      {m.label}
    </button>
  {/each}
</div>

<style>
  .mode-bar {
    display: inline-flex;
    border-radius: 6px;
    overflow: hidden;
    border: 1px solid var(--mdv-border);
  }
  button {
    background: transparent;
    border: 0;
    padding: 0.3rem 0.85rem;
    font: inherit;
    color: var(--mdv-text-mute);
    cursor: pointer;
    transition:
      background-color 0.12s,
      color 0.12s;
  }
  button + button {
    border-left: 1px solid var(--mdv-border);
  }
  button:hover:not(:disabled):not(.active) {
    background: var(--mdv-surface-hi);
    color: var(--mdv-text);
  }
  button.active {
    background: var(--mdv-accent-bg);
    color: var(--mdv-accent-fg);
  }
  button:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }
</style>
