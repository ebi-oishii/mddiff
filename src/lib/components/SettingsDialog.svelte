<script lang="ts">
  import { settings } from "$lib/stores/settings.svelte";
  import type {
    EditorTheme,
    FontSize,
    TabWidth,
    Theme,
  } from "$lib/stores/settings.svelte";
  import type { DiffSubmode, Mode } from "$lib/types";
  import { i18n, type Locale } from "$lib/i18n/index.svelte";
  import {
    EXTRA_LANGUAGES,
    THEMES_LIGHT,
    THEMES_DARK,
  } from "$lib/views/highlight.svelte";

  let { onClose }: { onClose: () => void } = $props();

  function persistChange<K extends keyof typeof settings>(key: K, value: unknown) {
    // @ts-expect-error narrow on key — store fields are heterogeneous
    settings[key] = value;
    settings.persist();
  }
</script>

<div class="overlay" role="presentation" onclick={onClose}>
  <div
    class="modal"
    role="dialog"
    tabindex="-1"
    aria-modal="true"
    aria-labelledby="settings-title"
    onclick={(e) => e.stopPropagation()}
    onkeydown={(e) => e.key === "Escape" && onClose()}
  >
    <h2 id="settings-title">{i18n.t("settings.title")}</h2>

    <div class="section-title">{i18n.t("settings.sectionAppearance")}</div>

    <div class="row">
      <label for="theme">{i18n.t("settings.theme")}</label>
      <select
        id="theme"
        value={settings.theme}
        onchange={(e) =>
          persistChange("theme", (e.currentTarget as HTMLSelectElement).value as Theme)}
      >
        <option value="auto">{i18n.t("settings.themeAuto")}</option>
        <option value="light">{i18n.t("settings.themeLight")}</option>
        <option value="dark">{i18n.t("settings.themeDark")}</option>
      </select>
    </div>

    <div class="row">
      <label for="fontsize">{i18n.t("settings.fontSize")}</label>
      <select
        id="fontsize"
        value={settings.editorFontSize}
        onchange={(e) =>
          persistChange(
            "editorFontSize",
            (e.currentTarget as HTMLSelectElement).value as FontSize,
          )}
      >
        <option value="small">{i18n.t("settings.fontSizeSmall")}</option>
        <option value="medium">{i18n.t("settings.fontSizeMedium")}</option>
        <option value="large">{i18n.t("settings.fontSizeLarge")}</option>
      </select>
    </div>

    <div class="row">
      <label for="defmode">{i18n.t("settings.defaultMode")}</label>
      <select
        id="defmode"
        value={settings.defaultMode}
        onchange={(e) =>
          persistChange("defaultMode", (e.currentTarget as HTMLSelectElement).value as Mode)}
      >
        <option value="source">{i18n.t("mode.source")}</option>
        <option value="live">{i18n.t("mode.live")}</option>
        <option value="wysiwyg">{i18n.t("mode.wysiwyg")}</option>
        <option value="preview">{i18n.t("mode.preview")}</option>
        <option value="diff">{i18n.t("mode.diff")}</option>
      </select>
    </div>

    <div class="row">
      <label for="editortheme">{i18n.t("settings.editorTheme")}</label>
      <select
        id="editortheme"
        value={settings.editorTheme}
        onchange={(e) =>
          persistChange(
            "editorTheme",
            (e.currentTarget as HTMLSelectElement).value as EditorTheme,
          )}
      >
        <option value="github">{i18n.t("settings.editorThemeGithub")}</option>
        <option value="solarized">{i18n.t("settings.editorThemeSolarized")}</option>
        <option value="dracula">{i18n.t("settings.editorThemeDracula")}</option>
      </select>
    </div>

    <div class="section-title">{i18n.t("settings.sectionLanguage")}</div>

    <div class="row">
      <label for="language">{i18n.t("settings.language")}</label>
      <select
        id="language"
        value={i18n.preference}
        onchange={(e) =>
          i18n.set((e.currentTarget as HTMLSelectElement).value as Locale)}
      >
        <option value="auto">{i18n.t("settings.languageAuto")}</option>
        <option value="en">{i18n.t("settings.languageEn")}</option>
        <option value="ja">{i18n.t("settings.languageJa")}</option>
      </select>
    </div>

    <div class="section-title">{i18n.t("settings.sectionFile")}</div>

    <div class="row">
      <label for="autoreload">{i18n.t("settings.autoReload")}</label>
      <input
        id="autoreload"
        type="checkbox"
        checked={settings.autoReload}
        onchange={(e) =>
          persistChange("autoReload", (e.currentTarget as HTMLInputElement).checked)}
      />
    </div>
    <p class="row-hint">{i18n.t("settings.autoReloadHint")}</p>

    <div class="section-title">{i18n.t("settings.sectionSource")}</div>

    <div class="row">
      <label for="softwrap">{i18n.t("settings.softWrap")}</label>
      <input
        id="softwrap"
        type="checkbox"
        checked={settings.softWrap}
        onchange={(e) =>
          persistChange("softWrap", (e.currentTarget as HTMLInputElement).checked)}
      />
    </div>

    <div class="row">
      <label for="linenumbers">{i18n.t("settings.lineNumbers")}</label>
      <input
        id="linenumbers"
        type="checkbox"
        checked={settings.lineNumbers}
        onchange={(e) =>
          persistChange("lineNumbers", (e.currentTarget as HTMLInputElement).checked)}
      />
    </div>

    <div class="row">
      <label for="spellcheck">{i18n.t("settings.spellcheck")}</label>
      <input
        id="spellcheck"
        type="checkbox"
        checked={settings.spellcheck}
        onchange={(e) =>
          persistChange("spellcheck", (e.currentTarget as HTMLInputElement).checked)}
      />
    </div>
    <p class="row-hint">{i18n.t("settings.spellcheckHint")}</p>

    <div class="row">
      <label for="tabwidth">{i18n.t("settings.tabWidth")}</label>
      <select
        id="tabwidth"
        value={String(settings.tabWidth)}
        onchange={(e) =>
          persistChange(
            "tabWidth",
            Number((e.currentTarget as HTMLSelectElement).value) as TabWidth,
          )}
      >
        <option value="2">{i18n.t("settings.tab2")}</option>
        <option value="4">{i18n.t("settings.tab4")}</option>
        <option value="8">{i18n.t("settings.tab8")}</option>
      </select>
    </div>

    <div class="section-title">{i18n.t("settings.sectionCodeHighlight")}</div>

    <div class="row">
      <label for="cheme-light">{i18n.t("settings.codeHighlightThemeLight")}</label>
      <select
        id="cheme-light"
        value={settings.codeHighlightThemeLight}
        onchange={(e) =>
          persistChange(
            "codeHighlightThemeLight",
            (e.currentTarget as HTMLSelectElement).value,
          )}
      >
        {#each THEMES_LIGHT as t}
          <option value={t}>{t}</option>
        {/each}
      </select>
    </div>

    <div class="row">
      <label for="cheme-dark">{i18n.t("settings.codeHighlightThemeDark")}</label>
      <select
        id="cheme-dark"
        value={settings.codeHighlightThemeDark}
        onchange={(e) =>
          persistChange(
            "codeHighlightThemeDark",
            (e.currentTarget as HTMLSelectElement).value,
          )}
      >
        {#each THEMES_DARK as t}
          <option value={t}>{t}</option>
        {/each}
      </select>
    </div>
    <p class="row-hint">{i18n.t("settings.codeHighlightThemeHint")}</p>

    <div class="row extras-row">
      <label for="cheme-extras">{i18n.t("settings.codeHighlightExtras")}</label>
      <div class="extras">
        <details>
          <summary>
            {settings.codeHighlightExtras.length === 0
              ? i18n.t("settings.codeHighlightExtrasNone")
              : i18n.t("settings.codeHighlightExtrasCount", {
                  n: String(settings.codeHighlightExtras.length),
                })}
          </summary>
          <div class="extras-grid">
            {#each EXTRA_LANGUAGES as name}
              {@const checked = settings.codeHighlightExtras.includes(name)}
              <label class="extras-item">
                <input
                  type="checkbox"
                  {checked}
                  onchange={(e) => {
                    const on = (e.currentTarget as HTMLInputElement).checked;
                    const next = on
                      ? [...settings.codeHighlightExtras, name]
                      : settings.codeHighlightExtras.filter((x) => x !== name);
                    persistChange("codeHighlightExtras", next);
                  }}
                />
                <span>{name}</span>
              </label>
            {/each}
          </div>
        </details>
      </div>
    </div>
    <p class="row-hint">{i18n.t("settings.codeHighlightExtrasHint")}</p>

    <div class="section-title">{i18n.t("settings.sectionDiff")}</div>

    <div class="row">
      <label for="diffsubmode">{i18n.t("settings.diffSubmode")}</label>
      <select
        id="diffsubmode"
        value={settings.diffDefaultSubmode}
        onchange={(e) =>
          persistChange(
            "diffDefaultSubmode",
            (e.currentTarget as HTMLSelectElement).value as DiffSubmode,
          )}
      >
        <option value="highlight">{i18n.t("settings.diffSubmodeHighlight")}</option>
        <option value="full">{i18n.t("settings.diffSubmodeFull")}</option>
        <option value="sidebyside">{i18n.t("settings.diffSubmodeSbs")}</option>
        <option value="blame">{i18n.t("settings.diffSubmodeBlame")}</option>
      </select>
    </div>

    <div class="row">
      <label for="diffdebounce">{i18n.t("settings.diffDebounce")}</label>
      <select
        id="diffdebounce"
        value={String(settings.diffDebounceMs)}
        onchange={(e) =>
          persistChange(
            "diffDebounceMs",
            Number((e.currentTarget as HTMLSelectElement).value),
          )}
      >
        <option value="100">{i18n.t("settings.diffDebounce100")}</option>
        <option value="250">{i18n.t("settings.diffDebounce250")}</option>
        <option value="500">{i18n.t("settings.diffDebounce500")}</option>
        <option value="1000">{i18n.t("settings.diffDebounce1000")}</option>
      </select>
    </div>

    <p class="hint">{i18n.t("settings.hint")}</p>

    <div class="actions">
      <button type="button" class="link" onclick={() => settings.reset()}
        >{i18n.t("settings.restore")}</button
      >
      <button type="button" class="primary" onclick={onClose}>{i18n.t("settings.done")}</button>
    </div>
  </div>
</div>

<style>
  .overlay {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.45);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 100;
    padding: 1rem;
  }
  .modal {
    background: var(--mddiff-surface-pop);
    color: var(--mddiff-text);
    border: 1px solid var(--mddiff-border);
    border-radius: 8px;
    padding: 1.25rem 1.5rem;
    max-width: 28em;
    width: 100%;
    box-shadow: 0 12px 40px var(--mddiff-shadow);
    max-height: calc(100vh - 2rem);
    overflow-y: auto;
  }
  h2 {
    margin: 0 0 1rem;
    font-size: 1.1rem;
  }
  .section-title {
    margin: 1rem 0 0.4rem;
    font-size: 0.72rem;
    text-transform: uppercase;
    letter-spacing: 0.06em;
    color: var(--mddiff-text-mute);
    border-top: 1px solid var(--mddiff-border-mute);
    padding-top: 0.7rem;
  }
  .section-title:first-of-type {
    border-top: 0;
    padding-top: 0;
    margin-top: 0;
  }
  .row {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    margin: 0.5rem 0;
    font-size: 0.9rem;
  }
  .row label {
    flex: 1;
  }
  .row select {
    font: inherit;
    padding: 0.3rem 0.5rem;
    background: var(--mddiff-bg);
    color: var(--mddiff-text);
    border: 1px solid var(--mddiff-border);
    border-radius: 4px;
    min-width: 12em;
  }
  .row input[type="checkbox"] {
    width: 1rem;
    height: 1rem;
    margin: 0;
  }
  .hint {
    margin: 1rem 0 0;
    font-size: 0.8rem;
    color: var(--mddiff-text-mute);
  }
  .row-hint {
    margin: -0.3rem 0 0.6rem;
    font-size: 0.78rem;
    color: var(--mddiff-text-mute);
    line-height: 1.4;
  }
  /* Extras: 156 language checkboxes are too many to show flat; wrap in a
     collapsed <details> summary + a 3-column grid so the dialog stays
     compact until the user opens it. */
  .extras-row {
    align-items: flex-start;
  }
  .extras {
    flex: 1;
    min-width: 0;
  }
  .extras summary {
    cursor: pointer;
    font-size: 0.85rem;
    color: var(--mddiff-text);
    padding: 0.35rem 0.5rem;
    border: 1px solid var(--mddiff-border);
    border-radius: 4px;
    background: var(--mddiff-surface);
  }
  .extras summary:hover {
    background: var(--mddiff-surface-hi);
  }
  .extras-grid {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: 0.15rem 0.75rem;
    margin-top: 0.5rem;
    max-height: 18rem;
    overflow-y: auto;
    padding: 0.4rem 0.5rem;
    border: 1px solid var(--mddiff-border);
    border-radius: 4px;
    font-size: 0.8rem;
  }
  .extras-item {
    display: flex;
    align-items: center;
    gap: 0.3rem;
    cursor: pointer;
    color: var(--mddiff-text);
    padding: 0.1rem 0;
  }
  .extras-item input {
    margin: 0;
  }
  .actions {
    display: flex;
    justify-content: space-between;
    gap: 0.5rem;
    margin-top: 1rem;
  }
  .actions button {
    background: transparent;
    border: 1px solid var(--mddiff-border);
    border-radius: 5px;
    padding: 0.4rem 1rem;
    font: inherit;
    color: var(--mddiff-text);
    cursor: pointer;
  }
  .actions button:hover {
    background: var(--mddiff-surface-hi);
  }
  .actions button.primary {
    background: var(--mddiff-accent);
    color: #fff;
    border-color: transparent;
  }
  .actions button.primary:hover {
    filter: brightness(0.92);
  }
  .actions button.link {
    border: 0;
    color: var(--mddiff-accent);
    padding: 0.4rem 0;
  }
  .actions button.link:hover {
    background: transparent;
    text-decoration: underline;
  }
</style>
