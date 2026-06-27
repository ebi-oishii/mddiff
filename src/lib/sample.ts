/**
 * Built-in sample Markdown loaded by the "Sample" button. Exists primarily
 * so the app has something to look at when launched on a fresh simulator
 * (or any environment without convenient files to open). Covers the syntax
 * each mode handles, so it doubles as a smoke test.
 */
export const SAMPLE_MD = `# Welcome to mdv

A lightweight Markdown viewer and editor for Desktop, Mobile and TUI.

## Modes

Switch with the tabs above (⌘1〜⌘5 on Desktop):

- **Source** — raw Markdown editor (CodeMirror 6)
- **Live Preview** — Typora-style: the line you edit stays raw, others render inline
- **WYSIWYG** — rich editing via Milkdown
- **Preview** — read-only rendered view
- **Diff** — compare against a Git base (Desktop only)

## Task list

- [x] Try the **Source** mode
- [x] Switch to **Live Preview** — the syntax characters fade
- [ ] Switch to **WYSIWYG** and edit a heading
- [ ] Open the **Preview** to see the rendered result
- [ ] Save the file (\`⌘S\`) or close without saving

## Inline styles

You can use *italic*, **bold**, ~~strikethrough~~ and \`inline code\`.

Links work too: [Tauri](https://tauri.app), [Svelte](https://svelte.dev).

## Code block

\`\`\`rust
fn main() {
    let greeting = "こんにちは, mdv!";
    println!("{}", greeting);
}
\`\`\`

## Blockquote

> 1 ファイルを静かに読んで、少し直して、差分を見る。
> それが mdv の役目です。

## Table

| Mode         | Editable | Renders |
|--------------|----------|---------|
| Source       | ✓        | raw     |
| Live Preview | ✓        | inline  |
| WYSIWYG      | ✓        | full    |
| Preview      | ✗        | full    |
| Diff         | ✗        | both    |

---

This sample exists in the binary, so you can always come back to it via the
**Sample** button.
`;
