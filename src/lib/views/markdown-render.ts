import MarkdownIt from "markdown-it";
import type Token from "markdown-it/lib/token.mjs";
import DOMPurify from "dompurify";
import taskLists from "markdown-it-task-lists";
import anchor from "markdown-it-anchor";
import { rewriteRelativeImageSrc } from "./image-path";

/**
 * Create a fresh MarkdownIt instance configured the way mddiff's Preview / Diff
 * SideBySide views want it: GFM-flavored prose rendering with task-list
 * checkbox support enabled but inert (CSS-rendered, no click handler — the
 * checkbox state mirror is owned by the WYSIWYG view).
 *
 * The image renderer is overridden to rewrite relative `<img src>` paths to
 * Tauri's `asset:` URL scheme, anchored at `env.docPath`. Without this the
 * webview's CSP refuses to load local files and pasted images stay invisible.
 * Absolute paths and `http(s):` / `data:` / etc. URLs pass through unchanged.
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
  // Auto-generate `id="..."` slugs on headings so `[link](#installation)`
  // anchor jumps work without manual id markup. Default GFM-style slugifier
  // matches what most users expect (`Installation` → `installation`).
  md.use(anchor, { permalink: false });

  const defaultImage = md.renderer.rules.image;
  md.renderer.rules.image = function (tokens, idx, options, env, self) {
    const token = tokens[idx];
    const srcIdx = token.attrIndex("src");
    const docPath = (env as RenderEnv).docPath ?? null;
    if (srcIdx >= 0 && token.attrs) {
      const src = token.attrs[srcIdx][1];
      token.attrs[srcIdx][1] = rewriteRelativeImageSrc(src, docPath);
    }
    return defaultImage
      ? defaultImage(tokens, idx, options, env, self)
      : self.renderToken(tokens, idx, options);
  };

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

type RenderEnv = { docPath?: string };

/**
 * Two-stage markdown-it pipeline that all the rendered-view consumers
 * (Preview, Diff Side-by-Side) share:
 *   1. parse to tokens (block-level tokens carry `token.map = [start, end_exclusive]`)
 *   2. for each block_open token: tag with `data-mddiff-line` (drives scroll
 *      sync) and let the optional per-token hook attach anything else (Diff
 *      SBS uses it to inject `class="mddiff-changed mddiff-changed-{kind}"` when
 *      the token's source range overlaps a visible hunk on its side)
 *   3. render the (mutated) tokens through MarkdownIt's renderer, sharing
 *      the same `env` object as the parse step. The env also carries
 *      `docPath` for the image renderer's relative-path rewriting.
 *   4. DOMPurify.sanitize with `data-mddiff-line` whitelisted so it survives the
 *      attribute filter
 *
 * `docPath` is the absolute path of the open document — `null` for the
 * untitled buffer. When `null`, relative image paths render as-is (the
 * webview will fail to load them, but no rewriting is possible anyway since
 * there's no anchor for the relative path).
 *
 * `perTokenHook` runs once per block_open token (same iteration that sets
 * the `data-mddiff-line` attribute) and receives the token plus the 1-based
 * source line range. Return values are ignored — mutate the token in place
 * via `token.attrJoin` / `token.attrSet`.
 */
export function renderWithLineMap(
  md: MarkdownIt,
  text: string,
  docPath: string | null,
  perTokenHook?: (token: Token, startLine: number, endLine: number) => void,
): string {
  const env: RenderEnv = { docPath: docPath ?? undefined };
  const tokens = md.parse(text, env);

  for (const token of tokens) {
    if (!token.map || !token.type.endsWith("_open")) continue;
    const startLine = token.map[0] + 1;
    const endLine = token.map[1];
    token.attrJoin("data-mddiff-line", String(startLine));
    perTokenHook?.(token, startLine, endLine);
  }

  return DOMPurify.sanitize(md.renderer.render(tokens, md.options, env), {
    // markdown-it-anchor adds `id` to headings; keep that. data-mddiff-line
    // is our own attribute for scroll sync.
    ADD_ATTR: ["data-mddiff-line", "id"],
    // Default DOMPurify URI regex allows http(s)/mailto/tel/sms/cid/xmpp/ftp
    // but blocks custom schemes. Tauri's `convertFileSrc()` returns
    // `asset://localhost/...` URLs, which we need to keep so pasted images
    // render. We also want `file://` to survive so link-click can hand it
    // off to the OS opener.
    ALLOWED_URI_REGEXP:
      /^(?:(?:(?:f|ht)tps?|mailto|tel|callto|sms|cid|xmpp|asset|file):|[^a-z]|[a-z+.\-]+(?:[^a-z+.\-:]|$))/i,
  });
}

