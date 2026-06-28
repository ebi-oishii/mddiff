import { HighlightStyle, syntaxHighlighting } from "@codemirror/language";
import { tags } from "@lezer/highlight";

/**
 * Markdown-aware syntax highlight for the Source view's CodeMirror editor.
 * Colors come from CSS custom properties (--mdv-syntax-*) so light / dark /
 * auto themes hand back the right value automatically. The actual color
 * tokens are defined in +page.svelte alongside the rest of the app palette.
 *
 * Inspired by GitHub Primer for at-a-glance familiarity.
 */
const mdvHighlightStyle = HighlightStyle.define([
  // Headings: bold + accent color. Levels 1–3 are heavier than 4–6 to mimic
  // how rendered Markdown weights its hierarchy.
  { tag: tags.heading1, color: "var(--mdv-syntax-heading)", fontWeight: "700" },
  { tag: tags.heading2, color: "var(--mdv-syntax-heading)", fontWeight: "700" },
  { tag: tags.heading3, color: "var(--mdv-syntax-heading)", fontWeight: "700" },
  { tag: tags.heading4, color: "var(--mdv-syntax-heading)", fontWeight: "600" },
  { tag: tags.heading5, color: "var(--mdv-syntax-heading)", fontWeight: "600" },
  { tag: tags.heading6, color: "var(--mdv-syntax-heading)", fontWeight: "600" },
  // Emphasis / strong: just typographic, no color change so it reads natural.
  { tag: tags.strong, fontWeight: "700" },
  { tag: tags.emphasis, fontStyle: "italic" },
  { tag: tags.strikethrough, textDecoration: "line-through" },
  // Inline code and fenced code blocks share a token color (the content,
  // not the backticks). The backticks themselves are processingInstruction.
  { tag: tags.monospace, color: "var(--mdv-syntax-code)" },
  // Link target / URL text. Anchor labels are `link`, raw URLs are `url`.
  { tag: tags.url, color: "var(--mdv-syntax-link)" },
  { tag: tags.link, color: "var(--mdv-syntax-link)" },
  // Blockquote text rendered as muted italic so quoted prose stands apart.
  { tag: tags.quote, color: "var(--mdv-syntax-quote)", fontStyle: "italic" },
  // Markdown punctuation — the *, **, #, > etc. that the user typed but
  // are visual noise once parsed. Render them in a muted color.
  { tag: tags.processingInstruction, color: "var(--mdv-syntax-punct)" },
  // Thematic breaks (--- / ***).
  { tag: tags.contentSeparator, color: "var(--mdv-syntax-punct)" },
  // YAML front-matter or other meta blocks (e.g. Jekyll-style).
  { tag: tags.meta, color: "var(--mdv-syntax-meta)" },
]);

export const mdvSyntaxHighlighting = syntaxHighlighting(mdvHighlightStyle);
