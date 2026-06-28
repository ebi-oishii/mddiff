class DocStore {
  text = $state("");
  path = $state<string | null>(null);
  savedText = $state("");
  gitAvailable = $state(false);

  /** Captured disk text for the "Compare with disk" action triggered from
   * the external-change banner. DiffView reads this when the user picks the
   * disk base option. */
  pendingDiskCompare = $state<string | null>(null);

  /**
   * Topmost visible source line (1-based). Views write to this on unmount and
   * read from it on mount so switching modes keeps the user at the same place.
   * View implementations differ:
   *   - SourceView / LivePreviewView: CodeMirror posAtCoords / scrollIntoView
   *   - PreviewView: `[data-mdv-line]` attributes injected via markdown-it
   *   - WysiwygView: ProseMirror top-level children indexed against
   *     markdown-it top-level block tokens (Milkdown has no native source map)
   */
  currentLine = $state(1);

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
