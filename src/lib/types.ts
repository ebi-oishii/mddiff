export type Mode = "source" | "live" | "wysiwyg" | "preview" | "diff";

export type HunkKind = "added" | "modified" | "removed";

/**
 * Change descriptor carrying line ranges on both the OLD and NEW sides.
 * All line numbers are 1-based. A `point` (anchor) is encoded as `start ===
 * end` (the line immediately before insertion / deletion, or 0 for top).
 *
 * - `added`    : `new_*` spans inserted lines; `old_start === old_end` is the
 *                anchor where they would appear in OLD.
 * - `removed`  : `old_*` spans deleted lines; `new_start === new_end` is the
 *                anchor where the deletion occurred in NEW.
 * - `modified` : both `new_*` and `old_*` span the replaced lines.
 */
export interface HunkSummary {
  kind: HunkKind;
  new_start: number;
  new_end: number;
  old_start: number;
  old_end: number;
}

export function removedCount(h: HunkSummary): number {
  if (h.kind === "added") return 0;
  return h.old_end - h.old_start + 1;
}

export type DiffLine =
  | { kind: "equal"; old_no: number; new_no: number; text: string }
  | { kind: "added"; new_no: number; text: string }
  | { kind: "removed"; old_no: number; text: string };

export type DiffSubmode = "highlight" | "full" | "sidebyside";

export interface SideBySidePayload {
  old_text: string;
  new_text: string;
  hunks: HunkSummary[];
}

export type BaseKind = "special" | "branch" | "tag" | "commit";

/**
 * - `differs`: file at this revision differs from current buffer AND is the
 *   canonical entry of its same-content run
 * - `identical`: file at this revision is byte-identical to current buffer
 * - `redundant`: a more recent commit shown above has the same content;
 *   choosing this gives the same diff, so the marker is suppressed
 * - `unknown`: couldn't determine (rare)
 */
export type DiffMarker = "differs" | "identical" | "redundant" | "unknown";

export interface BaseOption {
  revspec: string;
  label: string;
  kind: BaseKind;
  detail: string | null;
  marker: DiffMarker;
  /**
   * For Commit kind: did this commit actually change the file (blob differs
   * from its parent)? Special / Branch / Tag are always `true` so they're
   * never filtered out of the default picker.
   */
  file_changed: boolean;
}
