import { invoke } from "@tauri-apps/api/core";
import type {
  BaseOption,
  DiffLine,
  HunkSummary,
  SideBySidePayload,
} from "$lib/types";

/**
 * Returns whether `path` lives in a Git repository.
 * On mobile builds the underlying Tauri command isn't compiled in, so we
 * swallow the resulting "command not found" error and return false so the
 * Diff tab degrades gracefully.
 */
export async function gitIsRepo(path: string): Promise<boolean> {
  try {
    return await invoke<boolean>("git_is_repo", { path });
  } catch {
    return false;
  }
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

export async function gitSideBySide(
  path: string,
  currentText: string,
  base?: string,
): Promise<SideBySidePayload> {
  return await invoke<SideBySidePayload>("git_side_by_side", {
    path,
    currentText,
    base,
  });
}

/**
 * Pure text-vs-text diffs. Used by the "Compare with disk" path which
 * doesn't go through Git.
 */
export async function diffTextHunks(
  oldText: string,
  newText: string,
): Promise<HunkSummary[]> {
  return await invoke<HunkSummary[]>("diff_text_hunks", { oldText, newText });
}

export async function diffTextFull(
  oldText: string,
  newText: string,
): Promise<DiffLine[]> {
  return await invoke<DiffLine[]>("diff_text_full", { oldText, newText });
}

export async function diffTextSideBySide(
  oldText: string,
  newText: string,
): Promise<SideBySidePayload> {
  return await invoke<SideBySidePayload>("diff_text_side_by_side", {
    oldText,
    newText,
  });
}
