import { convertFileSrc } from "@tauri-apps/api/core";

/**
 * Path-resolution helpers shared by the markdown-it image renderer (Preview /
 * Diff) and the WYSIWYG / Live Preview DOM rewriter. Pulled out so the two
 * pipelines apply exactly the same rules — `asset://` scheme via Tauri's
 * `convertFileSrc()` for relative paths, untouched for absolute / URL-y src.
 */

/**
 * Is `src` a relative path (no scheme, no leading slash, no Windows drive)?
 * `http:`, `https:`, `data:`, `asset:`, `file:` etc. return false.
 * `/Users/x.png` and `C:\foo.png` return false (already absolute).
 */
export function isRelativePath(src: string): boolean {
  if (/^[a-z][a-z0-9+.-]*:/i.test(src)) return false;
  if (src.startsWith("/")) return false;
  if (/^[A-Za-z]:[\\/]/.test(src)) return false;
  return true;
}

/**
 * Resolve `rel` against the directory of `docPath`. Forward-slash joiner is
 * fine even on Windows — `convertFileSrc` accepts mixed separators and
 * Markdown link paths typically use `/`.
 */
export function resolveRelativeToDoc(rel: string, docPath: string): string {
  const parts = docPath.split(/[\\/]/);
  parts.pop(); // remove filename
  return `${parts.join("/")}/${rel}`;
}

/**
 * If `src` is a relative path AND we have an anchor (`docPath`), convert to
 * Tauri's `asset://` URL. Otherwise return `src` unchanged. Safe to call
 * repeatedly on the same string — already-resolved `asset://...` URLs aren't
 * relative anymore and pass through.
 */
export function rewriteRelativeImageSrc(
  src: string,
  docPath: string | null,
): string {
  if (!docPath || !isRelativePath(src)) return src;
  return convertFileSrc(resolveRelativeToDoc(src, docPath));
}
