/**
 * Read-only "view this file at an older commit" mode. While `history` is
 * non-null, the active view renders `history.content` (instead of `doc.text`)
 * and editing is suppressed at the view level. `commits` + `index` drive the
 * Prev/Next walk through eligible revisions; `previousMode` is what we should
 * return to once the user exits history mode.
 */
export type HistoryView = {
  revspec: string;
  label: string;
  content: string;
  commits: { revspec: string; label: string }[];
  index: number;
};

class DocStore {
  text = $state("");
  path = $state<string | null>(null);
  savedText = $state("");
  gitAvailable = $state(false);

  history = $state<HistoryView | null>(null);

  /** Captured disk text for the "Compare with disk" action triggered from
   * the external-change banner. DiffView reads this when the user picks the
   * disk base option. */
  pendingDiskCompare = $state<string | null>(null);

  /**
   * Topmost visible source line (1-based). Views write to this on unmount and
   * read from it on mount so switching modes keeps the user at the same place.
   * View implementations differ:
   *   - SourceView / LivePreviewView: CodeMirror posAtCoords / scrollIntoView
   *   - PreviewView: `[data-mddiff-line]` attributes injected via markdown-it
   *   - WysiwygView: ProseMirror top-level children indexed against
   *     markdown-it top-level block tokens (Milkdown has no native source map)
   */
  currentLine = $state(1);

  /**
   * One-shot signal for the outline sidebar (and future TOC consumers) to
   * tell the active view "scroll to this line now". Each view subscribes via
   * `$effect`, performs its native scroll-to-line, and resets the signal
   * back to `null`. Distinct from `currentLine` (which is OUTPUT of scroll
   * tracking) so we don't fight the scroll tracker.
   */
  pendingScrollLine = $state<number | null>(null);

  /** Request the active view to scroll to a 1-based source line. */
  jumpToLine(line: number) {
    this.pendingScrollLine = line;
  }

  /**
   * Enter history view at the given commit position. The caller has already
   * resolved the file content at that commit; we just install it.
   */
  enterHistory(
    revspec: string,
    label: string,
    content: string,
    commits: { revspec: string; label: string }[],
    index: number,
  ) {
    this.history = { revspec, label, content, commits, index };
  }

  /** Replace the content shown by history view (for Prev/Next walk). */
  updateHistoryContent(index: number, revspec: string, label: string, content: string) {
    if (!this.history) return;
    this.history = { ...this.history, index, revspec, label, content };
  }

  /** Restore the historical content into the live buffer as an edit and exit. */
  restoreHistory() {
    if (!this.history) return;
    this.text = this.history.content;
    this.history = null;
  }

  exitHistory() {
    this.history = null;
  }

  get dirty() {
    return this.text !== this.savedText;
  }

  setText(t: string) {
    this.text = t;
  }

  load(path: string | null, text: string, gitAvailable: boolean) {
    this.path = path;
    this.text = text;
    this.savedText = text;
    this.gitAvailable = gitAvailable;
    this.currentLine = 1;
    this.pendingDiskCompare = null;
    this.history = null;
  }

  /**
   * Swap the buffer to a disk-read copy without resetting view state.
   * Used by external-change auto-reload and the [Revert to disk] banner
   * action — same content as `load()` would have given, but we keep the
   * scroll position (currentLine) so the user isn't ejected to line 1.
   */
  reloadFromDisk(text: string) {
    this.text = text;
    this.savedText = text;
  }

  /**
   * After Save As, the file now lives at a new path with possibly different
   * Git status. Buffer contents are unchanged — `markSaved()` is the
   * companion that snapshots them as clean.
   */
  setPath(path: string, gitAvailable: boolean) {
    this.path = path;
    this.gitAvailable = gitAvailable;
  }

  markSaved() {
    this.savedText = this.text;
  }

  /**
   * Called when WYSIWYG (Milkdown) round-trips the document on mount and
   * yields a normalized form (e.g. `*foo*` -> `_foo_`, link reference
   * expansion). If the user had no unsaved edits at that point, we treat the
   * normalized form as the new baseline so the dirty indicator doesn't
   * appear out of nowhere just from opening WYSIWYG. If they already had
   * unsaved edits, preserve dirty — the normalization is part of those edits.
   *
   * Side effect: `savedText` may diverge from the bytes actually on disk in
   * the no-prior-edits case. We accept that trade-off so the UX makes sense.
   */
  adoptNormalized(normalized: string) {
    if (this.text === this.savedText) {
      this.text = normalized;
      this.savedText = normalized;
    } else {
      this.text = normalized;
    }
  }
}

export const doc = new DocStore();
