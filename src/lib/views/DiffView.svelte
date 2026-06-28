<script lang="ts">
  import { doc } from "$lib/stores/doc.svelte";
  import {
    gitFullDiff,
    gitHunks,
    gitListBases,
    gitSideBySide,
  } from "$lib/ipc/git";
  import type {
    BaseKind,
    BaseOption,
    DiffLine,
    DiffSubmode,
    HunkSummary,
    SideBySidePayload,
  } from "$lib/types";
  import HighlightView from "./diff/HighlightView.svelte";
  import FullDiffView from "./diff/FullDiffView.svelte";
  import SideBySideView from "./diff/SideBySideView.svelte";

  const CUSTOM = "__custom__";

  let submode = $state<DiffSubmode>("sidebyside");
  let bases = $state<BaseOption[]>([]);
  let selected = $state<string>("HEAD");
  let customBase = $state("HEAD");
  let hunks = $state<HunkSummary[]>([]);
  let lines = $state<DiffLine[]>([]);
  let sbs = $state<SideBySidePayload | null>(null);
  let error = $state<string | null>(null);
  let loading = $state(false);
  // Default: hide commits that didn't touch this file. The "All commits"
  // toggle reveals them as the sub-feature for per-commit comparison.
  let showAllCommits = $state(false);

  let diffTimer: ReturnType<typeof setTimeout> | null = null;
  let basesTimer: ReturnType<typeof setTimeout> | null = null;

  const isCustom = $derived(selected === CUSTOM);
  const base = $derived(isCustom ? customBase.trim() || "HEAD" : selected);

  $effect(() => {
    void doc.path;
    void doc.text;
    if (!doc.path || !doc.gitAvailable) return;
    if (basesTimer) clearTimeout(basesTimer);
    const path = doc.path;
    const text = doc.text;
    basesTimer = setTimeout(() => {
      gitListBases(path, text).then((b) => {
        bases = b;
      });
    }, 400);
    return () => {
      if (basesTimer) clearTimeout(basesTimer);
    };
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
      } else if (submode === "full") {
        lines = await gitFullDiff(doc.path, doc.text, base);
      } else {
        sbs = await gitSideBySide(doc.path, doc.text, base);
      }
    } catch (e) {
      error = String(e);
    } finally {
      loading = false;
    }
  }

  function byKind(kind: BaseKind): BaseOption[] {
    const all = bases.filter((b) => b.kind === kind);
    if (kind === "commit" && !showAllCommits) {
      return all.filter((b) => b.file_changed);
    }
    return all;
  }

  const hiddenCommitCount = $derived(
    bases.filter((b) => b.kind === "commit" && !b.file_changed).length,
  );

  function optionLabel(b: BaseOption): string {
    const marker =
      b.marker === "differs"
        ? "● "
        : b.marker === "identical"
          ? "○ "
          : "  "; // redundant or unknown
    return marker + b.label;
  }

  function applyCustom(e: SubmitEvent) {
    e.preventDefault();
    customBase = customBase.trim() || "HEAD";
  }

  const addedSum = $derived(
    hunks.reduce((n, h) => n + (h.kind === "removed" ? 0 : h.new_end - h.new_start + 1), 0),
  );
  const removedSum = $derived(
    hunks.reduce(
      (n, h) => n + (h.kind === "added" ? 0 : h.old_end - h.old_start + 1),
      0,
    ),
  );
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
        <button
          role="tab"
          aria-selected={submode === "sidebyside"}
          class:active={submode === "sidebyside"}
          onclick={() => (submode = "sidebyside")}
        >
          Side-by-Side
        </button>
      </div>
      <label class="base-select">
        <span class="prefix">vs</span>
        <select
          bind:value={selected}
          aria-label="Compare base revision"
          title="● = differs from current buffer · ○ = identical · (blank) = same content as a more recent commit shown above"
        >
          <optgroup label="Special">
            {#each byKind("special") as b}
              <option value={b.revspec} title={b.detail ?? undefined}>{optionLabel(b)}</option>
            {/each}
          </optgroup>
          {#if byKind("branch").length > 0}
            <optgroup label="Branches">
              {#each byKind("branch") as b}
                <option value={b.revspec}>{optionLabel(b)}</option>
              {/each}
            </optgroup>
          {/if}
          {#if byKind("tag").length > 0}
            <optgroup label="Tags">
              {#each byKind("tag") as b}
                <option value={b.revspec}>{optionLabel(b)}</option>
              {/each}
            </optgroup>
          {/if}
          {#if byKind("commit").length > 0}
            <optgroup
              label={showAllCommits
                ? "Recent commits (all)"
                : "Recent commits that changed this file"}
            >
              {#each byKind("commit") as b}
                <option value={b.revspec}>{optionLabel(b)}</option>
              {/each}
            </optgroup>
          {/if}
          <option value={CUSTOM}>Custom…</option>
        </select>
      </label>
      {#if hiddenCommitCount > 0 || showAllCommits}
        <!-- Sub-feature: per-commit comparison. Off by default so the
             picker only shows commits that actually changed the file. -->
        <label class="all-commits-toggle">
          <input
            type="checkbox"
            bind:checked={showAllCommits}
            aria-label="Show commits that didn't change this file"
          />
          <span>All commits</span>
          {#if !showAllCommits}<span class="muted">(+{hiddenCommitCount})</span>{/if}
        </label>
      {/if}
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
        <span class="added">+{addedSum}</span>
        <span class="removed">−{removedSum}</span>
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
  {:else if submode === "full"}
    <FullDiffView {lines} />
  {:else if sbs}
    <SideBySideView payload={sbs} baseLabel={base} />
  {:else}
    <div class="empty">Loading…</div>
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
    gap: 0.75rem;
    padding: 0.5rem 1rem;
    border-bottom: 1px solid var(--mdv-border);
    background: var(--mdv-surface);
    flex-shrink: 0;
    flex-wrap: wrap;
    font-size: 0.85rem;
  }
  /* In fullscreen the title overlay covers the top-left, hiding the
     Highlight / Full / Side-by-Side tabs. Push the toolbar down so the
     overlay sits in its own row above. */
  :global(:root[data-fullscreen]) .submode-bar {
    padding-top: 2.5rem;
  }
  .left {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    flex-wrap: wrap;
  }
  @media (max-width: 640px) {
    .submode-bar {
      padding: 0.4rem 0.6rem;
    }
    .submode-toggle button,
    .custom-form button {
      padding: 0.4rem 0.7rem;
    }
  }
  .submode-toggle {
    display: inline-flex;
    border: 1px solid var(--mdv-border);
    border-radius: 5px;
    overflow: hidden;
  }
  .submode-toggle button {
    background: transparent;
    border: 0;
    padding: 0.25rem 0.7rem;
    font: inherit;
    color: var(--mdv-text-mute);
    cursor: pointer;
  }
  .submode-toggle button + button {
    border-left: 1px solid var(--mdv-border);
  }
  .submode-toggle button.active {
    background: var(--mdv-accent-bg);
    color: var(--mdv-accent-fg);
  }
  .base-select {
    display: inline-flex;
    align-items: center;
    gap: 0.4rem;
    color: var(--mdv-text-mute);
  }
  .base-select .prefix {
    user-select: none;
  }
  .base-select select {
    font: inherit;
    background: var(--mdv-bg);
    color: var(--mdv-text);
    border: 1px solid var(--mdv-border);
    border-radius: 4px;
    padding: 0.22rem 0.4rem;
    max-width: 24em;
  }
  .all-commits-toggle {
    display: inline-flex;
    align-items: center;
    gap: 0.3rem;
    font-size: 0.82rem;
    color: var(--mdv-text-mute);
    cursor: pointer;
    user-select: none;
  }
  .all-commits-toggle input {
    margin: 0;
  }
  .all-commits-toggle .muted {
    color: var(--mdv-text-subtle);
    font-size: 0.92em;
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
    background: var(--mdv-bg);
    color: var(--mdv-text);
    border: 1px solid var(--mdv-border);
    border-radius: 4px;
  }
  .custom-form button {
    background: transparent;
    border: 1px solid var(--mdv-border);
    border-radius: 4px;
    padding: 0.2rem 0.6rem;
    font: inherit;
    color: var(--mdv-text);
    cursor: pointer;
  }
  .custom-form button:hover {
    background: var(--mdv-surface-hi);
  }
  .meta {
    display: flex;
    align-items: center;
    gap: 0.6rem;
    font-family: ui-monospace, monospace;
  }
  /* sync toggle lives inside SideBySideView itself (floating button between
     the two panes, Overleaf-style) */
  .added {
    color: var(--mdv-success-fg);
  }
  .removed {
    color: var(--mdv-danger-fg);
  }
  .loading {
    color: var(--mdv-text-mute);
  }
  .empty,
  .error {
    padding: 2rem;
    text-align: center;
    color: var(--mdv-text-mute);
  }
  .error {
    color: var(--mdv-danger-fg);
  }
</style>
