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
    border: 1px solid light-dark(#ddd, #444);
  }
  button {
    background: transparent;
    border: 0;
    padding: 0.35rem 0.9rem;
    font: inherit;
    color: light-dark(#444, #ccc);
    cursor: pointer;
  }
  button + button {
    border-left: 1px solid light-dark(#ddd, #444);
  }
  button:hover:not(:disabled) {
    background: light-dark(#f2f2f2, #2a2a2a);
  }
  button.active {
    background: light-dark(#e3eaf5, #2b3a55);
    color: light-dark(#16325c, #b9d0ff);
  }
  button:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }
</style>
