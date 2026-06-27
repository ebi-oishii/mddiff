import { invoke } from "@tauri-apps/api/core";
import { open as openDialog, save as saveDialog } from "@tauri-apps/plugin-dialog";
import { gitIsRepo } from "./git";

const MD_FILTER = { name: "Markdown", extensions: ["md", "markdown"] };

export type LoadedFile = { path: string; text: string; gitAvailable: boolean };

export async function pickAndReadFile(): Promise<LoadedFile | null> {
  const selected = await openDialog({ multiple: false, filters: [MD_FILTER] });
  if (typeof selected !== "string") return null;
  const text = await invoke<string>("read_text_file", { path: selected });
  const gitAvailable = await gitIsRepo(selected);
  return { path: selected, text, gitAvailable };
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
