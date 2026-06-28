<script lang="ts">
  import { i18n } from "$lib/i18n/index.svelte";
  import type { Heading } from "$lib/views/outline";

  let {
    headings,
    activeIndex,
    onJump,
    onClose,
  }: {
    headings: Heading[];
    activeIndex: number;
    onJump: (line: number) => void;
    onClose: () => void;
  } = $props();
</script>

<aside class="outline" aria-label={i18n.t("outline.title")}>
  <header>
    <h3>{i18n.t("outline.title")}</h3>
    <button
      type="button"
      class="close"
      aria-label={i18n.t("outline.close")}
      title={i18n.t("outline.close")}
      onclick={onClose}>×</button
    >
  </header>
  {#if headings.length === 0}
    <p class="empty">{i18n.t("outline.empty")}</p>
  {:else}
    <ol class="tree">
      {#each headings as h, i (h.line)}
        <li
          class:active={i === activeIndex}
          style="padding-left: {0.25 + (h.level - 1) * 0.85}rem"
        >
          <button
            type="button"
            class="row"
            onclick={() => onJump(h.line)}
            title={h.text}
          >
            <span class="level">H{h.level}</span>
            <span class="text">{h.text}</span>
          </button>
        </li>
      {/each}
    </ol>
  {/if}
</aside>

<style>
  .outline {
    width: 240px;
    flex-shrink: 0;
    border-left: 1px solid var(--mddiff-border-mute);
    background: var(--mddiff-surface);
    display: flex;
    flex-direction: column;
    overflow: hidden;
    font-size: 0.85rem;
  }
  /* Vertical span matches the floating ☰ button (top: 0.45rem + 34px),
     so the button sits centered inside the dark band instead of poking out
     past its bottom edge. The right padding reserves space so the ×
     close-button doesn't slide under the ☰ button. */
  header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0.5rem 0.75rem;
    padding-right: 3.5rem;
    min-height: 3rem;
    box-sizing: border-box;
    border-bottom: 1px solid var(--mddiff-border-mute);
    background: var(--mddiff-surface-hi);
  }
  h3 {
    margin: 0;
    font-size: 0.72rem;
    text-transform: uppercase;
    letter-spacing: 0.06em;
    color: var(--mddiff-text-mute);
    font-weight: 600;
  }
  .close {
    background: transparent;
    border: 0;
    cursor: pointer;
    color: var(--mddiff-text-mute);
    font-size: 1.1rem;
    line-height: 1;
    padding: 0 0.25rem;
    border-radius: 3px;
  }
  .close:hover {
    background: var(--mddiff-surface-pop);
    color: var(--mddiff-text);
  }
  .empty {
    padding: 1rem 0.75rem;
    color: var(--mddiff-text-mute);
    font-size: 0.85rem;
  }
  .tree {
    list-style: none;
    margin: 0;
    padding: 0.4rem 0;
    overflow-y: auto;
    flex: 1;
  }
  .tree li {
    margin: 0;
  }
  .row {
    display: flex;
    align-items: baseline;
    gap: 0.5rem;
    width: 100%;
    text-align: left;
    background: transparent;
    border: 0;
    padding: 0.2rem 0.75rem 0.2rem 0;
    font: inherit;
    color: var(--mddiff-text);
    cursor: pointer;
    border-radius: 0;
    overflow: hidden;
  }
  .row:hover {
    background: var(--mddiff-surface-hi);
  }
  li.active .row {
    background: var(--mddiff-accent-bg, var(--mddiff-surface-hi));
    color: var(--mddiff-accent-fg, var(--mddiff-text));
    font-weight: 600;
  }
  .level {
    font-family: ui-monospace, "SF Mono", Menlo, monospace;
    font-size: 0.68rem;
    color: var(--mddiff-text-subtle);
    flex-shrink: 0;
    line-height: 1;
  }
  .text {
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    flex: 1;
    min-width: 0;
  }
</style>
