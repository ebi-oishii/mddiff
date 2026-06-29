import { invoke } from "@tauri-apps/api/core";

export type PastedImage = {
  /** Path relative to the document directory, ready for a Markdown link. */
  rel_path: string;
  /** Absolute path on disk. */
  abs_path: string;
};

/**
 * If the system clipboard contains an image, encode it as PNG and write it
 * to `<doc-stem>.assets/<timestamp>.png` next to the open document. Returns
 * the relative path to insert into a Markdown link, or `null` when the
 * clipboard had no image (in which case the caller should fall through to
 * normal text paste behavior).
 *
 * Errors (permission, missing parent dir, disk full) bubble as rejected
 * promises so the caller can surface them via the standard error banner.
 */
export async function pasteImage(docPath: string): Promise<PastedImage | null> {
  return invoke<PastedImage | null>("paste_image", { docPath });
}
