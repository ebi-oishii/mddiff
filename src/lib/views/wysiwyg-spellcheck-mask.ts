import { Plugin, PluginKey } from "@milkdown/prose/state";
import { Decoration, DecorationSet } from "@milkdown/prose/view";

/**
 * ProseMirror plugin that adds `spellcheck="false"` to every code block
 * (node) and inline code (mark) range. We use PM decorations — not raw
 * DOM mutation — so PM treats the attribute as part of its own render
 * pipeline. That sidesteps the deadlock that the v0.2.0 MutationObserver
 * approach hit: PM's internal observer detected our external setAttribute
 * as content drift, re-rendered the node to "fix" it, our observer
 * re-fired on the resulting childList change, infinite loop.
 *
 * Schema names follow Milkdown's commonmark preset:
 *   - block code → node `code_block`
 *   - inline code → mark `inlineCode`
 */
const spellcheckMaskKey = new PluginKey("wysiwyg-spellcheck-mask");

export const wysiwygSpellcheckMaskPlugin = new Plugin({
  key: spellcheckMaskKey,
  props: {
    decorations(state) {
      const decos: Decoration[] = [];
      state.doc.descendants((node, pos) => {
        // Block: stamp the whole node.
        if (node.type.name === "code_block") {
          decos.push(
            Decoration.node(pos, pos + node.nodeSize, { spellcheck: "false" }),
          );
          return false; // no recursion into the code text
        }
        // Inline marks: any text node carrying the inlineCode mark gets
        // a span wrapper with spellcheck="false". `Decoration.inline`
        // wraps the range in an additional element so the attribute lands
        // on a DOM ancestor of the marked text — browsers respect the
        // closest spellcheck attribute up the chain.
        if (node.isText) {
          for (const mark of node.marks) {
            if (mark.type.name === "inlineCode") {
              decos.push(
                Decoration.inline(pos, pos + node.nodeSize, {
                  spellcheck: "false",
                }),
              );
              break;
            }
          }
        }
        return true;
      });
      return DecorationSet.create(state.doc, decos);
    },
  },
});
