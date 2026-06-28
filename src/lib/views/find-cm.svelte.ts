import { SearchCursor } from "@codemirror/search";
import { EditorState, StateEffect, StateField } from "@codemirror/state";
import {
  Decoration,
  type DecorationSet,
  EditorView,
} from "@codemirror/view";

/**
 * CodeMirror-side find / replace state, paired with FindBar in the host
 * (SourceView / LivePreviewView). Differs from the DOM-based `FindState`
 * in that highlights are CM decorations (re-derived from the doc) instead
 * of `<mark>` wrappers in the DOM.
 *
 * Why not the built-in @codemirror/search panel: we want a UI consistent
 * with the DOM views (Preview, Diff, WYSIWYG), and the floating widget
 * gives more control over placement around the title overlay.
 */

type Match = { from: number; to: number };

interface FindData {
  query: string;
  matches: Match[];
  currentIdx: number; // 0-based, -1 for none
  decorations: DecorationSet;
}

const EMPTY: FindData = {
  query: "",
  matches: [],
  currentIdx: -1,
  decorations: Decoration.none,
};

const setFindQuery = StateEffect.define<string>();
const setCurrentIdx = StateEffect.define<number>();

function compute(state: EditorState, query: string, want: number): FindData {
  if (!query) return EMPTY;
  const matches: Match[] = [];
  // Case-insensitive to match find-dom.ts's behavior — same UI must mean
  // the same semantics across Source/Live (CM) and Preview/Diff (DOM).
  const cursor = new SearchCursor(
    state.doc,
    query,
    0,
    state.doc.length,
    (x) => x.toLowerCase(),
  );
  while (!cursor.next().done) {
    matches.push({ from: cursor.value.from, to: cursor.value.to });
  }
  if (matches.length === 0) {
    return { query, matches, currentIdx: -1, decorations: Decoration.none };
  }
  const idx = ((want % matches.length) + matches.length) % matches.length;
  return { query, matches, currentIdx: idx, decorations: buildDecos(matches, idx) };
}

function buildDecos(matches: Match[], current: number): DecorationSet {
  const decos = matches.map((m, i) =>
    Decoration.mark({
      class: i === current ? "mdv-find-hit mdv-find-current" : "mdv-find-hit",
    }).range(m.from, m.to),
  );
  return Decoration.set(decos, true);
}

const findField = StateField.define<FindData>({
  create: () => EMPTY,
  update(value, tr) {
    let next = value;
    for (const e of tr.effects) {
      if (e.is(setFindQuery)) {
        next = compute(tr.state, e.value, 0);
      } else if (e.is(setCurrentIdx)) {
        if (next.matches.length === 0) {
          next = { ...next, currentIdx: -1 };
        } else {
          const idx =
            ((e.value % next.matches.length) + next.matches.length) %
            next.matches.length;
          next = {
            ...next,
            currentIdx: idx,
            decorations: buildDecos(next.matches, idx),
          };
        }
      }
    }
    // Recompute matches when the document changes so highlights track edits.
    if (tr.docChanged && next.query) {
      next = compute(tr.state, next.query, next.currentIdx >= 0 ? next.currentIdx : 0);
    }
    return next;
  },
  provide: (f) => EditorView.decorations.from(f, (v) => v.decorations),
});

/** Build the CM extensions (StateField + listener). The listener calls
 * `onUpdate` whenever the find data changes so the host's reactive state
 * (matchCount / currentIndex) stays in sync. */
export function findExtension(onUpdate: (data: FindData) => void) {
  return [
    findField,
    EditorView.updateListener.of((update) => {
      const before = update.startState.field(findField, false);
      const after = update.state.field(findField, false);
      if (before !== after && after) onUpdate(after);
    }),
  ];
}

export class CmFindState {
  open = $state(false);
  query = $state("");
  replaceQuery = $state("");
  replaceVisible = $state(false);
  matchCount = $state(0);
  currentIndex = $state(0); // 1-based, 0 for no current
  focusVersion = $state(0);

  private view: EditorView | null = null;

  /** Receives the find data from the EditorView's updateListener. */
  syncFromData = (data: FindData) => {
    this.matchCount = data.matches.length;
    this.currentIndex = data.currentIdx + 1;
  };

  bind(view: EditorView) {
    this.view = view;
  }

  openFind = () => {
    this.open = true;
    this.replaceVisible = false;
    this.focusVersion++;
  };

  openReplace = () => {
    this.open = true;
    this.replaceVisible = true;
    this.focusVersion++;
  };

  close = () => {
    this.open = false;
    this.query = "";
    this.replaceQuery = "";
    this.replaceVisible = false;
    this.matchCount = 0;
    this.currentIndex = 0;
    this.view?.dispatch({ effects: setFindQuery.of("") });
  };

  next = () => {
    const data = this.view?.state.field(findField, false);
    if (!data || data.matches.length === 0) return;
    this.view?.dispatch({ effects: setCurrentIdx.of(data.currentIdx + 1) });
    this.scrollCurrentIntoView();
  };

  prev = () => {
    const data = this.view?.state.field(findField, false);
    if (!data || data.matches.length === 0) return;
    this.view?.dispatch({ effects: setCurrentIdx.of(data.currentIdx - 1) });
    this.scrollCurrentIntoView();
  };

  replace = () => {
    const view = this.view;
    if (!view) return;
    const data = view.state.field(findField, false);
    if (!data || data.currentIdx < 0) return;
    const match = data.matches[data.currentIdx];
    view.dispatch({
      changes: { from: match.from, to: match.to, insert: this.replaceQuery },
    });
    // After the doc change the field recomputed matches. The currentIdx is
    // kept; advance to the next so repeated Enter walks through.
    this.next();
  };

  replaceAll = () => {
    const view = this.view;
    if (!view) return;
    const data = view.state.field(findField, false);
    if (!data || data.matches.length === 0) return;
    const changes = data.matches.map((m) => ({
      from: m.from,
      to: m.to,
      insert: this.replaceQuery,
    }));
    view.dispatch({ changes });
  };

  /** Re-apply on query change. Host calls this from a $effect on
   * `this.query`. */
  refresh = () => {
    if (!this.view) return;
    if (!this.open) return;
    this.view.dispatch({ effects: setFindQuery.of(this.query) });
    this.scrollCurrentIntoView();
  };

  onKeydown = (e: KeyboardEvent) => {
    if (!(e.metaKey || e.ctrlKey)) return;
    const key = e.key.toLowerCase();
    if (key === "f") {
      e.preventDefault();
      this.openFind();
    } else if (key === "h") {
      e.preventDefault();
      this.openReplace();
    }
  };

  destroy() {
    this.view = null;
  }

  private scrollCurrentIntoView() {
    const view = this.view;
    if (!view) return;
    const data = view.state.field(findField, false);
    if (!data || data.currentIdx < 0) return;
    const match = data.matches[data.currentIdx];
    view.dispatch({
      effects: EditorView.scrollIntoView(match.from, { y: "center" }),
    });
  }
}
