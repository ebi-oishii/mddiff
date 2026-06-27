import { invoke } from "@tauri-apps/api/core";
import type { BaseOption, DiffLine, HunkSummary } from "$lib/types";

export async function gitIsRepo(path: string): Promise<boolean> {
  return await invoke<boolean>("git_is_repo", { path });
}

export async function gitListBases(
  path: string,
  currentText?: string,
): Promise<BaseOption[]> {
  return await invoke<BaseOption[]>("git_list_bases", { path, currentText });
}

export async function gitHunks(
  path: string,
  currentText: string,
  base?: string,
): Promise<HunkSummary[]> {
  return await invoke<HunkSummary[]>("git_hunks", { path, currentText, base });
}

export async function gitFullDiff(
  path: string,
  currentText: string,
  base?: string,
): Promise<DiffLine[]> {
  return await invoke<DiffLine[]>("git_full_diff", { path, currentText, base });
}
