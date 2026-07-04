/**
 * Syntax-highlighting integration for Preview, Live Preview and future
 * consumers. Wraps highlight.js in a way that:
 *
 *   1. Always ships the *common* language set (~35 languages) as a static
 *      import — it's the mainline cost we're OK paying (~50KB gzip).
 *   2. Exposes extras (~150 additional languages) as lazy, code-split chunks
 *      via `import.meta.glob` so they only enter the bundle if the user
 *      opts them in from Settings.
 *   3. Ships a curated set of light/dark syntax themes as inline CSS
 *      strings via Vite's `?inline` import — swapping a theme is a
 *      textContent update on a single <style> tag, no network roundtrip.
 *
 * Extending later:
 *   - New extra language → drop the name into `EXTRA_LANGUAGES`; Vite
 *     already globs the whole `lib/languages/*.js` directory, so no
 *     wiring changes are needed.
 *   - New theme → add a `?inline` import + entry in `THEMES`. The
 *     Settings dropdown picks up names from `THEME_LIGHT` / `THEME_DARK`
 *     via their `themes.ts` re-exports.
 */
import hljs from "highlight.js/lib/common";

// -----------------------------------------------------------------------
// Language loading
// -----------------------------------------------------------------------

// Names of every language shipped with highlight.js *outside* the common
// set. Generated at branch time from `node_modules/highlight.js/lib/languages/`
// minus `lib/common.js`'s import list. Kept sorted so it diffs cleanly
// when new upstream languages appear.
export const EXTRA_LANGUAGES: readonly string[] = [
  "1c", "abnf", "accesslog", "actionscript", "ada", "angelscript",
  "apache", "applescript", "arcade", "arduino", "armasm", "asciidoc",
  "aspectj", "autohotkey", "autoit", "avrasm", "awk", "axapta", "basic",
  "bnf", "brainfuck", "cal", "capnproto", "ceylon", "clean", "clojure",
  "clojure-repl", "cmake", "coffeescript", "coq", "cos", "crmsh",
  "crystal", "csp", "d", "dart", "delphi", "diff", "django", "dns",
  "dockerfile", "dos", "dsconfig", "dts", "dust", "ebnf", "elixir",
  "elm", "erb", "erlang", "erlang-repl", "excel", "fix", "flix",
  "fortran", "fsharp", "gams", "gauss", "gcode", "gherkin", "glsl",
  "gml", "golo", "gradle", "groovy", "haml", "handlebars", "haskell",
  "haxe", "hsp", "http", "hy", "inform7", "irpf90", "isbl", "jboss-cli",
  "julia", "julia-repl", "lasso", "latex", "ldif", "leaf", "lisp",
  "livecodeserver", "livescript", "llvm", "lsl", "mathematica", "matlab",
  "maxima", "mel", "mercury", "mipsasm", "mizar", "mojolicious",
  "monkey", "moonscript", "n1ql", "nestedtext", "nginx", "nim", "nix",
  "node-repl", "nsis", "ocaml", "openscad", "oxygene", "parser3",
  "pf", "pgsql", "pony", "powershell", "processing", "profile",
  "prolog", "properties", "protobuf", "puppet", "purebasic", "q",
  "qml", "reasonml", "rib", "roboconf", "routeros", "rsl", "ruleslanguage",
  "sas", "scala", "scheme", "scilab", "smali", "smalltalk", "sml",
  "sqf", "stan", "stata", "step21", "stylus", "subunit", "taggerscript",
  "tap", "tcl", "thrift", "tp", "twig", "vala", "vbscript",
  "vbscript-html", "verilog", "vhdl", "vim", "wren", "x86asm", "xl",
  "xquery", "zephir",
];

// Vite bundles each of these as its own async chunk (see rollup output).
// The glob path targets node_modules; Vite's default settings resolve
// package files inside node_modules just fine as glob targets.
const langModules = import.meta.glob<{ default: unknown }>(
  "/node_modules/highlight.js/lib/languages/*.js",
);

// Cache of already-registered extras so a settings-change re-run doesn't
// pay the import cost twice.
const registered = new Set<string>();

/**
 * Load and register a single extra language. Safe to call for the same
 * name multiple times — becomes a no-op after the first successful
 * registration. Silently ignores unknown names (invalid setting won't
 * blow up the app).
 */
export async function loadExtraLanguage(name: string): Promise<void> {
  if (registered.has(name)) return;
  const key = `/node_modules/highlight.js/lib/languages/${name}.js`;
  const loader = langModules[key];
  if (!loader) return;
  try {
    const mod = await loader();
    // hljs.registerLanguage accepts a LanguageFn synthesizer.
    hljs.registerLanguage(name, mod.default as never);
    registered.add(name);
  } catch (e) {
    console.warn(`[mddiff] failed to load extra language "${name}":`, e);
  }
}

/**
 * Load a batch of extras in parallel. Used at app startup once
 * `settings.codeHighlightExtras` has been hydrated.
 */
export async function loadExtraLanguages(names: readonly string[]): Promise<void> {
  await Promise.all(names.map((n) => loadExtraLanguage(n)));
}

// Re-export hljs so consumers don't reach into common paths directly.
// Callers use this to feed markdown-it's `highlight` option and the
// CodeMirror plugin for Live Preview.
export { hljs };

// -----------------------------------------------------------------------
// Theme loading
// -----------------------------------------------------------------------

// Curated set of light/dark themes. Additions here should also update
// THEMES_LIGHT / THEMES_DARK below so the Settings dropdown surfaces them.
//
// `?inline` reads the CSS as a string at build time, avoiding a network
// roundtrip to a stylesheet file. Total size for the set below is
// ~20KB, negligible next to the JS bundle.
import themeGithub from "highlight.js/styles/github.css?inline";
import themeGithubDark from "highlight.js/styles/github-dark.css?inline";
import themeGithubDarkDimmed from "highlight.js/styles/github-dark-dimmed.css?inline";
import themeAtomOneLight from "highlight.js/styles/atom-one-light.css?inline";
import themeAtomOneDark from "highlight.js/styles/atom-one-dark.css?inline";
import themeVs from "highlight.js/styles/vs.css?inline";
import themeVs2015 from "highlight.js/styles/vs2015.css?inline";
import themeXcode from "highlight.js/styles/xcode.css?inline";
import themeMonokai from "highlight.js/styles/monokai.css?inline";
import themeNord from "highlight.js/styles/nord.css?inline";
import themeTokyoNightLight from "highlight.js/styles/tokyo-night-light.css?inline";
import themeTokyoNightDark from "highlight.js/styles/tokyo-night-dark.css?inline";
import themeKimbieLight from "highlight.js/styles/kimbie-light.css?inline";
import themeKimbieDark from "highlight.js/styles/kimbie-dark.css?inline";
import themeObsidian from "highlight.js/styles/obsidian.css?inline";

/** Registry of every shipped theme, keyed by highlight.js filename. */
export const THEMES: Record<string, string> = {
  github: themeGithub,
  "github-dark": themeGithubDark,
  "github-dark-dimmed": themeGithubDarkDimmed,
  "atom-one-light": themeAtomOneLight,
  "atom-one-dark": themeAtomOneDark,
  vs: themeVs,
  vs2015: themeVs2015,
  xcode: themeXcode,
  monokai: themeMonokai,
  nord: themeNord,
  "tokyo-night-light": themeTokyoNightLight,
  "tokyo-night-dark": themeTokyoNightDark,
  "kimbie-light": themeKimbieLight,
  "kimbie-dark": themeKimbieDark,
  obsidian: themeObsidian,
};

/** Themes that read cleanly on a light background. Drives the Settings
 * dropdown's `codeHighlightThemeLight` options. */
export const THEMES_LIGHT: readonly string[] = [
  "github",
  "atom-one-light",
  "vs",
  "xcode",
  "tokyo-night-light",
  "kimbie-light",
];

/** Themes that read cleanly on a dark background. */
export const THEMES_DARK: readonly string[] = [
  "github-dark",
  "github-dark-dimmed",
  "atom-one-dark",
  "vs2015",
  "monokai",
  "nord",
  "tokyo-night-dark",
  "kimbie-dark",
  "obsidian",
];

const STYLE_ID = "mddiff-hljs-theme";

/**
 * Apply a theme by writing its CSS into a well-known <style> tag in
 * document head. Idempotent — replaces content in place rather than
 * churning DOM nodes.
 */
export function applyHighlightTheme(name: string): void {
  if (typeof document === "undefined") return;
  const css = THEMES[name] ?? THEMES.github;
  let style = document.getElementById(STYLE_ID) as HTMLStyleElement | null;
  if (!style) {
    style = document.createElement("style");
    style.id = STYLE_ID;
    document.head.appendChild(style);
  }
  if (style.textContent !== css) {
    style.textContent = css;
  }
}
