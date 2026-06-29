import { openUrl } from "@tauri-apps/plugin-opener";
import { WebviewWindow } from "@tauri-apps/api/webviewWindow";
import { resolveRelativeToDoc, isRelativePath } from "./image-path";

/**
 * Markdown text-file extensions that mddiff itself can open. Anything not in
 * this set falls through to the OS opener for relative paths.
 */
const TEXT_EXTENSIONS = new Set([
  "md",
  "markdown",
  "mdv",
  "mddiff",
  "txt",
  "rst",
]);

export type LinkClickContext = {
  /** Current document path (`null` for the untitled buffer). */
  getDocPath: () => string | null;
  /**
   * Optional hook for in-view anchor scrolling. Receives the slug (after the
   * `#`). When omitted the click is treated like any other no-scheme path.
   */
  onScrollAnchor?: (id: string) => void;
  /**
   * When `true`, only handle clicks where the platform's primary modifier
   * (⌘ on macOS, Ctrl elsewhere) is held. Use this for editable / selectable
   * views (Live Preview, WYSIWYG, Diff Side-by-Side) so plain clicks still
   * position the cursor / select text. Read-only views (Preview) leave this
   * `false` so any click opens the link.
   */
  requireModifier?: boolean;
};

export function modifierPressed(event: MouseEvent | KeyboardEvent): boolean {
  // Don't try to detect platform — accept either modifier. ⌘ on Mac,
  // Ctrl on Win/Linux. Pressing the "wrong" modifier still opens the link,
  // which is a forgiving fallback rather than a footgun.
  return event.metaKey || event.ctrlKey;
}

/**
 * Apply the same dispatch table as {@link handleLinkClick} but starting from
 * an already-extracted href. Used by the CodeMirror extension (Source / Live
 * Preview) where there's no `<a>` element to crawl — only a syntax-tree
 * Link node and a doc-text slice.
 *
 * Caller is responsible for preventDefault (or whatever editor-event suppression
 * makes sense).
 */
export function routeHref(href: string, ctx: LinkClickContext): void {
  if (href.startsWith("#")) {
    ctx.onScrollAnchor?.(href.slice(1));
    return;
  }

  const schemeMatch = href.match(/^([a-z][a-z0-9+.-]*):/i);
  if (schemeMatch) {
    const scheme = schemeMatch[1].toLowerCase();
    if (scheme === "asset") return; // internal images — nothing to do

    if (scheme === "file") {
      const decoded = decodeFileUrl(href);
      if (decoded && isTextFile(decoded)) {
        void openInNewWindow(decoded);
      } else {
        void openUrl(href).catch((e) =>
          console.error("[link-click] opener failed:", e),
        );
      }
      return;
    }

    void openUrl(href).catch((e) =>
      console.error("[link-click] opener failed:", e),
    );
    return;
  }

  if (!isRelativePath(href)) {
    void openUrl(`file://${href}`).catch((e) =>
      console.error("[link-click] opener failed:", e),
    );
    return;
  }

  const docPath = ctx.getDocPath();
  if (!docPath) return;
  const abs = resolveRelativeToDoc(href, docPath);
  if (isTextFile(abs)) {
    void openInNewWindow(abs);
  } else {
    void openUrl(`file://${abs}`).catch((e) =>
      console.error("[link-click] opener failed:", e),
    );
  }
}

/**
 * Decide what to do for an `<a>` click and execute it. Returns `true` if the
 * event was handled (caller should treat the click as consumed), `false` if
 * the caller should let the default behavior run.
 *
 * Dispatch table:
 *   - `#anchor`        → in-view scroll (delegate to onScrollAnchor)
 *   - `asset:` URLs    → leave alone (used for inline images)
 *   - `http(s)`, `mailto`, `tel`, `sms`, `ftp`, ...  → OS opener
 *   - `file://` to text → mddiff new window
 *   - `file://` other   → OS opener
 *   - relative path to text → mddiff new window (anchored at the open doc)
 *   - relative path other   → OS opener via `file://<abs>`
 *
 * Errors from the opener / WebviewWindow constructor are swallowed and
 * logged: a failed link shouldn't crash the surrounding view. The user gets
 * the silent fall-through and can retry.
 */
export function handleLinkClick(
  event: MouseEvent,
  ctx: LinkClickContext,
): boolean {
  const target = event.target as HTMLElement | null;
  if (!target) return false;
  const anchor = target.closest("a");
  if (!anchor) return false;
  const href = anchor.getAttribute("href");
  if (!href) return false;

  // Editable views require ⌘/Ctrl to navigate so a plain click can still
  // place the cursor. Without the modifier, fall through to the host's
  // default click behavior (cursor positioning).
  if (ctx.requireModifier && !modifierPressed(event)) return false;

  // asset: URLs are internal (images) — let the webview handle them so the
  // image preview / WYSIWYG embed still loads. routeHref also short-circuits
  // on asset:, but we need to skip the preventDefault below.
  if (href.startsWith("asset:")) return false;

  event.preventDefault();
  routeHref(href, ctx);
  return true;
}

function isTextFile(path: string): boolean {
  const last = path.split(/[?#]/)[0];
  const ext = last.split(".").pop()?.toLowerCase();
  return TEXT_EXTENSIONS.has(ext ?? "");
}

/** `file:///Users/foo/x.md` → `/Users/foo/x.md`. */
function decodeFileUrl(href: string): string | null {
  // Strip leading `file://` (and the optional triple slash on Unix).
  const m = href.match(/^file:\/\/(\/[^?#]*)/);
  if (!m) return null;
  try {
    return decodeURIComponent(m[1]);
  } catch {
    return null;
  }
}

/**
 * Spawn a new mddiff window pre-targeted at `absPath`. The new window's
 * +page.svelte reads `?file=` from its URL on mount and routes it through
 * the normal open flow (so large-file warning, watcher, etc. all kick in).
 *
 * Window label format: `doc-<timestamp>`. Capability config matches this
 * via the `doc-*` glob in capabilities/default.json.
 */
async function openInNewWindow(absPath: string): Promise<void> {
  const label = `doc-${Date.now()}`;
  const url = `index.html?file=${encodeURIComponent(absPath)}`;
  try {
    const win = new WebviewWindow(label, {
      url,
      title: basename(absPath),
      width: 1024,
      height: 768,
    });
    win.once("tauri://error", (e) => {
      console.error("[link-click] new window failed:", e);
    });
  } catch (e) {
    console.error("[link-click] new window threw:", e);
  }
}

function basename(p: string): string {
  return p.split(/[/\\]/).pop() || p;
}
