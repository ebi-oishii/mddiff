<script lang="ts">
  import { onMount, tick } from "svelte";

  /**
   * Floating Find bar used by Preview / Diff views. CodeMirror-backed views
   * (Source / Live) get the editor's own search panel instead — this is for
   * read-only DOM views that don't have one built in.
   *
   * Host owns matching: this component just renders input + count + nav and
   * calls back. The host is responsible for highlighting and scroll.
   */
  let {
    query = $bindable(""),
    matchCount = 0,
    currentIndex = 0,
    focusVersion = 0,
    onnext,
    onprev,
    onclose,
  }: {
    query?: string;
    matchCount?: number;
    currentIndex?: number;
    /** Bump from the host to re-focus the input when ⌘F is pressed again
     * while the bar is already open. */
    focusVersion?: number;
    onnext: () => void;
    onprev: () => void;
    onclose: () => void;
  } = $props();

  let input: HTMLInputElement;

  onMount(async () => {
    await tick();
    input?.focus();
    input?.select();
  });

  $effect(() => {
    void focusVersion;
    if (focusVersion > 0) {
      input?.focus();
      input?.select();
    }
  });

  function onKeydown(e: KeyboardEvent) {
    if (e.key === "Escape") {
      e.preventDefault();
      onclose();
      return;
    }
    if (e.key === "Enter") {
      e.preventDefault();
      if (e.shiftKey) onprev();
      else onnext();
      return;
    }
    // ⌘G / Ctrl+G — next match (works even when input has focus)
    if ((e.metaKey || e.ctrlKey) && e.key.toLowerCase() === "g") {
      e.preventDefault();
      if (e.shiftKey) onprev();
      else onnext();
    }
  }

  const hasQuery = $derived(query.length > 0);
  const countLabel = $derived(
    !hasQuery
      ? ""
      : matchCount === 0
        ? "0 / 0"
        : `${currentIndex} / ${matchCount}`,
  );
</script>

<div class="find-bar" role="search">
  <input
    bind:this={input}
    bind:value={query}
    onkeydown={onKeydown}
    type="text"
    spellcheck="false"
    autocomplete="off"
    placeholder="Find"
    aria-label="Find in view"
  />
  <span class="count" aria-live="polite">{countLabel}</span>
  <button
    type="button"
    class="nav"
    aria-label="Previous match"
    title="Previous (Shift+Enter / ⇧⌘G)"
    disabled={!hasQuery || matchCount === 0}
    onclick={onprev}
  >
    ↑
  </button>
  <button
    type="button"
    class="nav"
    aria-label="Next match"
    title="Next (Enter / ⌘G)"
    disabled={!hasQuery || matchCount === 0}
    onclick={onnext}
  >
    ↓
  </button>
  <button
    type="button"
    class="close"
    aria-label="Close find"
    title="Close (Esc)"
    onclick={onclose}
  >
    ×
  </button>
</div>

<style>
  .find-bar {
    position: fixed;
    top: 0.45rem;
    right: 3rem;
    z-index: 20;
    display: inline-flex;
    align-items: center;
    gap: 0.35rem;
    padding: 0.3rem 0.45rem;
    background: var(--mdv-surface-pop);
    border: 1px solid var(--mdv-border);
    border-radius: 6px;
    box-shadow: 0 2px 8px var(--mdv-shadow);
    font-size: 0.82rem;
  }
  /* When the title overlay is visible (fullscreen), drop below it. */
  :global(:root[data-fullscreen]) .find-bar {
    top: 2.6rem;
  }
  input {
    width: 14em;
    padding: 0.15rem 0.4rem;
    font: inherit;
    background: var(--mdv-bg);
    color: var(--mdv-text);
    border: 1px solid var(--mdv-border);
    border-radius: 4px;
  }
  input:focus {
    outline: 1px solid var(--mdv-accent);
    outline-offset: 0;
  }
  .count {
    min-width: 4em;
    text-align: center;
    color: var(--mdv-text-mute);
    font-variant-numeric: tabular-nums;
  }
  .nav,
  .close {
    background: transparent;
    border: 0;
    padding: 0.1rem 0.35rem;
    font: inherit;
    color: var(--mdv-text);
    cursor: pointer;
    border-radius: 3px;
    line-height: 1;
  }
  .nav:hover:not(:disabled),
  .close:hover {
    background: var(--mdv-surface-hi);
  }
  .nav:disabled {
    opacity: 0.35;
    cursor: not-allowed;
  }
  .close {
    font-size: 1.1rem;
    margin-left: 0.1rem;
  }
  /* Marker styles for DOM-based find (Preview / Diff). CodeMirror views
     get their own highlighting from @codemirror/search. */
  :global(mark.mdv-find-hit) {
    background: light-dark(#fde68a, #4a3a10);
    color: inherit;
    padding: 0;
    border-radius: 2px;
  }
  :global(mark.mdv-find-current) {
    background: light-dark(#fbbf24, #b97c10);
    outline: 1px solid light-dark(#b45309, #fbbf24);
  }
</style>
