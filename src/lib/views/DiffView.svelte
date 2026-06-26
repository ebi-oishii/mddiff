<script lang="ts">
  import { doc } from "$lib/stores/doc.svelte";
  import { gitFullDiff, gitHunks, gitListBases } from "$lib/ipc/git";
  import type {
    BaseKind,
    BaseOption,
    DiffLine,
    DiffSubmode,
    HunkSummary,
  } from "$lib/types";
  import HighlightView from "./diff/HighlightView.svelte";
  import FullDiffView from "./diff/FullDiffView.svelte";

  const CUSTOM = "__custom__";

  let submode = $state<DiffSubmode>("highlight");
  let bases = $state<BaseOption[]>([]);
  let selected = $state<string>("HEAD");
  let customBase = $state("HEAD");
  let hunks = $state<HunkSummary[]>([]);
  let lines = $state<DiffLine[]>([]);
  let error = $state<string | null>(null);
  let loading = $state(false);

  let diffTimer: ReturnType<typeof setTimeout> | null = null;

  const isCustom = $derived(selected === CUSTOM);
  const base = $derived(isCustom ? customBase.trim() || "HEAD" : selected);

  $effect(() => {
    if (!doc.path || !doc.gitAvailable) return;
    gitListBases(doc.path).then((b) => {
      bases = b;
    });
  });

  $effect(() => {
    void doc.path;
    void doc.text;
    void submode;
    void base;

    if (!doc.path || !doc.gitAvailable) return;
    if (diffTimer) clearTimeout(diffTimer);
    diffTimer = setTimeout(load, 250);
    return () => {
      if (diffTimer) clearTimeout(diffTimer);
    };
  });

  async function load() {
    if (!doc.path) return;
    loading = true;
    error = null;
    try {
      if (submode === "highlight") {
        hunks = await gitHunks(doc.path, doc.text, base);
      } else {
        lines = await gitFullDiff(doc.path, doc.text, base);
      }
    } catch (e) {
      error = String(e);
    } finally {
      loading = false;
    }
  }

  function byKind(kind: BaseKind): BaseOption[] {
    return bases.filter((b) => b.kind === kind);
  }

  function applyCustom(e: SubmitEvent) {
    e.preventDefault();
    customBase = customBase.trim() || "HEAD";
  }

  const addedCount = $derived(
    hunks.reduce(
      (n, h) =>
        n + (h.kind === "added" || h.kind === "modified" ? h.end_line - h.start_line + 1 : 0),
      0,
    ),
  );
  const removedCount = $derived(hunks.reduce((n, h) => n + h.removed_count, 0));
</script>

<div class="diff-view">
  <div class="submode-bar">
    <div class="left">
      <div class="submode-toggle" role="tablist">
        <button
          role="tab"
          aria-selected={submode === "highlight"}
          class:active={submode === "highlight"}
          onclick={() => (submode = "highlight")}
        >
          Highlight Only
        </button>
        <button
          role="tab"
          aria-selected={submode === "full"}
          class:active={submode === "full"}
          onclick={() => (submode = "full")}
        >
          Full
        </button>
      </div>
      <label class="base-select">
        <span class="prefix">vs</span>
        <select bind:value={selected} aria-label="Compare base revision">
          <optgroup label="Special">
            {#each byKind("special") as b}
              <option value={b.revspec} title={b.detail ?? undefined}>{b.label}</option>
            {/each}
          </optgroup>
          {#if byKind("branch").length > 0}
            <optgroup label="Branches">
              {#each byKind("branch") as b}
                <option value={b.revspec}>{b.label}</option>
              {/each}
            </optgroup>
          {/if}
          {#if byKind("tag").length > 0}
            <optgroup label="Tags">
              {#each byKind("tag") as b}
                <option value={b.revspec}>{b.label}</option>
              {/each}
            </optgroup>
          {/if}
          {#if byKind("commit").length > 0}
            <optgroup label="Recent commits">
              {#each byKind("commit") as b}
                <option value={b.revspec}>{b.label}</option>
              {/each}
            </optgroup>
          {/if}
          <option value={CUSTOM}>Custom…</option>
        </select>
      </label>
      {#if isCustom}
        <form class="custom-form" onsubmit={applyCustom}>
          <input
            type="text"
            bind:value={customBase}
            spellcheck="false"
            autocomplete="off"
            placeholder="revspec"
            aria-label="Custom revision"
          />
          <button type="submit">Apply</button>
        </form>
      {/if}
    </div>
    <div class="meta">
      {#if loading}<span class="loading">…</span>{/if}
      {#if submode === "highlight" && !loading && !error}
        <span class="added">+{addedCount}</span>
        <span class="removed">−{removedCount}</span>
      {/if}
    </div>
  </div>

  {#if !doc.path}
    <div class="empty">No file open.</div>
  {:else if !doc.gitAvailable}
    <div class="empty">This file is not in a Git repository.</div>
  {:else if error}
    <div class="error">{error}</div>
  {:else if submode === "highlight"}
    <HighlightView text={doc.text} {hunks} />
  {:else}
    <FullDiffView {lines} />
  {/if}
</div>

<style>
  .diff-view {
    display: flex;
    flex-direction: column;
    height: 100%;
    min-height: 0;
  }
  .submode-bar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 1rem;
    padding: 0.5rem 1rem;
    border-bottom: 1px solid light-dark(#ddd, #333);
    background: light-dark(#fafafa, #222);
    flex-shrink: 0;
    font-size: 0.85rem;
  }
  .left {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    flex-wrap: wrap;
  }
  .submode-toggle {
    display: inline-flex;
    border: 1px solid light-dark(#ddd, #444);
    border-radius: 5px;
    overflow: hidden;
  }
  .submode-toggle button {
    background: transparent;
    border: 0;
    padding: 0.25rem 0.7rem;
    font: inherit;
    color: light-dark(#444, #ccc);
    cursor: pointer;
  }
  .submode-toggle button + button {
    border-left: 1px solid light-dark(#ddd, #444);
  }
  .submode-toggle button.active {
    background: light-dark(#e3eaf5, #2b3a55);
    color: light-dark(#16325c, #b9d0ff);
  }
  .base-select {
    display: inline-flex;
    align-items: center;
    gap: 0.4rem;
    color: light-dark(#666, #999);
  }
  .base-select .prefix {
    user-select: none;
  }
  .base-select select {
    font: inherit;
    background: light-dark(#fff, #1a1a1a);
    color: inherit;
    border: 1px solid light-dark(#ccc, #444);
    border-radius: 4px;
    padding: 0.18rem 0.4rem;
    max-width: 24em;
  }
  .custom-form {
    display: inline-flex;
    gap: 0.4rem;
  }
  .custom-form input {
    width: 12em;
    padding: 0.2rem 0.5rem;
    font: inherit;
    font-family: ui-monospace, "SF Mono", Menlo, monospace;
    background: light-dark(#fff, #1a1a1a);
    color: inherit;
    border: 1px solid light-dark(#ccc, #444);
    border-radius: 4px;
  }
  .custom-form button {
    background: transparent;
    border: 1px solid light-dark(#ccc, #444);
    border-radius: 4px;
    padding: 0.2rem 0.6rem;
    font: inherit;
    color: inherit;
    cursor: pointer;
  }
  .custom-form button:hover {
    background: light-dark(#eee, #2a2a2a);
  }
  .meta {
    display: flex;
    gap: 0.6rem;
    font-family: ui-monospace, monospace;
  }
  .added {
    color: #2ea043;
  }
  .removed {
    color: #cf222e;
  }
  .loading {
    color: light-dark(#888, #666);
  }
  .empty,
  .error {
    padding: 2rem;
    text-align: center;
    color: light-dark(#888, #aaa);
  }
  .error {
    color: light-dark(#a33, #ffb4b4);
  }
</style>
