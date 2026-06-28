/**
 * DOM-based find for read-only views (Preview, Diff sub-views) that don't
 * use CodeMirror. Walks text nodes inside a given scope, wraps every
 * case-insensitive occurrence of the query in a `<mark class="mdv-find-hit">`,
 * and exposes navigation between matches.
 *
 * Marker styling lives in FindBar.svelte (`:global`) so views don't need to
 * repeat it.
 */
export class DomFinder {
  private marks: HTMLElement[] = [];
  private currentIdx = -1;

  constructor(private scope: HTMLElement) {}

  get matchCount(): number {
    return this.marks.length;
  }

  /** 1-based, 0 when no current match. */
  get currentIndex(): number {
    return this.currentIdx + 1;
  }

  apply(query: string): void {
    this.clear();
    if (!query) return;
    if (!this.scope || !this.scope.isConnected) return;

    const lowerQ = query.toLowerCase();
    const qLen = query.length;

    // Collect text nodes up front; mutating during traversal breaks the
    // TreeWalker.
    const textNodes: Text[] = [];
    const walker = document.createTreeWalker(this.scope, NodeFilter.SHOW_TEXT, {
      acceptNode: (node) => {
        // Skip text inside the find UI itself (it lives outside this scope
        // anyway, but be defensive) and inside script/style elements.
        const parent = (node as Text).parentElement;
        if (!parent) return NodeFilter.FILTER_REJECT;
        if (parent.closest(".find-bar")) return NodeFilter.FILTER_REJECT;
        const tag = parent.tagName;
        if (tag === "SCRIPT" || tag === "STYLE") return NodeFilter.FILTER_REJECT;
        // Skip diff gutter line numbers and +/- signs — matching against
        // `1` shouldn't paint every numbered gutter cell. These classes
        // are used by FullDiffView / HighlightView.
        if (parent.closest(".ln, .sign")) return NodeFilter.FILTER_REJECT;
        return NodeFilter.FILTER_ACCEPT;
      },
    });
    let n: Node | null;
    while ((n = walker.nextNode())) textNodes.push(n as Text);

    for (const node of textNodes) {
      const text = node.nodeValue ?? "";
      const lower = text.toLowerCase();
      let from = 0;
      let lastEnd = 0;
      const fragments: Node[] = [];
      while (true) {
        const found = lower.indexOf(lowerQ, from);
        if (found < 0) break;
        if (found > lastEnd) {
          fragments.push(document.createTextNode(text.slice(lastEnd, found)));
        }
        const mark = document.createElement("mark");
        mark.className = "mdv-find-hit";
        mark.textContent = text.slice(found, found + qLen);
        fragments.push(mark);
        this.marks.push(mark);
        lastEnd = found + qLen;
        from = lastEnd;
      }
      if (fragments.length === 0) continue;
      if (lastEnd < text.length) {
        fragments.push(document.createTextNode(text.slice(lastEnd)));
      }
      const parent = node.parentNode;
      if (!parent) continue;
      for (const f of fragments) parent.insertBefore(f, node);
      parent.removeChild(node);
    }

    if (this.marks.length > 0) {
      this.currentIdx = 0;
      this.markCurrent();
      this.scrollCurrentIntoView();
    }
  }

  next(): void {
    if (this.marks.length === 0) return;
    this.currentIdx = (this.currentIdx + 1) % this.marks.length;
    this.markCurrent();
    this.scrollCurrentIntoView();
  }

  prev(): void {
    if (this.marks.length === 0) return;
    this.currentIdx =
      (this.currentIdx - 1 + this.marks.length) % this.marks.length;
    this.markCurrent();
    this.scrollCurrentIntoView();
  }

  clear(): void {
    for (const m of this.marks) {
      const parent = m.parentNode;
      if (!parent) continue;
      parent.replaceChild(document.createTextNode(m.textContent ?? ""), m);
      // Collapse adjacent text nodes that we split apart on apply().
      parent.normalize();
    }
    this.marks = [];
    this.currentIdx = -1;
  }

  private markCurrent(): void {
    this.marks.forEach((m, i) => {
      m.classList.toggle("mdv-find-current", i === this.currentIdx);
    });
  }

  private scrollCurrentIntoView(): void {
    if (this.currentIdx < 0) return;
    this.marks[this.currentIdx]?.scrollIntoView({
      block: "center",
      behavior: "smooth",
    });
  }
}
