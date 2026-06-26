export type Mode = "source" | "preview" | "diff";

export type HunkKind = "added" | "modified" | "removed";

export interface HunkSummary {
  kind: HunkKind;
  start_line: number;
  end_line: number;
  removed_count: number;
}

export type DiffLine =
  | { kind: "equal"; old_no: number; new_no: number; text: string }
  | { kind: "added"; new_no: number; text: string }
  | { kind: "removed"; old_no: number; text: string };

export type DiffSubmode = "highlight" | "full";

export type BaseKind = "special" | "branch" | "tag" | "commit";

export interface BaseOption {
  revspec: string;
  label: string;
  kind: BaseKind;
  detail: string | null;
}
