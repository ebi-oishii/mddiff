import { invoke } from "@tauri-apps/api/core";
import { open as openDialog, save as saveDialog } from "@tauri-apps/plugin-dialog";

const MD_FILTER = { name: "Markdown", extensions: ["md", "markdown"] };

export type LoadedFile = { path: string; text: string };

export async function pickAndReadFile(): Promise<LoadedFile | null> {
  const selected = await openDialog({ multiple: false, filters: [MD_FILTER] });
  if (typeof selected !== "string") return null;
  const text = await invoke<string>("read_text_file", { path: selected });
  return { path: selected, text };
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
