import MarkdownIt from "markdown-it";
import DOMPurify from "dompurify";
import taskLists from "markdown-it-task-lists";

/**
 * Single, reusable markdown-it instance for export. Mirrors the settings of
 * PreviewView so the exported output matches what the user sees on screen.
 */
const md = new MarkdownIt({
  html: true,
  linkify: true,
  breaks: false,
  typographer: true,
});
md.use(taskLists, { enabled: false, label: false });

function escapeAttr(s: string): string {
  return s.replace(/[&<>"']/g, (c) =>
    ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;", "'": "&#39;" })[c]!,
  );
}

/**
 * Minimal print/screen CSS embedded into exported HTML and the PDF iframe.
 * Aims for readability without trying to look fancy — the assumption is that
 * users opening the file in a browser or pasting it into a doc want plain
 * prose styling, not the app's UI chrome.
 */
const EXPORT_CSS = `
  :root { color-scheme: light dark; }
  body {
    font-family: -apple-system, "Segoe UI", Roboto, "Hiragino Sans", "Yu Gothic", sans-serif;
    font-size: 16px;
    line-height: 1.7;
    max-width: 80ch;
    margin: 2em auto;
    padding: 0 1.5em;
    color: #222;
  }
  h1, h2 { border-bottom: 1px solid #eee; padding-bottom: 0.3em; }
  h1 { font-size: 2em; margin: 1.5em 0 0.5em; }
  h2 { font-size: 1.5em; margin: 1.5em 0 0.5em; }
  h3 { font-size: 1.25em; margin: 1.25em 0 0.5em; }
  p { margin: 0.75em 0; }
  code {
    background: #f4f4f4; padding: 0.1em 0.35em; border-radius: 3px;
    font-family: ui-monospace, "SF Mono", Menlo, monospace; font-size: 0.92em;
  }
  pre { background: #f4f4f4; padding: 1em; border-radius: 6px; overflow: auto; }
  pre code { background: transparent; padding: 0; }
  blockquote { margin: 1em 0; padding: 0 1em; border-left: 4px solid #ddd; color: #666; }
  a { color: #0969da; }
  ul, ol { padding-left: 1.5em; }
  li.task-list-item { list-style: none; margin-left: -1.5em; }
  li.task-list-item input { margin-right: 0.5em; vertical-align: middle; }
  table { border-collapse: collapse; margin: 1em 0; }
  th, td { border: 1px solid #ddd; padding: 0.5em 0.8em; }
  img { max-width: 100%; }
  hr { border: 0; border-top: 1px solid #ddd; margin: 1.5em 0; }
  @media print {
    body { max-width: none; margin: 0; padding: 1.5cm; }
    a { color: inherit; text-decoration: none; }
    pre, blockquote, table { page-break-inside: avoid; }
  }
`;

export function renderToHtml(text: string, title: string): string {
  const body = DOMPurify.sanitize(md.render(text));
  return `<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8">
<title>${escapeAttr(title)}</title>
<style>${EXPORT_CSS}</style>
</head>
<body>
${body}
</body>
</html>`;
}

interface Token {
  type: string;
  content: string;
  children?: Token[] | null;
  attrGet?: (name: string) => string | null;
}

/**
 * Walk markdown-it tokens and produce a readable plain-text rendering. Drops
 * inline formatting (bold/italic/code/link markers) but preserves structure
 * (lists with bullets, headings as paragraphs, code blocks verbatim).
 *
 * Round-tripping back to Markdown is *not* a goal — this is for "I just want
 * the prose, no syntax noise".
 */
export function renderToPlainText(text: string): string {
  const tokens = md.parse(text, {}) as Token[];
  const out: string[] = [];
  const listStack: ("bullet" | "ordered")[] = [];
  const orderedCount: number[] = [];

  const inlineText = (tok: Token): string => {
    if (!tok.children) return tok.content;
    let s = "";
    for (const child of tok.children) {
      switch (child.type) {
        case "text":
          s += child.content;
          break;
        case "softbreak":
          s += " ";
          break;
        case "hardbreak":
          s += "\n";
          break;
        case "code_inline":
          s += child.content;
          break;
        case "image":
          s += child.attrGet?.("alt") ?? child.content ?? "";
          break;
        // emphasis / strong / link open-close: nothing emitted, the text children carry through
      }
    }
    return s;
  };

  for (const t of tokens) {
    switch (t.type) {
      case "inline":
        out.push(inlineText(t));
        break;
      case "heading_close":
        out.push("\n\n");
        break;
      case "paragraph_close":
        out.push("\n\n");
        break;
      case "fence":
      case "code_block":
        out.push(t.content);
        if (!t.content.endsWith("\n")) out.push("\n");
        out.push("\n");
        break;
      case "hr":
        out.push("---\n\n");
        break;
      case "bullet_list_open":
        listStack.push("bullet");
        break;
      case "ordered_list_open":
        listStack.push("ordered");
        orderedCount.push(1);
        break;
      case "bullet_list_close":
        listStack.pop();
        out.push("\n");
        break;
      case "ordered_list_close":
        listStack.pop();
        orderedCount.pop();
        out.push("\n");
        break;
      case "list_item_open": {
        const depth = listStack.length - 1;
        const indent = "  ".repeat(depth);
        if (listStack[listStack.length - 1] === "bullet") {
          out.push(`${indent}- `);
        } else {
          const n = orderedCount[orderedCount.length - 1];
          out.push(`${indent}${n}. `);
          orderedCount[orderedCount.length - 1] = n + 1;
        }
        break;
      }
      case "list_item_close":
        // already terminated by paragraph_close emitting \n\n; collapse extras later
        break;
      case "blockquote_close":
        out.push("\n");
        break;
    }
  }

  return out.join("").replace(/\n{3,}/g, "\n\n").trimEnd() + "\n";
}

/**
 * Inject the rendered export HTML into the main document as a print-only
 * container, then call window.print() so the user picks "Save as PDF" in the
 * OS print dialog. @media print hides the rest of the app and reveals only
 * the container. After printing, the container and stylesheet are removed.
 *
 * The earlier iframe-based approach (iframe.contentWindow.print()) is
 * silently ignored by wry / WKWebView under Tauri — the OS print dialog
 * never opens. Printing the main window works because Tauri proxies
 * window.print() through to the OS-level WebView API.
 */
export async function printAsPdf(text: string, title: string): Promise<void> {
  const html = renderToHtml(text, title);

  // Pull just the body markup and the export <style> contents out of the
  // standalone HTML document so we can inline them into the host page.
  const parsed = new DOMParser().parseFromString(html, "text/html");
  const bodyContent = parsed.body.innerHTML;
  const exportStyle = parsed.querySelector("style")?.textContent ?? "";

  const container = document.createElement("div");
  container.id = "mdv-print-container";
  container.innerHTML = bodyContent;

  const styleEl = document.createElement("style");
  styleEl.id = "mdv-print-style";
  // Single @media print block: hide everything else, show the container,
  // override the app's 100vh body so all content paginates. EXPORT_CSS has
  // its own @media print inside; nested @media print resolves the same as
  // a single one per CSS spec.
  styleEl.textContent = `
    #mdv-print-container { display: none; }
    @media print {
      html, body {
        height: auto !important;
        overflow: visible !important;
      }
      body > *:not(#mdv-print-container) { display: none !important; }
      #mdv-print-container { display: block !important; }
      ${exportStyle}
    }
  `;

  document.body.appendChild(container);
  document.head.appendChild(styleEl);

  const savedTitle = document.title;
  document.title = title;

  // Wait for layout / fonts to settle before triggering the print dialog.
  await new Promise((r) => setTimeout(r, 80));

  window.print();

  // Cleanup after the dialog closes. afterprint isn't always reliable
  // (different platforms / headless modes), so back it up with a timer.
  const cleanup = () => {
    if (container.isConnected) container.remove();
    if (styleEl.isConnected) styleEl.remove();
    document.title = savedTitle;
  };
  window.addEventListener("afterprint", cleanup, { once: true });
  setTimeout(cleanup, 60_000);
}

/**
 * Render to a DOCX byte stream via `@turbodocx/html-to-docx`. The library is
 * heavy (~700KB minified including JSZip) so it's loaded lazily — users who
 * never click DOCX never pay the bundle cost.
 */
export async function renderToDocx(text: string, title: string): Promise<Uint8Array> {
  const html = renderToHtml(text, title);
  const mod = await import("@turbodocx/html-to-docx");
  const HTMLtoDOCX = (mod as { default: typeof import("@turbodocx/html-to-docx") }).default ?? mod;
  const result = await (HTMLtoDOCX as unknown as (
    htmlString: string,
    headerHTMLstring?: string | null,
    documentOptions?: { title?: string },
  ) => Promise<ArrayBuffer | Blob | Uint8Array>)(html, null, { title });

  if (result instanceof Blob) {
    return new Uint8Array(await result.arrayBuffer());
  }
  if (result instanceof Uint8Array) {
    return result;
  }
  return new Uint8Array(result as ArrayBuffer);
}
