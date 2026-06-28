<script lang="ts">
  import { onMount, tick } from "svelte";

  /**
   * Floating Find / Find-and-Replace bar. Style follows VSCode's editor
   * find widget: a single line with the query input and nav, and an optional
   * second line with the replace input and ↻ / ⤓ buttons. The chevron at
   * the left toggles the replace row, ⌘H opens with the row visible.
   *
   * Host owns matching: this component just renders inputs / count / nav
   * and forwards user intent via callbacks. The host applies highlights,
   * scroll, and the actual replace operation.
   */
  let {
    query = $bindable(""),
    replaceQuery = $bindable(""),
    matchCount = 0,
    currentIndex = 0,
    focusVersion = 0,
    enableReplace = false,
    replaceVisible = $bindable(false),
    onnext,
    onprev,
    onreplace,
    onreplaceAll,
    onclose,
  }: {
    query?: string;
    replaceQuery?: string;
    matchCount?: number;
    currentIndex?: number;
    /** Bump from the host to re-focus the input when ⌘F is pressed again
     * while the bar is already open. */
    focusVersion?: number;
    /** When true, surface the chevron toggle and the replace row. Hosts
     * that wrap read-only content (Preview / Diff) keep this off. */
    enableReplace?: boolean;
    /** Whether the replace row is shown. ⌘H sets this to true on open. */
    replaceVisible?: boolean;
    onnext: () => void;
    onprev: () => void;
    onreplace?: () => void;
    onreplaceAll?: () => void;
    onclose: () => void;
  } = $props();

  let findInput: HTMLInputElement;
  // Declared as $state so the auto-focus $effect below can read it without
  // tripping Svelte's "updated outside reactive context" warning. (findInput
  // is only touched in onMount and a $effect that doesn't tree-shake, so it
  // stays as a plain let.)
  let replaceInput = $state<HTMLInputElement | undefined>(undefined);

  onMount(async () => {
    await tick();
    findInput?.focus();
    findInput?.select();
  });

  $effect(() => {
    void focusVersion;
    if (focusVersion > 0) {
      findInput?.focus();
      findInput?.select();
    }
  });

  function onFindKeydown(e: KeyboardEvent) {
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
    // ⌘G / Ctrl+G — next match
    if ((e.metaKey || e.ctrlKey) && e.key.toLowerCase() === "g") {
      e.preventDefault();
      if (e.shiftKey) onprev();
      else onnext();
      return;
    }
    // ⌘H / Ctrl+H — toggle replace row (matches VSCode)
    if ((e.metaKey || e.ctrlKey) && e.key.toLowerCase() === "h" && enableReplace) {
      e.preventDefault();
      replaceVisible = !replaceVisible;
    }
  }

  function onReplaceKeydown(e: KeyboardEvent) {
    if (e.key === "Escape") {
      e.preventDefault();
      onclose();
      return;
    }
    if (e.key === "Enter") {
      e.preventDefault();
      if (e.shiftKey) onreplaceAll?.();
      else onreplace?.();
      return;
    }
    // ⌘⇧Enter — replace all, ⌘Enter — replace single (mirror VSCode)
    if ((e.metaKey || e.ctrlKey) && e.key === "Enter") {
      e.preventDefault();
      if (e.shiftKey) onreplaceAll?.();
      else onreplace?.();
    }
  }

  // Auto-focus the replace input when the row first becomes visible.
  $effect(() => {
    if (replaceVisible) {
      tick().then(() => replaceInput?.focus());
    }
  });

  const hasQuery = $derived(query.length > 0);
  const countLabel = $derived(
    !hasQuery
      ? ""
      : matchCount === 0
        ? "0 / 0"
        : `${currentIndex} / ${matchCount}`,
  );
  const noMatches = $derived(!hasQuery || matchCount === 0);
</script>

<div class="find-bar" role="search" class:two-row={enableReplace && replaceVisible}>
  <div class="row">
    {#if enableReplace}
      <button
        type="button"
        class="chevron"
        aria-label={replaceVisible ? "Hide replace" : "Show replace"}
        aria-expanded={replaceVisible}
        title={replaceVisible ? "Hide replace (⌘H)" : "Show replace (⌘H)"}
        onclick={() => (replaceVisible = !replaceVisible)}
      >
        {replaceVisible ? "▼" : "▶"}
      </button>
    {/if}
    <input
      bind:this={findInput}
      bind:value={query}
      onkeydown={onFindKeydown}
      type="text"
      spellcheck="false"
      autocomplete="off"
      placeholder="Find"
      aria-label="Find"
    />
    <span class="count" aria-live="polite">{countLabel}</span>
    <button
      type="button"
      class="nav"
      aria-label="Previous match"
      title="Previous (Shift+Enter / ⇧⌘G)"
      disabled={noMatches}
      onclick={onprev}
    >
      ↑
    </button>
    <button
      type="button"
      class="nav"
      aria-label="Next match"
      title="Next (Enter / ⌘G)"
      disabled={noMatches}
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
  {#if enableReplace && replaceVisible}
    <div class="row replace-row">
      <span class="chevron-spacer" aria-hidden="true"></span>
      <input
        bind:this={replaceInput}
        bind:value={replaceQuery}
        onkeydown={onReplaceKeydown}
        type="text"
        spellcheck="false"
        autocomplete="off"
        placeholder="Replace"
        aria-label="Replace with"
      />
      <span class="count" aria-hidden="true"></span>
      <button
        type="button"
        class="nav"
        aria-label="Replace one"
        title="Replace (Enter / ⌘Enter)"
        disabled={noMatches}
        onclick={() => onreplace?.()}
      >
        ↻
      </button>
      <button
        type="button"
        class="nav"
        aria-label="Replace all"
        title="Replace all (Shift+Enter / ⇧⌘Enter)"
        disabled={noMatches}
        onclick={() => onreplaceAll?.()}
      >
        ⤓
      </button>
      <span class="close-spacer" aria-hidden="true"></span>
    </div>
  {/if}
</div>

<style>
  .find-bar {
    position: fixed;
    top: 0.45rem;
    right: 3rem;
    z-index: 20;
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
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
  .row {
    display: inline-flex;
    align-items: center;
    gap: 0.35rem;
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
  .chevron {
    background: transparent;
    border: 0;
    padding: 0.1rem 0.25rem;
    font: inherit;
    font-size: 0.7rem;
    color: var(--mdv-text-mute);
    cursor: pointer;
    border-radius: 3px;
    line-height: 1;
    width: 1.2em;
  }
  .chevron:hover {
    background: var(--mdv-surface-hi);
    color: var(--mdv-text);
  }
  /* Vertical alignment of the replace row's controls under the find row's
     controls. The chevron / close are removed on row 2, but the input,
     count and nav buttons stay column-aligned. */
  .chevron-spacer {
    width: 1.2em;
    display: inline-block;
  }
  .close-spacer {
    width: 1.4em;
    display: inline-block;
  }
  /* Marker styles for DOM-based find (Preview / Diff / WYSIWYG). */
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
  /* CodeMirror find highlight (matches the DOM marks above). */
  :global(.cm-editor .mdv-find-hit) {
    background: light-dark(#fde68a, #4a3a10);
    border-radius: 2px;
  }
  :global(.cm-editor .mdv-find-current) {
    background: light-dark(#fbbf24, #b97c10);
    outline: 1px solid light-dark(#b45309, #fbbf24);
    border-radius: 2px;
  }
</style>
