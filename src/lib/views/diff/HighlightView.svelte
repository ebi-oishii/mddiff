<script lang="ts">
  import { onDestroy, onMount } from "svelte";
  import type { HunkKind, HunkSummary } from "$lib/types";
  import { removedCount } from "$lib/types";
  import FindBar from "$lib/components/FindBar.svelte";
  import { FindState } from "../find.svelte";

  let { text, hunks }: { text: string; hunks: HunkSummary[] } = $props();

  const lines = $derived(text.split("\n"));

  function markFor(lineNo: number): HunkKind | "" {
    for (const h of hunks) {
      if (h.kind === "removed") continue;
      if (lineNo >= h.new_start && lineNo <= h.new_end) return h.kind;
    }
    return "";
  }

  function removedAfter(lineNo: number): number {
    let n = 0;
    for (const h of hunks) {
      if (h.kind === "removed" && h.new_start === lineNo) n += removedCount(h);
    }
    return n;
  }

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
    void text;
    void hunks;
    void find.query;
    void find.open;
    find.refresh();
  });
</script>

<div class="hl-scroller" bind:this={scroller}>
  <div class="hl">
    {#if removedAfter(0) > 0}
      <div class="rem-marker">— {removedAfter(0)} line{removedAfter(0) === 1 ? "" : "s"} removed —</div>
    {/if}
    {#each lines as line, i}
      {@const lineNo = i + 1}
      <div class="line" data-kind={markFor(lineNo)}>
        <span class="ln">{lineNo}</span>
        <span class="text">{line || " "}</span>
      </div>
      {#if removedAfter(lineNo) > 0}
        <div class="rem-marker">— {removedAfter(lineNo)} line{removedAfter(lineNo) === 1 ? "" : "s"} removed —</div>
      {/if}
    {/each}
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
  .hl-scroller {
    height: 100%;
    overflow: auto;
  }
  .hl {
    font-family: ui-monospace, "SF Mono", Menlo, Consolas, monospace;
    font-size: 14px;
    line-height: 1.5;
    padding: 0.5rem 0 4rem;
  }
  .line {
    display: flex;
    border-left: 3px solid transparent;
    padding-left: 0;
  }
  .line[data-kind="added"] {
    border-left-color: #2ea043;
    background: light-dark(#e6ffec, rgba(46, 160, 67, 0.12));
  }
  .line[data-kind="modified"] {
    border-left-color: #d29922;
    background: light-dark(#fff8c5, rgba(210, 153, 34, 0.12));
  }
  .ln {
    display: inline-block;
    width: 4.5em;
    color: light-dark(#888, #666);
    text-align: right;
    padding: 0 1em 0 0.5em;
    user-select: none;
    flex-shrink: 0;
  }
  .text {
    white-space: pre;
    flex: 1;
    min-width: 0;
    padding-right: 1em;
  }
  .rem-marker {
    font-size: 0.78rem;
    color: light-dark(#cf222e, #ff7b7b);
    padding: 0.15rem 0.5rem;
    text-align: center;
    border-top: 1px dashed light-dark(#fcc, #663);
    border-bottom: 1px dashed light-dark(#fcc, #663);
    background: light-dark(#ffeef0, rgba(207, 34, 46, 0.1));
  }
</style>
