import type { HunkSummary } from "$lib/types";

/**
 * Map a source line in the OLD pane to the corresponding line in NEW, using
 * the HunkSummary list. Unchanged regions map identity-plus-running-offset;
 * lines inside a hunk map to the hunk's corresponding range (proportional
 * for Modified, to the anchor for Removed).
 */
export function mapOldToNew(oldLine: number, hunks: HunkSummary[]): number {
  if (oldLine < 1) return 1;
  const sorted = [...hunks].sort((a, b) => oldAnchor(a) - oldAnchor(b));

  let delta = 0;
  for (const h of sorted) {
    if (h.kind === "added") {
      // old_start == old_end is the "after this OLD line" anchor; additions
      // sit between that line and the next one.
      if (oldLine <= h.old_start) break;
      delta += h.new_end - h.new_start + 1;
    } else if (h.kind === "removed") {
      if (oldLine < h.old_start) break;
      if (oldLine <= h.old_end) {
        // Deleted line — point at the NEW anchor (line before/at the gap).
        return Math.max(1, h.new_start);
      }
      delta -= h.old_end - h.old_start + 1;
    } else {
      // modified
      if (oldLine < h.old_start) break;
      if (oldLine <= h.old_end) {
        const oldSpan = h.old_end - h.old_start;
        const newSpan = h.new_end - h.new_start;
        if (oldSpan === 0) return h.new_start;
        const frac = (oldLine - h.old_start) / oldSpan;
        return Math.round(h.new_start + frac * newSpan);
      }
      delta += h.new_end - h.new_start - (h.old_end - h.old_start);
    }
  }
  return Math.max(1, oldLine + delta);
}

/**
 * Mirror of `mapOldToNew`, going the other direction.
 */
export function mapNewToOld(newLine: number, hunks: HunkSummary[]): number {
  if (newLine < 1) return 1;
  const sorted = [...hunks].sort((a, b) => newAnchor(a) - newAnchor(b));

  let delta = 0;
  for (const h of sorted) {
    if (h.kind === "removed") {
      if (newLine <= h.new_start) break;
      delta += h.old_end - h.old_start + 1;
    } else if (h.kind === "added") {
      if (newLine < h.new_start) break;
      if (newLine <= h.new_end) {
        return Math.max(1, h.old_start);
      }
      delta -= h.new_end - h.new_start + 1;
    } else {
      // modified
      if (newLine < h.new_start) break;
      if (newLine <= h.new_end) {
        const newSpan = h.new_end - h.new_start;
        const oldSpan = h.old_end - h.old_start;
        if (newSpan === 0) return h.old_start;
        const frac = (newLine - h.new_start) / newSpan;
        return Math.round(h.old_start + frac * oldSpan);
      }
      delta += h.old_end - h.old_start - (h.new_end - h.new_start);
    }
  }
  return Math.max(1, newLine + delta);
}

function oldAnchor(h: HunkSummary): number {
  // Sort key on the OLD side. Added hunks sit "between" old_start and the
  // next line, so add 0.5 to order them after lines at old_start.
  return h.kind === "added" ? h.old_start + 0.5 : h.old_start;
}

function newAnchor(h: HunkSummary): number {
  return h.kind === "removed" ? h.new_start + 0.5 : h.new_start;
}
