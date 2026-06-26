<script lang="ts">
  import type { Mode } from "$lib/types";

  let { mode = $bindable() }: { mode: Mode } = $props();

  const modes: Mode[] = ["source", "preview"];
</script>

<div class="mode-bar" role="tablist" aria-label="Editor mode">
  {#each modes as m}
    <button
      role="tab"
      aria-selected={mode === m}
      class:active={mode === m}
      onclick={() => (mode = m)}
    >
      {m === "source" ? "Source" : "Preview"}
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
  button:hover {
    background: light-dark(#f2f2f2, #2a2a2a);
  }
  button.active {
    background: light-dark(#e3eaf5, #2b3a55);
    color: light-dark(#16325c, #b9d0ff);
  }
</style>
