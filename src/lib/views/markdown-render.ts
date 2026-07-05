import MarkdownIt from "markdown-it";
import type Token from "markdown-it/lib/token.mjs";
import DOMPurify from "dompurify";
import taskLists from "markdown-it-task-lists";
import anchor from "markdown-it-anchor";
import katex from "@vscode/markdown-it-katex";
import { rewriteRelativeImageSrc } from "./image-path";
import { hljs } from "./highlight.svelte";
// KaTeX ships its own stylesheet that positions the rendered math and
// declares @font-face for the KaTeX_* fonts. Importing it as a side
// effect lets Vite bundle the CSS + the WOFF/WOFF2 font files with the
// rest of the app assets — no manual `<link>` or font copying needed.
import "katex/dist/katex.min.css";

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
  // Explicit type annotation is required because `highlight` below closes
  // over `md` to reach `md.utils.escapeHtml`, and TS would otherwise fail
  // to infer the type due to the circular self-reference.
  const md: MarkdownIt = new MarkdownIt({
    html: true,
    linkify: true,
    breaks: false,
    typographer: true,
    // Fenced code blocks with a language tag get run through highlight.js.
    // Unknown languages (or missing tag) fall back to escaped plain text
    // wrapped in the same `.hljs` class so the theme's background /
    // foreground still apply — this way every code block gets consistent
    // styling regardless of language coverage.
    highlight: (str, lang): string => {
      if (lang && hljs.getLanguage(lang)) {
        try {
          return (
            '<pre class="hljs"><code>' +
            hljs.highlight(str, { language: lang, ignoreIllegals: true }).value +
            "</code></pre>"
          );
        } catch {
          // Fall through to plain rendering on any parser error.
        }
      }
      return (
        '<pre class="hljs"><code>' + md.utils.escapeHtml(str) + "</code></pre>"
      );
    },
  });
  md.use(taskLists, { enabled: false, label: false });
  // Auto-generate `id="..."` slugs on headings so `[link](#installation)`
  // anchor jumps work without manual id markup. Default GFM-style slugifier
  // matches what most users expect (`Installation` → `installation`).
  md.use(anchor, { permalink: false });
  // LaTeX math: `$...$` inline, `$$...$$` block. `\begin{env}...\end{env}`
  // outside of `$$` needs `enableBareBlocks: true` to be recognized; leave
  // it off for now to keep dollar-heavy prose (currency etc.) matching
  // upstream markdown semantics — users can escape with `\$` when needed.
  md.use(katex, {
    throwOnError: false,
    // Emit an error node inline rather than aborting the render for
    // syntactically invalid math. Matches the mermaid error UX.
    errorColor: "#a02020",
  });

  // Intercept ```mermaid ... ``` fences before the highlight option would
  // emit a plain-text fallback. We swap in a placeholder div carrying
  // the raw source in a data-* attribute; the async mermaid module (see
  // `mermaid.svelte.ts`) walks the rendered DOM later and replaces
  // placeholders with the rendered SVG. Doing the swap here (rather
  // than post-processing the whole HTML) keeps the pipeline single-pass
  // and lets the sanitizer see (and preserve) the placeholder attributes.
  const defaultFence = md.renderer.rules.fence;
  md.renderer.rules.fence = function (tokens, idx, options, env, self) {
    const info = tokens[idx].info.trim().toLowerCase();
    if (info === "mermaid") {
      const source = md.utils.escapeHtml(tokens[idx].content);
      return `<div class="mddiff-mermaid" data-mermaid-source="${source}"></div>`;
    }
    return defaultFence
      ? defaultFence(tokens, idx, options, env, self)
      : self.renderToken(tokens, idx, options);
  };

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
    // is our own attribute for scroll sync. data-mermaid-source carries
    // the raw diagram body for the client-side renderer to pick up.
    ADD_ATTR: ["data-mddiff-line", "data-mermaid-source", "id"],
    // Default DOMPurify URI regex allows http(s)/mailto/tel/sms/cid/xmpp/ftp
    // but blocks custom schemes. Tauri's `convertFileSrc()` returns
    // `asset://localhost/...` URLs, which we need to keep so pasted images
    // render. We also want `file://` to survive so link-click can hand it
    // off to the OS opener.
    ALLOWED_URI_REGEXP:
      /^(?:(?:(?:f|ht)tps?|mailto|tel|callto|sms|cid|xmpp|asset|file):|[^a-z]|[a-z+.\-]+(?:[^a-z+.\-:]|$))/i,
    // Also allow the MathML profile so KaTeX's <math>...<mrow>... output
    // survives sanitization. KaTeX also emits an SVG fallback for legacy
    // browsers; keep the SVG profile enabled too.
    USE_PROFILES: { html: true, mathMl: true, svg: true },
  });
}

