import { DomFinder } from "./find-dom";

/**
 * Reactive find state for DOM-based views (Preview / Diff sub-views).
 * Holds the open flag, current query, match stats, and a re-focus version
 * counter for the FindBar. Provides imperative methods that wrap a
 * DomFinder targeting the view's scroll container.
 *
 * Usage:
 *   const find = new FindState();
 *   onMount(() => {
 *     find.bind(scroller);
 *     window.addEventListener("keydown", find.onKeydown);
 *   });
 *   $effect(() => {
 *     void textOrHtml;  // re-trigger when content changes
 *     void find.query;
 *     find.refresh();
 *   });
 */
export class FindState {
  open = $state(false);
  query = $state("");
  matchCount = $state(0);
  currentIndex = $state(0);
  focusVersion = $state(0);

  private finder: DomFinder | null = null;

  bind(scope: HTMLElement | null) {
    this.finder = scope ? new DomFinder(scope) : null;
  }

  openFind = () => {
    this.open = true;
    this.focusVersion++;
  };

  close = () => {
    this.open = false;
    this.query = "";
    this.finder?.clear();
    this.matchCount = 0;
    this.currentIndex = 0;
  };

  next = () => {
    this.finder?.next();
    this.syncFromFinder();
  };

  prev = () => {
    this.finder?.prev();
    this.syncFromFinder();
  };

  /** Re-apply the current query against the DOM. Called from a $effect that
   * watches `open`, `query`, and the host's content. Defers one frame so
   * Svelte-rendered DOM updates have landed before traversal. */
  refresh = () => {
    if (!this.finder) return;
    if (!this.open || !this.query) {
      this.finder.clear();
      this.matchCount = 0;
      this.currentIndex = 0;
      return;
    }
    requestAnimationFrame(() => {
      if (!this.finder) return;
      this.finder.apply(this.query);
      this.syncFromFinder();
    });
  };

  /** Keydown handler intended for `window.addEventListener`. Opens the bar
   * on ⌘F / Ctrl+F. */
  onKeydown = (e: KeyboardEvent) => {
    if ((e.metaKey || e.ctrlKey) && e.key.toLowerCase() === "f") {
      e.preventDefault();
      this.openFind();
    }
  };

  destroy() {
    this.finder?.clear();
    this.finder = null;
  }

  private syncFromFinder() {
    if (!this.finder) return;
    this.matchCount = this.finder.matchCount;
    this.currentIndex = this.finder.currentIndex;
  }
}
