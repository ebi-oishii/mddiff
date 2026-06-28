<script lang="ts">
  import { doc } from "$lib/stores/doc.svelte";
  import { i18n } from "$lib/i18n/index.svelte";
  import { gitReadAt } from "$lib/ipc/git";

  let loading = $state(false);
  let error = $state<string | null>(null);

  const hist = $derived(doc.history);
  const canOlder = $derived(!!hist && hist.index < hist.commits.length - 1);
  const canNewer = $derived(!!hist && hist.index > 0);

  async function walk(delta: number) {
    if (!hist || !doc.path) return;
    const next = hist.index + delta;
    if (next < 0 || next >= hist.commits.length) return;
    const target = hist.commits[next];
    loading = true;
    error = null;
    try {
      const content = await gitReadAt(doc.path, target.revspec);
      doc.updateHistoryContent(next, target.revspec, target.label, content);
    } catch {
      error = i18n.t("history.error");
    } finally {
      loading = false;
    }
  }
</script>

{#if hist}
  <div class="history-banner" role="status">
    <div class="left">
      <span class="badge">⌖</span>
      <span class="prefix">{i18n.t("history.bannerReadonly")}</span>
      <span class="label" title={hist.revspec}>{hist.label}</span>
      <button
        type="button"
        class="unpin"
        onclick={() => doc.exitHistory()}
        aria-label={i18n.t("history.exit")}
        title={`${i18n.t("history.exit")} (Esc)`}
      >×</button>
      {#if loading}<span class="loading">…</span>{/if}
      {#if error}<span class="error">{error}</span>{/if}
    </div>
    <div class="right">
      <button
        type="button"
        class="nav"
        onclick={() => walk(1)}
        disabled={!canOlder || loading}
        title={i18n.t("history.prev")}
        aria-label={i18n.t("history.prev")}
      >←</button>
      <button
        type="button"
        class="nav"
        onclick={() => walk(-1)}
        disabled={!canNewer || loading}
        title={i18n.t("history.next")}
        aria-label={i18n.t("history.next")}
      >→</button>
      <button
        type="button"
        class="restore"
        onclick={() => doc.restoreHistory()}
        disabled={loading}
      >{i18n.t("history.restore")}</button>
    </div>
  </div>
{/if}

<style>
  .history-banner {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 0.75rem;
    padding: 0.45rem 0.85rem;
    background: var(--mddiff-accent-bg, var(--mddiff-surface-hi));
    color: var(--mddiff-accent-fg, var(--mddiff-text));
    border-bottom: 1px solid var(--mddiff-border);
    font-size: 0.85rem;
    flex-shrink: 0;
  }
  .left {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    min-width: 0;
  }
  .badge {
    font-family: ui-monospace, "SF Mono", Menlo, monospace;
    opacity: 0.8;
  }
  .prefix {
    opacity: 0.85;
  }
  .label {
    font-family: ui-monospace, "SF Mono", Menlo, monospace;
    font-weight: 600;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    min-width: 0;
  }
  .loading {
    opacity: 0.6;
  }
  .error {
    color: var(--mddiff-danger-fg);
  }
  .right {
    display: flex;
    align-items: center;
    gap: 0.3rem;
    flex-shrink: 0;
  }
  button {
    font: inherit;
    background: transparent;
    color: inherit;
    border: 1px solid var(--mddiff-border-mute);
    border-radius: 4px;
    padding: 0.18rem 0.6rem;
    cursor: pointer;
  }
  button:hover:not(:disabled) {
    background: var(--mddiff-surface-pop);
  }
  button:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }
  .nav {
    font-family: ui-monospace, "SF Mono", Menlo, monospace;
    padding: 0.18rem 0.5rem;
  }
  .restore {
    font-weight: 600;
  }
  /* Unpin sits in the label group on the left so the affordance is right
     next to the revision name it represents ("close THIS version"). */
  .unpin {
    flex-shrink: 0;
    border: 0;
    font-size: 1.1rem;
    line-height: 1;
    padding: 0 0.45rem;
    opacity: 0.8;
  }
  .unpin:hover {
    opacity: 1;
    background: var(--mddiff-surface-pop);
  }
</style>
