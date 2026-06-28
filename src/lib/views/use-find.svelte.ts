import { onMount, onDestroy } from "svelte";
import { CmFindState } from "./find-cm.svelte";
import { FindState } from "./find.svelte";

/**
 * Wire a {@link CmFindState} into the host. Handles the boilerplate every
 * CodeMirror-backed view repeated: register the window keydown listener
 * (⌘F / Esc), tear down on destroy, and refresh marks when the dependency
 * function reads change.
 *
 * The host is still responsible for calling `find.bind(view)` after it has
 * created the EditorView (typically inside its own onMount). That keeps the
 * bind step a one-liner and dodges the script-order race where the composable
 * would run before the host had a view to bind to.
 *
 * `depsFn` is read inside an `$effect` so any `$state` it touches triggers a
 * `refresh()` — pass the same reactive reads (`void text; void find.query;`)
 * the manual `$effect` used to track.
 */
export function useCmFind(depsFn: () => void): CmFindState {
  const find = new CmFindState();

  onMount(() => {
    window.addEventListener("keydown", find.onKeydown);
  });

  onDestroy(() => {
    window.removeEventListener("keydown", find.onKeydown);
    find.destroy();
  });

  $effect(() => {
    depsFn();
    find.refresh();
  });

  return find;
}

/**
 * DOM-based variant for non-CodeMirror hosts (Preview, Diff sub-views,
 * WYSIWYG). Same lifecycle / keydown / refresh pattern as {@link useCmFind};
 * the host calls `find.bind(scope)` after the scoping element has been
 * `bind:this`-attached (typically inside its own onMount).
 */
export function useFind(depsFn: () => void): FindState {
  const find = new FindState();

  onMount(() => {
    window.addEventListener("keydown", find.onKeydown);
  });

  onDestroy(() => {
    window.removeEventListener("keydown", find.onKeydown);
    find.destroy();
  });

  $effect(() => {
    depsFn();
    find.refresh();
  });

  return find;
}
