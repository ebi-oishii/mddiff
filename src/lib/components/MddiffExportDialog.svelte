<script lang="ts">
  import { gitListBases } from "$lib/ipc/git";
  import { mddiffPack } from "$lib/ipc/mddiff";
  import { pickSavePath, writeFile } from "$lib/ipc/fs";
  import { humanizeError } from "$lib/errors";
  import { i18n } from "$lib/i18n/index.svelte";
  import type { BaseKind, BaseOption } from "$lib/types";

  let {
    path,
    currentText,
    onSaved,
    onCancel,
  }: {
    path: string;
    currentText: string;
    onSaved: (msg: string) => void;
    onCancel: () => void;
  } = $props();

  let bases = $state<BaseOption[]>([]);
  let selected = $state<string>("HEAD");
  let loading = $state(false);
  let error = $state<string | null>(null);

  $effect(() => {
    gitListBases(path, currentText)
      .then((b) => {
        bases = b;
      })
      .catch((e) => {
        error = humanizeError(e, "other");
      });
  });

  function byKind(kind: BaseKind): BaseOption[] {
    return bases.filter((b) => b.kind === kind);
  }

  function label(b: BaseOption): string {
    return b.label;
  }

  async function save() {
    loading = true;
    error = null;
    try {
      const out = await pickSavePath("mddiff", "mddiff package", path);
      if (!out) {
        loading = false;
        return;
      }
      const packed = await mddiffPack(path, currentText, selected);
      await writeFile(out, packed.content);
      onSaved(
        `.mddiff saved: ${packed.commit_count} commits, ${packed.snapshot_count} snapshots, ${formatBytes(packed.bundle_bytes)} compressed`,
      );
    } catch (e) {
      error = humanizeError(e, "write");
    } finally {
      loading = false;
    }
  }

  function formatBytes(n: number): string {
    if (n < 1024) return `${n} B`;
    if (n < 1024 * 1024) return `${(n / 1024).toFixed(1)} KB`;
    return `${(n / (1024 * 1024)).toFixed(2)} MB`;
  }
</script>

<div class="overlay" role="presentation" onclick={onCancel}>
  <div
    class="modal"
    role="dialog"
    tabindex="-1"
    aria-modal="true"
    aria-labelledby="mddiff-export-title"
    onclick={(e) => e.stopPropagation()}
    onkeydown={(e) => e.key === "Escape" && onCancel()}
  >
    <h2 id="mddiff-export-title">{i18n.t("mddiffExport.title")}</h2>

    <label class="row">
      <span>{i18n.t("mddiffExport.baseLabel")}</span>
      <select bind:value={selected} disabled={loading}>
        <optgroup label={i18n.t("mddiffExport.groupSpecial")}>
          {#each byKind("special") as b}
            <option value={b.revspec}>{label(b)}</option>
          {/each}
        </optgroup>
        {#if byKind("branch").length > 0}
          <optgroup label={i18n.t("mddiffExport.groupBranches")}>
            {#each byKind("branch") as b}
              <option value={b.revspec}>{label(b)}</option>
            {/each}
          </optgroup>
        {/if}
        {#if byKind("tag").length > 0}
          <optgroup label={i18n.t("mddiffExport.groupTags")}>
            {#each byKind("tag") as b}
              <option value={b.revspec}>{label(b)}</option>
            {/each}
          </optgroup>
        {/if}
        {#if byKind("commit").length > 0}
          <optgroup label={i18n.t("mddiffExport.groupRecent")}>
            {#each byKind("commit") as b}
              <option value={b.revspec}>{label(b)}</option>
            {/each}
          </optgroup>
        {/if}
      </select>
    </label>

    {#if error}
      <div class="error">{error}</div>
    {/if}

    <div class="actions">
      <button type="button" onclick={onCancel} disabled={loading}>{i18n.t("mddiffExport.cancel")}</button>
      <button type="button" class="primary" onclick={save} disabled={loading}>
        {loading ? i18n.t("mddiffExport.saving") : i18n.t("mddiffExport.save")}
      </button>
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
    background: var(--mddiff-surface-pop);
    color: var(--mddiff-text);
    border: 1px solid var(--mddiff-border);
    border-radius: 8px;
    padding: 1.25rem 1.5rem;
    max-width: 36em;
    width: 100%;
    box-shadow: 0 12px 40px var(--mddiff-shadow);
  }
  h2 {
    margin: 0 0 0.5rem;
    font-size: 1.1rem;
  }
  .row {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    margin: 0.75rem 0;
    font-size: 0.9rem;
  }
  .row span {
    flex-shrink: 0;
  }
  .row select {
    flex: 1;
    font: inherit;
    padding: 0.3rem 0.5rem;
    background: var(--mddiff-bg);
    color: var(--mddiff-text);
    border: 1px solid var(--mddiff-border);
    border-radius: 4px;
  }
  .error {
    padding: 0.5rem 0.7rem;
    background: var(--mddiff-danger-bg);
    color: var(--mddiff-danger-fg);
    border: 1px solid var(--mddiff-danger-border);
    border-radius: 4px;
    font-size: 0.85rem;
    margin: 0.5rem 0;
  }
  .actions {
    display: flex;
    justify-content: flex-end;
    gap: 0.5rem;
    margin-top: 1rem;
  }
  .actions button {
    background: transparent;
    border: 1px solid var(--mddiff-border);
    border-radius: 5px;
    padding: 0.4rem 1rem;
    font: inherit;
    color: var(--mddiff-text);
    cursor: pointer;
  }
  .actions button:hover:not(:disabled) {
    background: var(--mddiff-surface-hi);
  }
  .actions button.primary {
    background: var(--mddiff-accent);
    color: #fff;
    border-color: transparent;
  }
  .actions button.primary:hover:not(:disabled) {
    filter: brightness(0.92);
  }
  .actions button:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
</style>
