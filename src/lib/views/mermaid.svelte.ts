/**
 * Mermaid diagram rendering for Preview and Diff SBS views.
 *
 * We keep the mermaid module out of the main bundle (it's ~3.4 MB min /
 * ~950 KB gzip — big enough to notice on cold starts) by loading it
 * only when a diagram actually needs to render. The public API is a
 * single `renderPlaceholders(root)` call — the caller (usually a view's
 * `$effect` on rendered HTML) hands us a scroller and we find any
 * `.mddiff-mermaid` placeholders the markdown-it fence rule left
 * behind, then swap them with the rendered SVG in place.
 *
 * Placeholders carry their raw source in `data-mermaid-source`. We use
 * an attribute rather than the element's text content so downstream
 * sanitizers can't accidentally clobber diagram source with escape
 * transformations. The value comes from `element.getAttribute()`, which
 * decodes HTML entities for us, so mermaid receives the exact bytes
 * the author wrote.
 */

// A single lazy-loading promise so concurrent callers don't each pay
// the module-download cost. Resolved with the initialized mermaid
// default export.
let mermaidPromise: Promise<typeof import("mermaid").default> | null = null;

async function getMermaid() {
  if (!mermaidPromise) {
    mermaidPromise = import("mermaid").then((mod) => {
      // `startOnLoad: false` because we drive rendering ourselves — mermaid's
      // built-in auto-render walks the whole document, which we don't want
      // in a SPA where the DOM is constantly re-rendered.
      // `securityLevel: 'strict'` escapes any HTML in labels so authored
      // markdown can't XSS through the diagram body.
      mod.default.initialize({
        startOnLoad: false,
        theme: "default",
        securityLevel: "strict",
        fontFamily:
          '-apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif',
      });
      return mod.default;
    });
  }
  return mermaidPromise;
}

// Ids need to be unique per-render because mermaid uses them to build
// the SVG element's id and internal graph references. A monotonic
// counter is enough — collisions across renders inside the same page
// would corrupt the DOM.
let idCounter = 0;

/**
 * Find and render every mermaid placeholder under `root` that hasn't been
 * rendered yet. Idempotent — placeholders receive a `mddiff-mermaid-rendered`
 * (or `-error`) class after processing so repeat calls skip them.
 *
 * Errors are surfaced inline as a small red `<pre>` inside the same
 * placeholder rather than thrown; a single busted diagram shouldn't wreck
 * the whole preview.
 */
export async function renderPlaceholders(root: HTMLElement | null): Promise<void> {
  if (!root) return;
  const placeholders = root.querySelectorAll<HTMLElement>(
    ".mddiff-mermaid:not(.mddiff-mermaid-rendered):not(.mddiff-mermaid-error)",
  );
  if (placeholders.length === 0) return;
  const mermaid = await getMermaid();
  for (const el of placeholders) {
    const source = el.getAttribute("data-mermaid-source") ?? "";
    const id = `mddiff-mermaid-${++idCounter}`;
    try {
      const { svg } = await mermaid.render(id, source);
      el.innerHTML = svg;
      el.classList.add("mddiff-mermaid-rendered");
    } catch (err) {
      el.innerHTML =
        '<pre class="mddiff-mermaid-error-msg">' +
        escapeHtml(String(err instanceof Error ? err.message : err)) +
        "</pre>";
      el.classList.add("mddiff-mermaid-error");
    }
  }
}

function escapeHtml(s: string): string {
  return s
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;");
}
