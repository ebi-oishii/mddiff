import { invoke } from "@tauri-apps/api/core";

export interface PackResponse {
  content: string;
  commit_count: number;
  snapshot_count: number;
  bundle_bytes: number;
}

export async function mddiffPack(
  path: string,
  currentText: string,
  base: string,
): Promise<PackResponse> {
  return await invoke<PackResponse>("mddiff_pack", {
    path,
    currentText,
    base,
  });
}

/**
 * Strip the `<!-- mddiff:v1 ... -->` package block from a `.mddiff` file's content
 * and return the markdown body. Returns the input unchanged when no block is
 * present, so it's safe to call on any text.
 */
export async function mddiffExtractBody(content: string): Promise<string> {
  return await invoke<string>("mddiff_extract_body", { content });
}
