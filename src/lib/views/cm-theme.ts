import { EditorView } from "@codemirror/view";

/**
 * CodeMirror theme delivered through CM's own theming pipeline so it wins
 * the specificity fight against CM's built-in defaults. Plain `:global(.cm-*)`
 * CSS loses because CM's defaults use the same single-class specificity but
 * are injected at runtime, so they typically end up later in cascade order
 * and override ours — even with `light-dark()`.
 *
 * Values are read from our app-wide design tokens so this stays in sync with
 * the rest of the UI without duplicating colors.
 */
export const mddiffCmTheme = EditorView.theme({
  "&": {
    backgroundColor: "var(--mddiff-editor-bg)",
    color: "var(--mddiff-text)",
    height: "100%",
  },
  // CodeMirror's base theme paints a `outline: 1px dotted` on the focused
  // editor wrapper. That's invisible when the editor extends to the
  // viewport edge, but Source's outer padding pulls the wrapper inward and
  // the dotted line shows up against the bg. Suppress it — the caret +
  // active-line decoration already signal focus.
  "&.cm-focused": {
    outline: "none",
  },
  ".cm-content": {
    caretColor: "var(--mddiff-text)",
  },
  ".cm-cursor, .cm-dropCursor": {
    borderLeftColor: "var(--mddiff-text)",
  },
  ".cm-gutters": {
    backgroundColor: "var(--mddiff-editor-gutter)",
    color: "var(--mddiff-text-subtle)",
    borderRight: "1px solid var(--mddiff-border-mute)",
  },
  ".cm-gutterElement": {
    color: "var(--mddiff-text-subtle)",
  },
  ".cm-activeLine": {
    // Shared with .source::before's active-line extension overlay so the
    // strip outside cm-editor stays color-matched. Defined in +page.svelte.
    backgroundColor: "var(--mddiff-active-line-bg)",
  },
  ".cm-activeLineGutter": {
    backgroundColor:
      "color-mix(in srgb, var(--mddiff-accent) 12%, transparent)",
    color: "var(--mddiff-text)",
  },
  "&.cm-focused .cm-selectionBackground, ::selection": {
    backgroundColor: "var(--mddiff-accent-bg)",
  },
});
