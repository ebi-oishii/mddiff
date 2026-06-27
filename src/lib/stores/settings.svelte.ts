import type { Mode } from "$lib/types";

export type Theme = "auto" | "light" | "dark";
export type FontSize = "small" | "medium" | "large";

export interface Settings {
  theme: Theme;
  editorFontSize: FontSize;
  defaultMode: Mode;
}

const STORAGE_KEY = "mdv-settings-v1";

const DEFAULTS: Settings = {
  theme: "auto",
  editorFontSize: "medium",
  defaultMode: "source",
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

  /** Hydrate from localStorage. Call once at app mount on the client. */
  hydrate() {
    const s = load();
    this.theme = s.theme;
    this.editorFontSize = s.editorFontSize;
    this.defaultMode = s.defaultMode;
  }

  persist() {
    if (typeof localStorage === "undefined") return;
    const snapshot: Settings = {
      theme: this.theme,
      editorFontSize: this.editorFontSize,
      defaultMode: this.defaultMode,
    };
    localStorage.setItem(STORAGE_KEY, JSON.stringify(snapshot));
  }

  reset() {
    this.theme = DEFAULTS.theme;
    this.editorFontSize = DEFAULTS.editorFontSize;
    this.defaultMode = DEFAULTS.defaultMode;
    this.persist();
  }
}

export const settings = new SettingsStore();

export const FONT_SIZE_PX: Record<FontSize, number> = {
  small: 12,
  medium: 14,
  large: 17,
};
