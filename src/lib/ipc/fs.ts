import { invoke } from "@tauri-apps/api/core";
import { open as openDialog, save as saveDialog } from "@tauri-apps/plugin-dialog";
import { gitIsRepo } from "./git";
import { mdvExtractBody } from "./mdv";

const MD_FILTER = { name: "Markdown", extensions: ["md", "markdown", "mdv"] };

/** Mirror of mdv_core::fs::MAX_OPEN_BYTES. Files above this size trigger
 * the large-file warning modal. Keep in sync with Rust. */
export const LARGE_FILE_BYTES = 5 * 1024 * 1024;
/** Mirror of mdv_core::fs::HARD_CAP_BYTES. Files above this size are refused
 * outright — even `force=true` won't open them. */
export const HARD_CAP_BYTES = 100 * 1024 * 1024;

export type LoadedFile = { path: string; text: string; gitAvailable: boolean };

/** Open the OS file picker and return the chosen path (or null). The actual
 * read is split out so the caller can prompt for confirmation between
 * picking and reading. */
export async function pickFile(): Promise<string | null> {
  const selected = await openDialog({ multiple: false, filters: [MD_FILTER] });
  return typeof selected === "string" ? selected : null;
}

/** Read the file's content and produce a LoadedFile. Pass `force=true` after
 * the user has confirmed the large-file warning. */
export async function readFile(path: string, force = false): Promise<LoadedFile> {
  const raw = await invoke<string>("read_text_file", { path, force });
  // `.mdv` files carry a trailing `<!-- mdv:v1 ... -->` package block. Strip it
  // before handing to the editor so the user sees plain Markdown. The history
  // bundle is intentionally discarded — per the design, import doesn't merge
  // back into Git.
  const text = path.toLowerCase().endsWith(".mdv")
    ? await mdvExtractBody(raw)
    : raw;
  const gitAvailable = await gitIsRepo(path);
  return { path, text, gitAvailable };
}

export async function getFileSize(path: string): Promise<number> {
  return invoke<number>("file_size", { path });
}

export async function pickAndWriteFile(text: string): Promise<string | null> {
  const path = await saveDialog({ filters: [MD_FILTER] });
  if (!path) return null;
  await invoke("write_text_file", { path, content: text });
  return path;
}

export async function writeFile(path: string, text: string): Promise<void> {
  await invoke("write_text_file", { path, content: text });
}

/**
 * Read a file's text content without going through the open-dialog flow.
 * Used to reload from disk when the file changed externally.
 */
export async function readText(path: string): Promise<string> {
  return invoke<string>("read_text_file", { path });
}

/**
 * Start watching `path` for external changes. Replaces any previous watcher.
 * The Rust side emits a `file-external-change` event when changes are seen.
 */
export async function startWatch(path: string): Promise<void> {
  await invoke("start_watch", { path });
}

export async function stopWatch(): Promise<void> {
  await invoke("stop_watch");
}

export type ExternalChange = {
  path: string;
  kind: "modified" | "removed";
};

/**
 * Chunked Uint8Array → base64 so we don't blow the call stack on large arrays
 * (String.fromCharCode(...arr) is bounded by argument count limits).
 */
function uint8ToBase64(bytes: Uint8Array): string {
  let binary = "";
  const chunk = 0x8000;
  for (let i = 0; i < bytes.length; i += chunk) {
    binary += String.fromCharCode(...bytes.subarray(i, i + chunk));
  }
  return btoa(binary);
}

export async function writeBinaryFile(
  path: string,
  bytes: Uint8Array,
): Promise<void> {
  await invoke("write_binary_file", { path, base64: uint8ToBase64(bytes) });
}

/**
 * Open a Save As dialog for an arbitrary extension. Suggests `defaultPath`
 * derived from the currently open file (or "untitled") with the extension
 * swapped, so users can typically just hit Enter.
 */
export async function pickSavePath(
  ext: string,
  label: string,
  currentPath: string | null,
): Promise<string | null> {
  const defaultPath = currentPath
    ? currentPath.replace(/\.[^./\\]*$/, "") + "." + ext
    : "untitled." + ext;
  const path = await saveDialog({
    defaultPath,
    filters: [{ name: label, extensions: [ext] }],
  });
  return typeof path === "string" ? path : null;
}
