import type { DiffSubmode, Mode } from "$lib/types";

export type Theme = "auto" | "light" | "dark";
export type FontSize = "small" | "medium" | "large";
export type TabWidth = 2 | 4 | 8;
export type EditorTheme = "github" | "solarized" | "dracula";
/** highlight.js theme name (matches filenames under
 * `highlight.js/styles/*.css`). The registry of shipped themes lives in
 * `src/lib/views/highlight.svelte.ts` — keep this string typed loosely so
 * we can extend the registry without threading the type through here. */
export type CodeHighlightTheme = string;

export interface Settings {
  theme: Theme;
  editorFontSize: FontSize;
  defaultMode: Mode;
  /** When the open file changes externally and the buffer is clean, swap to
   * the disk content silently. With this off, every external change shows
   * the same banner that dirty changes get, so the user always confirms. */
  autoReload: boolean;
  /** Source view: wrap long lines at the editor edge (true) vs. let them
   * scroll horizontally (false). Live / Preview / WYSIWYG always wrap
   * because they're reading views. */
  softWrap: boolean;
  /** Source view: show the line-number gutter. */
  lineNumbers: boolean;
  /** Source view: how wide a tab character renders. */
  tabWidth: TabWidth;
  /** Diff view: how long to wait after a doc edit before recomputing the
   * diff. Lower = snappier, higher = less CPU on large files. */
  diffDebounceMs: number;
  /** Diff view: which sub-mode (Highlight / Full / Side-by-Side) to land
   * on when entering Diff mode. */
  diffDefaultSubmode: DiffSubmode;
  /** Editor syntax theme — swaps the --mddiff-syntax-* palette used by the
   * Source view's markdown highlighting. Doesn't touch editor background
   * or text color so it stays consistent with the app's light/dark mode. */
  editorTheme: EditorTheme;
  /** Show the outline sidebar (TOC) on the right. Toggled by ⌘⇧O. */
  outlineOpen: boolean;
  /** Browser/OS-native spellcheck on the editable surfaces (Source, Live
   * Preview, WYSIWYG). Toggling persists immediately but only takes effect
   * on view (re)mount — we don't reconfigure CM's contentAttributes
   * dynamically. */
  spellcheck: boolean;
  /** highlight.js theme applied to fenced code blocks when the app is in
   * light mode. Matches a filename under `highlight.js/styles/`. */
  codeHighlightThemeLight: CodeHighlightTheme;
  /** Same, but for dark mode. Swapped automatically based on the effective
   * light/dark state (auto follows OS). */
  codeHighlightThemeDark: CodeHighlightTheme;
  /** Names of highlight.js languages loaded in addition to the built-in
   * "common" set (~35 languages). Each name is loaded lazily via dynamic
   * import at startup, so shipping default = [] costs zero extra bundle. */
  codeHighlightExtras: string[];
}

const STORAGE_KEY = "mddiff-settings-v1";

const DEFAULTS: Settings = {
  theme: "auto",
  editorFontSize: "medium",
  defaultMode: "source",
  autoReload: true,
  softWrap: true,
  lineNumbers: true,
  tabWidth: 4,
  diffDebounceMs: 250,
  diffDefaultSubmode: "sidebyside",
  editorTheme: "github",
  outlineOpen: false,
  spellcheck: false,
  codeHighlightThemeLight: "github",
  codeHighlightThemeDark: "github-dark",
  codeHighlightExtras: [],
};

function load(): Settings {
  if (typeof localStorage === "undefined") return { ...DEFAULTS };
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (!raw) return { ...DEFAULTS };
    const parsed = JSON.parse(raw) as Partial<Settings>;
    return { ...DEFAULTS, ...parsed };
  } catch {
    return { ...DEFAULTS };
  }
}

class SettingsStore {
  theme = $state<Theme>(DEFAULTS.theme);
  editorFontSize = $state<FontSize>(DEFAULTS.editorFontSize);
  defaultMode = $state<Mode>(DEFAULTS.defaultMode);
  autoReload = $state<boolean>(DEFAULTS.autoReload);
  softWrap = $state<boolean>(DEFAULTS.softWrap);
  lineNumbers = $state<boolean>(DEFAULTS.lineNumbers);
  tabWidth = $state<TabWidth>(DEFAULTS.tabWidth);
  diffDebounceMs = $state<number>(DEFAULTS.diffDebounceMs);
  diffDefaultSubmode = $state<DiffSubmode>(DEFAULTS.diffDefaultSubmode);
  editorTheme = $state<EditorTheme>(DEFAULTS.editorTheme);
  outlineOpen = $state<boolean>(DEFAULTS.outlineOpen);
  spellcheck = $state<boolean>(DEFAULTS.spellcheck);
  codeHighlightThemeLight = $state<CodeHighlightTheme>(
    DEFAULTS.codeHighlightThemeLight,
  );
  codeHighlightThemeDark = $state<CodeHighlightTheme>(
    DEFAULTS.codeHighlightThemeDark,
  );
  codeHighlightExtras = $state<string[]>([...DEFAULTS.codeHighlightExtras]);

  /** Hydrate from localStorage. Call once at app mount on the client. */
  hydrate() {
    const s = load();
    this.theme = s.theme;
    this.editorFontSize = s.editorFontSize;
    this.defaultMode = s.defaultMode;
    this.autoReload = s.autoReload;
    this.softWrap = s.softWrap;
    this.lineNumbers = s.lineNumbers;
    this.tabWidth = s.tabWidth;
    this.diffDebounceMs = s.diffDebounceMs;
    this.diffDefaultSubmode = s.diffDefaultSubmode;
    this.editorTheme = s.editorTheme;
    this.outlineOpen = s.outlineOpen;
    this.spellcheck = s.spellcheck;
    this.codeHighlightThemeLight = s.codeHighlightThemeLight;
    this.codeHighlightThemeDark = s.codeHighlightThemeDark;
    this.codeHighlightExtras = [...s.codeHighlightExtras];
  }

  persist() {
    if (typeof localStorage === "undefined") return;
    const snapshot: Settings = {
      theme: this.theme,
      editorFontSize: this.editorFontSize,
      defaultMode: this.defaultMode,
      autoReload: this.autoReload,
      softWrap: this.softWrap,
      lineNumbers: this.lineNumbers,
      tabWidth: this.tabWidth,
      diffDebounceMs: this.diffDebounceMs,
      diffDefaultSubmode: this.diffDefaultSubmode,
      editorTheme: this.editorTheme,
      outlineOpen: this.outlineOpen,
      spellcheck: this.spellcheck,
      codeHighlightThemeLight: this.codeHighlightThemeLight,
      codeHighlightThemeDark: this.codeHighlightThemeDark,
      codeHighlightExtras: [...this.codeHighlightExtras],
    };
    localStorage.setItem(STORAGE_KEY, JSON.stringify(snapshot));
  }

  reset() {
    this.theme = DEFAULTS.theme;
    this.editorFontSize = DEFAULTS.editorFontSize;
    this.defaultMode = DEFAULTS.defaultMode;
    this.autoReload = DEFAULTS.autoReload;
    this.softWrap = DEFAULTS.softWrap;
    this.lineNumbers = DEFAULTS.lineNumbers;
    this.tabWidth = DEFAULTS.tabWidth;
    this.diffDebounceMs = DEFAULTS.diffDebounceMs;
    this.diffDefaultSubmode = DEFAULTS.diffDefaultSubmode;
    this.editorTheme = DEFAULTS.editorTheme;
    this.outlineOpen = DEFAULTS.outlineOpen;
    this.spellcheck = DEFAULTS.spellcheck;
    this.codeHighlightThemeLight = DEFAULTS.codeHighlightThemeLight;
    this.codeHighlightThemeDark = DEFAULTS.codeHighlightThemeDark;
    this.codeHighlightExtras = [...DEFAULTS.codeHighlightExtras];
    this.persist();
  }
}

export const settings = new SettingsStore();

export const FONT_SIZE_PX: Record<FontSize, number> = {
  small: 12,
  medium: 14,
  large: 17,
};
