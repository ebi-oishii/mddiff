<script lang="ts">
  import { gitListBases } from "$lib/ipc/git";
  import { mdvPack } from "$lib/ipc/mdv";
  import { pickSavePath, writeFile } from "$lib/ipc/fs";
  import { humanizeError } from "$lib/errors";
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
      const out = await pickSavePath("mdv", "mdv package", path);
      if (!out) {
        loading = false;
        return;
      }
      const packed = await mdvPack(path, currentText, selected);
      await writeFile(out, packed.content);
      onSaved(
        `.mdv saved: ${packed.commit_count} commits, ${packed.snapshot_count} snapshots, ${formatBytes(packed.bundle_bytes)} compressed`,
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
    aria-labelledby="mdv-export-title"
    onclick={(e) => e.stopPropagation()}
    onkeydown={(e) => e.key === "Escape" && onCancel()}
  >
    <h2 id="mdv-export-title">Export .mdv with history</h2>
    <p class="help">
      Bundle the file's history from the chosen base up to the current buffer into a
      self-contained <code>.mdv</code>. The receiver can read it as plain Markdown,
      and your local Git stays untouched.
    </p>

    <label class="row">
      <span>Include history since:</span>
      <select bind:value={selected} disabled={loading}>
        <optgroup label="Special">
          {#each byKind("special") as b}
            <option value={b.revspec}>{label(b)}</option>
          {/each}
        </optgroup>
        {#if byKind("branch").length > 0}
          <optgroup label="Branches">
            {#each byKind("branch") as b}
              <option value={b.revspec}>{label(b)}</option>
            {/each}
          </optgroup>
        {/if}
        {#if byKind("tag").length > 0}
          <optgroup label="Tags">
            {#each byKind("tag") as b}
              <option value={b.revspec}>{label(b)}</option>
            {/each}
          </optgroup>
        {/if}
        {#if byKind("commit").length > 0}
          <optgroup label="Recent commits">
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
      <button type="button" onclick={onCancel} disabled={loading}>Cancel</button>
      <button type="button" class="primary" onclick={save} disabled={loading}>
        {loading ? "Packing…" : "Save .mdv"}
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
    background: var(--mdv-surface-pop);
    color: var(--mdv-text);
    border: 1px solid var(--mdv-border);
    border-radius: 8px;
    padding: 1.25rem 1.5rem;
    max-width: 36em;
    width: 100%;
    box-shadow: 0 12px 40px var(--mdv-shadow);
  }
  h2 {
    margin: 0 0 0.5rem;
    font-size: 1.1rem;
  }
  .help {
    margin: 0 0 1rem;
    font-size: 0.88rem;
    color: var(--mdv-text-mute);
    line-height: 1.5;
  }
  .help code {
    font-family: ui-monospace, "SF Mono", Menlo, monospace;
    font-size: 0.92em;
    padding: 0 0.25em;
    background: var(--mdv-surface-hi);
    border-radius: 3px;
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
    background: var(--mdv-bg);
    color: var(--mdv-text);
    border: 1px solid var(--mdv-border);
    border-radius: 4px;
  }
  .error {
    padding: 0.5rem 0.7rem;
    background: var(--mdv-danger-bg);
    color: var(--mdv-danger-fg);
    border: 1px solid var(--mdv-danger-border);
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
    border: 1px solid var(--mdv-border);
    border-radius: 5px;
    padding: 0.4rem 1rem;
    font: inherit;
    color: var(--mdv-text);
    cursor: pointer;
  }
  .actions button:hover:not(:disabled) {
    background: var(--mdv-surface-hi);
  }
  .actions button.primary {
    background: var(--mdv-accent);
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
