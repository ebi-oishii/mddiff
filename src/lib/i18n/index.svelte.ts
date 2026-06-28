import { en, type Dictionary } from "./en";
import { ja } from "./ja";

const DICTS = { en, ja } as const;

/** Display locale. `auto` resolves at hydrate() time from navigator.language. */
export type Locale = "auto" | "en" | "ja";
/** Concrete (non-auto) locale that t() can actually look up. */
export type ResolvedLocale = Exclude<Locale, "auto">;

const STORAGE_KEY = "mddiff-locale-v1";

function detectFromNavigator(): ResolvedLocale {
  if (typeof navigator === "undefined") return "en";
  const lang = navigator.language?.toLowerCase() ?? "";
  return lang.startsWith("ja") ? "ja" : "en";
}

/**
 * Walk a dotted key path into the dictionary, returning the leaf string. Falls
 * back to the key itself when the path doesn't resolve so missing keys are
 * obvious in the UI (you see `foo.bar`) without throwing.
 *
 * Optional `params` substitutes `{name}` placeholders.
 */
function lookup(
  dict: Dictionary,
  key: string,
  params?: Record<string, string | number>,
): string {
  const parts = key.split(".");
  // Walk through nested keys. Use `unknown` and type-narrow each step rather
  // than `any` so we still get errors for truly wrong shapes.
  let cur: unknown = dict;
  for (const p of parts) {
    if (typeof cur !== "object" || cur === null) return key;
    cur = (cur as Record<string, unknown>)[p];
  }
  if (typeof cur !== "string") return key;
  if (!params) return cur;
  return cur.replace(/\{(\w+)\}/g, (_, k: string) =>
    k in params ? String(params[k]) : `{${k}}`,
  );
}

class I18nStore {
  /** User-chosen preference. May be `auto`. */
  preference = $state<Locale>("auto");
  /** Resolved concrete locale. Always `en` or `ja`. */
  resolved = $state<ResolvedLocale>("en");

  /** Call once at app mount. Reads localStorage + falls back to navigator. */
  hydrate() {
    if (typeof localStorage !== "undefined") {
      const saved = localStorage.getItem(STORAGE_KEY);
      if (saved === "en" || saved === "ja" || saved === "auto") {
        this.preference = saved;
      }
    }
    this.resolved =
      this.preference === "auto" ? detectFromNavigator() : this.preference;
  }

  set(locale: Locale) {
    this.preference = locale;
    this.resolved = locale === "auto" ? detectFromNavigator() : locale;
    if (typeof localStorage !== "undefined") {
      localStorage.setItem(STORAGE_KEY, locale);
    }
  }

  /**
   * Translate a key (e.g. `"menu.open"`) using the currently resolved locale.
   * Reads `this.resolved` ($state) so callers used inside Svelte templates
   * re-evaluate when the locale changes.
   */
  t(key: string, params?: Record<string, string | number>): string {
    return lookup(DICTS[this.resolved], key, params);
  }
}

export const i18n = new I18nStore();
