import { onMount, onDestroy } from "svelte";

/**
 * Track whether the user is holding ⌘ (Mac) or Ctrl (Win/Linux) and reflect
 * that on `document.documentElement` via the `mddiff-modifier-down` class.
 * CSS uses the class to switch link cursors from `text` / `default` to
 * `pointer` in editor views, signaling "click here to navigate".
 *
 * Implementation note: we update on `mousemove` (rather than only keydown /
 * keyup) so the class also clears when the user releases the modifier
 * outside the window — keyup wouldn't fire in that case and the cursor
 * would get stuck on `pointer`.
 */
export function useModifierCursorTracking(): void {
  function update(e: MouseEvent | KeyboardEvent) {
    const down = e.metaKey || e.ctrlKey;
    document.documentElement.classList.toggle("mddiff-modifier-down", down);
  }

  function clear() {
    document.documentElement.classList.remove("mddiff-modifier-down");
  }

  onMount(() => {
    window.addEventListener("mousemove", update);
    window.addEventListener("keydown", update);
    window.addEventListener("keyup", update);
    window.addEventListener("blur", clear);
  });

  onDestroy(() => {
    window.removeEventListener("mousemove", update);
    window.removeEventListener("keydown", update);
    window.removeEventListener("keyup", update);
    window.removeEventListener("blur", clear);
    clear();
  });
}
