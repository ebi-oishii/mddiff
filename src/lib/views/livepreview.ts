import { syntaxTree } from "@codemirror/language";
import { type EditorState, type Extension, RangeSet, type Range } from "@codemirror/state";
import {
  Decoration,
  type DecorationSet,
  EditorView,
  ViewPlugin,
  type ViewUpdate,
} from "@codemirror/view";

/**
 * CodeMirror 6 extension implementing a Typora-style live preview:
 *   - inline / heading syntax characters (e.g. `**`, `#`, `` ` ``, `[`, `](url)`)
 *     are visually hidden via Decoration.replace
 *   - the rendered text receives a class (mddiff-lp-h1, -bold, -italic, -code, -link, ...)
 *   - if the cursor / selection touches the same line as a node, that node is
 *     "active" and stays raw so the user can edit the markdown directly
 *
 * No round-tripping happens — the underlying document is always plain Markdown,
 * so `*foo*` vs `_foo_` etc. are preserved exactly.
 */
function selectionLines(state: EditorState): [number, number] {
  const sel = state.selection.main;
  return [
    state.doc.lineAt(sel.from).number,
    state.doc.lineAt(sel.to).number,
  ];
}

function nodeLines(state: EditorState, from: number, to: number): [number, number] {
  return [
    state.doc.lineAt(from).number,
    state.doc.lineAt(Math.max(from, Math.min(to, state.doc.length))).number,
  ];
}

function isActive(
  nodeFrom: number,
  nodeTo: number,
  selFromLine: number,
  selToLine: number,
  state: EditorState,
): boolean {
  const [fromLine, toLine] = nodeLines(state, nodeFrom, nodeTo);
  return !(selToLine < fromLine || selFromLine > toLine);
}

const HIDDEN = Decoration.replace({});

function buildDecorations(state: EditorState): DecorationSet {
  const ranges: Range<Decoration>[] = [];
  const [selFromLine, selToLine] = selectionLines(state);

  syntaxTree(state).iterate({
    enter(node) {
      const name = node.name;
      const active = isActive(node.from, node.to, selFromLine, selToLine, state);

      switch (name) {
        // Headings: always apply size class; hide leading "# " when inactive.
        case "ATXHeading1":
        case "ATXHeading2":
        case "ATXHeading3":
        case "ATXHeading4":
        case "ATXHeading5":
        case "ATXHeading6": {
          const level = name.slice("ATXHeading".length);
          ranges.push(
            Decoration.mark({ class: `mddiff-lp-h${level}` }).range(node.from, node.to),
          );
          return;
        }
        case "HeaderMark": {
          if (!active) {
            // Include the single space that follows `#`, `##`, ... if present.
            const trailing = state.doc.sliceString(node.to, node.to + 1);
            const to = trailing === " " ? node.to + 1 : node.to;
            if (to > node.from) {
              ranges.push(HIDDEN.range(node.from, to));
            }
          }
          return;
        }

        case "StrongEmphasis": {
          ranges.push(
            Decoration.mark({ class: "mddiff-lp-bold" }).range(node.from, node.to),
          );
          return;
        }
        case "Emphasis": {
          ranges.push(
            Decoration.mark({ class: "mddiff-lp-italic" }).range(node.from, node.to),
          );
          return;
        }
        case "InlineCode": {
          ranges.push(
            Decoration.mark({ class: "mddiff-lp-code" }).range(node.from, node.to),
          );
          return;
        }
        case "Link": {
          ranges.push(
            Decoration.mark({ class: "mddiff-lp-link" }).range(node.from, node.to),
          );
          return;
        }

        // Mark characters to hide when inactive.
        case "EmphasisMark":
        case "CodeMark":
        case "LinkMark": {
          if (!active) {
            ranges.push(HIDDEN.range(node.from, node.to));
          }
          return;
        }
        // Hide the URL part of `[text](url)` and link title when inactive.
        case "URL":
        case "LinkTitle": {
          if (!active) {
            ranges.push(HIDDEN.range(node.from, node.to));
          }
          return;
        }
      }
    },
  });

  return Decoration.set(ranges, true);
}

export const livePreviewExtension: Extension = ViewPlugin.fromClass(
  class {
    decorations: DecorationSet;
    constructor(view: EditorView) {
      this.decorations = buildDecorations(view.state);
    }
    update(update: ViewUpdate) {
      if (update.docChanged || update.selectionSet || update.viewportChanged) {
        this.decorations = buildDecorations(update.state);
      }
    }
  },
  {
    decorations: (v) => v.decorations,
    // Treat hidden ranges as atomic for cursor movement, so a single arrow key
    // jumps over `**` instead of stepping into the hidden gap.
    provide: (plugin) =>
      EditorView.atomicRanges.of((view) => {
        return view.plugin(plugin)?.decorations || RangeSet.empty;
      }),
  },
);
