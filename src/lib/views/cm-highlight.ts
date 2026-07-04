import { syntaxTree } from "@codemirror/language";
import { RangeSetBuilder } from "@codemirror/state";
import {
  Decoration,
  type DecorationSet,
  EditorView,
  ViewPlugin,
  type ViewUpdate,
} from "@codemirror/view";
import { hljs } from "./highlight.svelte";

/**
 * Live Preview syntax highlighting for fenced code blocks.
 *
 * We can't just point highlight.js at a `<pre>` DOM element (there isn't
 * one — Live Preview keeps the raw markdown in CodeMirror's document) so
 * this plugin bridges the gap:
 *
 *   1. Walk the lezer-markdown syntax tree for `FencedCode` nodes inside
 *      the current viewport (visible ranges only, keeps big docs cheap).
 *   2. For each fenced block, pluck its language ident from `CodeInfo`
 *      and the body from `CodeText`, feed the body through
 *      `hljs.highlight()` and get token HTML back.
 *   3. Walk that HTML with a tiny stack-based parser to recover
 *      `{from, to, className}` triples anchored at the original source
 *      offsets — hljs escapes special chars to entities and we count
 *      each entity as one source char to keep positions in sync.
 *   4. Emit CM `Decoration.mark` at those offsets with the same
 *      `hljs-*` class names highlight.js would use, so the active theme
 *      CSS (injected into <head> by `applyHighlightTheme`) paints the
 *      right colors without a second stylesheet.
 *
 * Only fenced blocks with a language tag that highlight.js recognises
 * get decorated; anything else falls through unchanged. Cheap to
 * rebuild — the plugin only recomputes on `docChanged` or
 * `viewportChanged`, not on selection changes.
 */

interface HljsToken {
  from: number; // 0-based source offset within the code block body
  to: number;
  className: string;
}

/**
 * Recover token ranges from highlight.js HTML by simulating source
 * position as we walk. Nested `<span>` tags are common (e.g. template
 * literal `${expr}` inside a string) so we use a stack rather than a
 * flat scan.
 */
function tokenize(html: string): HljsToken[] {
  const tokens: HljsToken[] = [];
  const stack: { className: string; from: number }[] = [];
  let pos = 0;
  let i = 0;
  while (i < html.length) {
    if (html.startsWith('<span class="', i)) {
      const q = html.indexOf('"', i + 13);
      if (q < 0) break;
      const className = html.substring(i + 13, q);
      const gt = html.indexOf(">", q);
      if (gt < 0) break;
      stack.push({ className, from: pos });
      i = gt + 1;
    } else if (html.startsWith("</span>", i)) {
      const top = stack.pop();
      if (top) tokens.push({ from: top.from, to: pos, className: top.className });
      i += 7;
    } else if (html[i] === "&") {
      // HTML entity (&amp; &lt; &gt; &quot; &#39; ...). Each represents
      // exactly one source character regardless of encoded length.
      const semi = html.indexOf(";", i);
      if (semi < 0 || semi - i > 8) {
        // Malformed / bare `&` — treat as one literal char.
        pos++;
        i++;
      } else {
        pos++;
        i = semi + 1;
      }
    } else {
      pos++;
      i++;
    }
  }
  return tokens;
}

function buildDecorations(view: EditorView): DecorationSet {
  const doc = view.state.doc;
  const tree = syntaxTree(view.state);

  // First pass: collect language + range per visible fenced block. We
  // resolve highlight and produce decorations in a second pass so we
  // can sort by `from` ascending before feeding RangeSetBuilder (which
  // requires non-decreasing starts).
  type Block = { language: string; codeFrom: number; codeTo: number };
  const blocks: Block[] = [];

  for (const { from, to } of view.visibleRanges) {
    tree.iterate({
      from,
      to,
      enter: (node) => {
        if (node.name !== "FencedCode") return;
        let language = "";
        let codeFrom = -1;
        let codeTo = -1;
        // Iterate direct children to find CodeInfo (lang tag) and
        // CodeText (body). We don't recurse — those are always
        // top-level children of a FencedCode.
        for (
          let child = node.node.firstChild;
          child != null;
          child = child.nextSibling
        ) {
          if (child.name === "CodeInfo") {
            language = doc.sliceString(child.from, child.to).trim().toLowerCase();
          } else if (child.name === "CodeText") {
            codeFrom = child.from;
            codeTo = child.to;
          }
        }
        if (language && codeFrom >= 0) {
          blocks.push({ language, codeFrom, codeTo });
        }
      },
    });
  }

  // Collect all token ranges globally, then feed them to the builder in
  // sorted order. Building per-block wouldn't help because CodeMirror's
  // RangeSetBuilder requires globally non-decreasing `from` values.
  type Ranged = { from: number; to: number; className: string };
  const ranged: Ranged[] = [];
  for (const { language, codeFrom, codeTo } of blocks) {
    if (!hljs.getLanguage(language)) continue;
    const code = doc.sliceString(codeFrom, codeTo);
    let html: string;
    try {
      html = hljs.highlight(code, { language, ignoreIllegals: true }).value;
    } catch {
      continue;
    }
    for (const t of tokenize(html)) {
      const from = codeFrom + t.from;
      const to = codeFrom + t.to;
      if (from < to && to <= codeTo) {
        ranged.push({ from, to, className: `hljs-${stripHljsPrefix(t.className)}` });
      }
    }
  }
  // Sort primarily by from asc; ties broken by to desc so outer
  // wrapping tokens come before inner tokens at the same offset.
  ranged.sort((a, b) => a.from - b.from || b.to - a.to);

  const builder = new RangeSetBuilder<Decoration>();
  for (const r of ranged) {
    builder.add(r.from, r.to, Decoration.mark({ class: r.className }));
  }
  return builder.finish();
}

// hljs class names already look like `hljs-keyword`, `hljs-string`,
// etc. Some themes also match on the bare `hljs-*` names, so we
// normalize: if the token starts with `hljs-`, leave it; otherwise
// prefix it. Belt-and-suspenders since current hljs output already
// includes the prefix.
function stripHljsPrefix(name: string): string {
  return name.startsWith("hljs-") ? name.slice(5) : name;
}

export const codeHighlightPlugin = ViewPlugin.fromClass(
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
