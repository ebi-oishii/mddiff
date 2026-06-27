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
export const mdvCmTheme = EditorView.theme({
  "&": {
    backgroundColor: "var(--mdv-editor-bg)",
    color: "var(--mdv-text)",
    height: "100%",
  },
  ".cm-content": {
    caretColor: "var(--mdv-text)",
  },
  ".cm-cursor, .cm-dropCursor": {
    borderLeftColor: "var(--mdv-text)",
  },
  ".cm-gutters": {
    backgroundColor: "var(--mdv-editor-gutter)",
    color: "var(--mdv-text-subtle)",
    borderRight: "1px solid var(--mdv-border-mute)",
  },
  ".cm-gutterElement": {
    color: "var(--mdv-text-subtle)",
  },
  ".cm-activeLine": {
    backgroundColor:
      "color-mix(in srgb, var(--mdv-accent) 6%, transparent)",
  },
  ".cm-activeLineGutter": {
    backgroundColor:
      "color-mix(in srgb, var(--mdv-accent) 12%, transparent)",
    color: "var(--mdv-text)",
  },
  "&.cm-focused .cm-selectionBackground, ::selection": {
    backgroundColor: "var(--mdv-accent-bg)",
  },
});
