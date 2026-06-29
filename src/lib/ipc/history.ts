import { invoke } from "@tauri-apps/api/core";
import type { SnapshotMeta } from "$lib/types";

/**
 * Local-only "save-event" history. Each successful save can drop a
 * zstd-compressed snapshot under the per-app data dir, keyed by the file's
 * absolute path. These never touch Git — they're a finer-grained safety net
 * in between commits.
 *
 * Mobile note: the Tauri commands are desktop-only, so we swallow errors and
 * return empty/no-op rather than throwing into the UI.
 */

export async function snapshotSave(
  path: string,
  content: string,
): Promise<SnapshotMeta | null> {
  try {
    return await invoke<SnapshotMeta | null>("snapshot_save", { path, content });
  } catch {
    return null;
  }
}

export async function snapshotList(path: string): Promise<SnapshotMeta[]> {
  try {
    return await invoke<SnapshotMeta[]>("snapshot_list", { path });
  } catch {
    return [];
  }
}

export async function snapshotRead(path: string, id: string): Promise<string> {
  return await invoke<string>("snapshot_read", { path, id });
}
