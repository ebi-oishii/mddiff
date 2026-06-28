import { doc } from "$lib/stores/doc.svelte";

export interface ScrollTrackerOptions {
  /**
   * Returns the top visible line number (1-based), or `null` when not
   * determinable (e.g. element detached and reporting a zero rect, no
   * content yet). The tracker only writes to `doc.currentLine` when
   * computeLine returns a non-null value, so each view is free to guard
   * against its own undefined states.
   */
  computeLine: () => number | null;
  /** Debounce delay before writing `doc.currentLine`. Defaults to 80ms — low
   * enough that mode switches feel current, high enough that a scroll-jog
   * doesn't write hundreds of times. */
  debounceMs?: number;
}

export interface ScrollTracker {
  /**
   * Run `computeLine()` immediately and update `doc.currentLine`. Call from
   * the host's onDestroy as a last-chance capture in case a scroll happened
   * in the final debounce window. Continuous scroll tracking already handles
   * the common case; this just covers the tail.
   */
  captureNow(): void;
  /** Remove the scroll listener and clear the pending debounce. Idempotent. */
  detach(): void;
}

/**
 * Attach a debounced scroll listener that writes the top visible source line
 * to `doc.currentLine`. This is the shared infrastructure behind cross-mode
 * scroll position retention — each view supplies its own `computeLine`
 * because how "top visible line" is determined differs by host
 * (CodeMirror posAtCoords vs. `data-mdv-line` block scan vs. ProseMirror
 * children index).
 *
 * Why continuous tracking instead of a one-shot read in onDestroy:
 * Svelte 5's onDestroy can fire after the DOM has been detached. By that
 * point getBoundingClientRect returns a zero rect and most computeLine
 * implementations bail or worse, write line 1 — silently clobbering the
 * saved line. Continuous capture sidesteps that race.
 */
export function attachScrollTracker(
  scroller: HTMLElement,
  opts: ScrollTrackerOptions,
): ScrollTracker {
  const debounceMs = opts.debounceMs ?? 80;
  let scrollTimer: ReturnType<typeof setTimeout> | null = null;
  let detached = false;

  function captureNow() {
    const line = opts.computeLine();
    if (line != null) doc.currentLine = line;
  }

  function onScroll() {
    if (scrollTimer) clearTimeout(scrollTimer);
    scrollTimer = setTimeout(captureNow, debounceMs);
  }

  scroller.addEventListener("scroll", onScroll, { passive: true });

  return {
    captureNow,
    detach() {
      if (detached) return;
      detached = true;
      if (scrollTimer) clearTimeout(scrollTimer);
      scroller.removeEventListener("scroll", onScroll);
    },
  };
}
