import { syntaxTree } from "@codemirror/language";
import { RangeSetBuilder } from "@codemirror/state";
import {
  Decoration,
  type DecorationSet,
  EditorView,
  ViewPlugin,
  type ViewUpdate,
} from "@codemirror/view";

/**
 * Mask markdown source ranges that aren't natural prose from the browser /
 * OS spellchecker so we don't get red underlines all over identifiers, URLs
 * and HTML. We achieve this by stamping `spellcheck="false"` on those
 * ranges via CM decorations — nested `spellcheck="false"` overrides the
 * editor-level `spellcheck="true"` on those substrings only.
 *
 * Node names follow @lezer/markdown (used by @codemirror/lang-markdown).
 * The selection is intentionally narrow: only content that's almost
 * guaranteed non-prose. Marker chars like `#`, `**`, `>` are single
 * non-alphabetic glyphs and spellcheckers already ignore them.
 */
const NO_SPELLCHECK_NODES = new Set<string>([
  // Code (variable names / identifiers are not prose)
  "InlineCode",
  "CodeText", // contents of a fenced code block
  "CodeBlock", // indented code block
  "CodeInfo", // language tag after the opening fence
  // URLs (domain components and paths)
  "URL",
  "Autolink",
  // Raw HTML
  "HTMLBlock",
  "HTMLTag",
  "CommentBlock",
  "ProcessingInstructionBlock",
]);

const noSpellcheck = Decoration.mark({
  attributes: { spellcheck: "false" },
});

function buildDecorations(view: EditorView): DecorationSet {
  const builder = new RangeSetBuilder<Decoration>();
  for (const { from, to } of view.visibleRanges) {
    syntaxTree(view.state).iterate({
      from,
      to,
      enter: (node) => {
        if (NO_SPELLCHECK_NODES.has(node.name)) {
          // Decorations must be in document order with start positions
          // strictly non-decreasing. The lezer iterator already visits
          // nodes in document order, so this is fine.
          builder.add(node.from, node.to, noSpellcheck);
        }
      },
    });
  }
  return builder.finish();
}

/**
 * Decoration plugin that paints `spellcheck="false"` on non-prose markdown
 * ranges (code, URLs, HTML). Stays cheap because it only re-builds on
 * docChanged / viewportChanged — selection or scroll alone don't rebuild.
 *
 * Safe to include unconditionally: when the editor-level spellcheck is
 * off, the false attribute is a no-op overlay.
 */
export const markdownSpellcheckMask = ViewPlugin.fromClass(
  class {
    decorations: DecorationSet;
    constructor(view: EditorView) {
      this.decorations = buildDecorations(view);
    }
    update(update: ViewUpdate) {
      if (update.docChanged || update.viewportChanged) {
        this.decorations = buildDecorations(update.view);
      }
    }
  },
  { decorations: (v) => v.decorations },
);
