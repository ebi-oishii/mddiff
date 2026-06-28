import { EditorView } from "@codemirror/view";

/**
 * Scroll a CodeMirror view to a given 1-based line number, clamped to the
 * document's range. Used by Source / Live Preview to restore the persisted
 * scroll position on mount when the user switches modes back into a
 * CM-backed view.
 *
 * Caller should typically wrap this in `requestAnimationFrame` so CodeMirror
 * has measured the layout first — calling it synchronously from `onMount`
 * before the editor has laid out its content can silently no-op.
 */
export function restoreCmToLine(view: EditorView, line: number): void {
  const total = view.state.doc.lines;
  const safe = Math.max(1, Math.min(total, line));
  const pos = view.state.doc.line(safe).from;
  view.dispatch({
    effects: EditorView.scrollIntoView(pos, { y: "start" }),
  });
}
