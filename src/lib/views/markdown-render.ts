import MarkdownIt from "markdown-it";
import type Token from "markdown-it/lib/token.mjs";
import DOMPurify from "dompurify";
import taskLists from "markdown-it-task-lists";

/**
 * Create a fresh MarkdownIt instance configured the way mdv's Preview / Diff
 * SideBySide views want it: GFM-flavored prose rendering with task-list
 * checkbox support enabled but inert (CSS-rendered, no click handler — the
 * checkbox state mirror is owned by the WYSIWYG view).
 *
 * Returns a new instance per call so callers can extend (`md.use(plugin)`)
 * without polluting other consumers. Cheap to construct — call from script
 * top-level alongside the view.
 *
 * To extend the pipeline (e.g. Mermaid, KaTeX), call `md.use(plugin)` on the
 * returned instance before passing it to {@link renderWithLineMap}.
 */
export function createPreviewMd(): MarkdownIt {
  const md = new MarkdownIt({
    html: true,
    linkify: true,
    breaks: false,
    typographer: true,
  });
  md.use(taskLists, { enabled: false, label: false });
  return md;
}

/**
 * Variant used by WYSIWYG for the line-map only (no rendering). Skips
 * typographer / task-list extensions — they alter token positions and we
 * only care about the source-line → top-level-block mapping. Cheaper for
 * the common case of "re-parse on every text change just to map blocks".
 */
export function createLineMapMd(): MarkdownIt {
  return new MarkdownIt({ html: true, linkify: true, breaks: false });
}

/**
 * Two-stage markdown-it pipeline that all the rendered-view consumers
 * (Preview, Diff Side-by-Side) share:
 *   1. parse to tokens (block-level tokens carry `token.map = [start, end_exclusive]`)
 *   2. for each block_open token: tag with `data-mdv-line` (drives scroll
 *      sync) and let the optional per-token hook attach anything else (Diff
 *      SBS uses it to inject `class="mdv-changed mdv-changed-{kind}"` when
 *      the token's source range overlaps a visible hunk on its side)
 *   3. render the (mutated) tokens through MarkdownIt's renderer, sharing
 *      the same `env` object as the parse step — taskLists hands state
 *      between phases via env, so a fresh `{}` here drops `[x]`/`[ ]` checkbox
 *      rendering. The env is owned per call so concurrent callers don't see
 *      each other's intermediate state.
 *   4. DOMPurify.sanitize with `data-mdv-line` whitelisted so it survives the
 *      attribute filter
 *
 * `perTokenHook` runs once per block_open token (same iteration that sets
 * the `data-mdv-line` attribute) and receives the token plus the 1-based
 * source line range. Return values are ignored — mutate the token in place
 * via `token.attrJoin` / `token.attrSet`.
 */
export function renderWithLineMap(
  md: MarkdownIt,
  text: string,
  perTokenHook?: (token: Token, startLine: number, endLine: number) => void,
): string {
  const env: Record<string, unknown> = {};
  const tokens = md.parse(text, env);

  for (const token of tokens) {
    if (!token.map || !token.type.endsWith("_open")) continue;
    const startLine = token.map[0] + 1;
    const endLine = token.map[1];
    token.attrJoin("data-mdv-line", String(startLine));
    perTokenHook?.(token, startLine, endLine);
  }

  return DOMPurify.sanitize(md.renderer.render(tokens, md.options, env), {
    ADD_ATTR: ["data-mdv-line"],
  });
}
