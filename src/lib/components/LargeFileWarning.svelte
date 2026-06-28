<script lang="ts">
  let {
    path,
    sizeBytes,
    onConfirm,
    onCancel,
  }: {
    path: string;
    sizeBytes: number;
    onConfirm: () => void;
    onCancel: () => void;
  } = $props();

  function formatSize(n: number): string {
    if (n < 1024) return `${n} B`;
    if (n < 1024 * 1024) return `${(n / 1024).toFixed(1)} KB`;
    return `${(n / (1024 * 1024)).toFixed(2)} MB`;
  }

  function basename(p: string): string {
    const parts = p.split(/[\\/]/);
    return parts[parts.length - 1] || p;
  }

  const sizeLabel = $derived(formatSize(sizeBytes));
</script>

<div class="overlay" role="presentation" onclick={onCancel}>
  <div
    class="modal"
    role="dialog"
    tabindex="-1"
    aria-modal="true"
    aria-labelledby="large-file-title"
    onclick={(e) => e.stopPropagation()}
    onkeydown={(e) => e.key === "Escape" && onCancel()}
  >
    <h2 id="large-file-title">Large file</h2>
    <p class="filename">{basename(path)}</p>
    <p class="size">
      Size: <strong>{sizeLabel}</strong> (warning threshold 5 MB)
    </p>
    <p class="hint">
      Opening files larger than 5 MB may make Live Preview, WYSIWYG and Diff
      slow or unresponsive. Source mode handles large files best — consider
      switching to it after opening.
    </p>

    <div class="actions">
      <button type="button" class="secondary" onclick={onCancel}>Cancel</button>
      <button type="button" class="primary" onclick={onConfirm}>Open anyway</button>
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
    max-width: 30em;
    width: 100%;
    box-shadow: 0 12px 40px var(--mdv-shadow);
  }
  h2 {
    margin: 0 0 0.5rem;
    font-size: 1.1rem;
    color: var(--mdv-warn-fg);
  }
  .filename {
    margin: 0.25rem 0;
    font-family: ui-monospace, "SF Mono", Menlo, monospace;
    font-size: 0.9rem;
    word-break: break-all;
  }
  .size {
    margin: 0.4rem 0 0.8rem;
    font-size: 0.9rem;
    color: var(--mdv-text-mute);
  }
  .hint {
    margin: 0.5rem 0 0;
    font-size: 0.85rem;
    color: var(--mdv-text-mute);
    line-height: 1.5;
  }
  .actions {
    display: flex;
    justify-content: flex-end;
    gap: 0.5rem;
    margin-top: 1.25rem;
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
  .actions button:hover {
    background: var(--mdv-surface-hi);
  }
  .actions button.primary {
    background: var(--mdv-warn-fg);
    color: light-dark(#fff, #1e1e1e);
    border-color: transparent;
  }
  .actions button.primary:hover {
    filter: brightness(0.92);
  }
</style>
