<script lang="ts">
  import { onDestroy, onMount } from "svelte";
  import type { DiffLine } from "$lib/types";
  import FindBar from "$lib/components/FindBar.svelte";
  import { FindState } from "../find.svelte";

  let { lines }: { lines: DiffLine[] } = $props();

  let scroller: HTMLDivElement;
  const find = new FindState();

  onMount(() => {
    find.bind(scroller);
    window.addEventListener("keydown", find.onKeydown);
  });

  onDestroy(() => {
    window.removeEventListener("keydown", find.onKeydown);
    find.destroy();
  });

  $effect(() => {
    void lines;
    void find.query;
    void find.open;
    find.refresh();
  });
</script>

<div class="diff-scroller" bind:this={scroller}>
  <div class="diff">
    {#if lines.length === 0}
      <div class="empty">No differences.</div>
    {:else}
      {#each lines as line, i (i)}
        {#if line.kind === "equal"}
          <div class="row equal">
            <span class="ln">{line.old_no}</span>
            <span class="ln">{line.new_no}</span>
            <span class="sign"> </span>
            <span class="text">{line.text || " "}</span>
          </div>
        {:else if line.kind === "added"}
          <div class="row added">
            <span class="ln"></span>
            <span class="ln">{line.new_no}</span>
            <span class="sign">+</span>
            <span class="text">{line.text || " "}</span>
          </div>
        {:else}
          <div class="row removed">
            <span class="ln">{line.old_no}</span>
            <span class="ln"></span>
            <span class="sign">−</span>
            <span class="text">{line.text || " "}</span>
          </div>
        {/if}
      {/each}
    {/if}
  </div>
</div>
{#if find.open}
  <FindBar
    bind:query={find.query}
    matchCount={find.matchCount}
    currentIndex={find.currentIndex}
    focusVersion={find.focusVersion}
    onnext={find.next}
    onprev={find.prev}
    onclose={find.close}
  />
{/if}

<style>
  .diff-scroller {
    height: 100%;
    overflow: auto;
  }
  .diff {
    font-family: ui-monospace, "SF Mono", Menlo, Consolas, monospace;
    font-size: 14px;
    line-height: 1.5;
    padding-bottom: 4rem;
  }
  .row {
    display: flex;
  }
  .row.added {
    background: light-dark(#e6ffec, rgba(46, 160, 67, 0.15));
  }
  .row.removed {
    background: light-dark(#ffebe9, rgba(207, 34, 46, 0.15));
  }
  .ln {
    display: inline-block;
    width: 4em;
    color: light-dark(#999, #666);
    text-align: right;
    padding: 0 0.6em;
    user-select: none;
    flex-shrink: 0;
  }
  .sign {
    width: 1.2em;
    text-align: center;
    user-select: none;
    flex-shrink: 0;
    color: light-dark(#666, #888);
  }
  .added .sign {
    color: #2ea043;
  }
  .removed .sign {
    color: #cf222e;
  }
  .text {
    white-space: pre;
    flex: 1;
    min-width: 0;
    padding-right: 1em;
  }
  .empty {
    padding: 2rem;
    text-align: center;
    color: light-dark(#888, #666);
  }
</style>
