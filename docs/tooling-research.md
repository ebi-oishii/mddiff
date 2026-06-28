# 既存ツール調査メモ

目的は競合分析ではなく、mddiff の使用感を良くするために既存ツールから学ぶことを整理すること。
mddiff は「Markdown IDE」や「ノート管理アプリ」ではなく、1 つの Markdown を静かに読む・少し直す・
差分を見るための軽量 viewer / reviewer として考える。

調査日: 2026-06-27

## 1. 方向性

mddiff の中心価値:

- 1 ファイルをすぐ開ける
- workspace / vault / project を要求しない
- 画面が静か
- Preview / Source / Live Preview / WYSIWYG / Diff の切り替えが軽い
- Git 管理下なら差分が自然に見える
- GUI と TUI/CLI の両方で同じ core を使える
- 必要なら `.mddiff` package として履歴ごと渡せる

既存ツールのいいところは取り入れるが、mddiff は多機能化で勝負しない。

## 2. 調査対象と学ぶ点

| Tool | 近い点 | mddiff に取り入れたい点 | mddiff では避けたい点 |
|---|---|---|---|
| VS Code Markdown | Markdown preview、Git diff、拡張性 | Preview / Source / Diff を滑らかに切り替える。Git 管理下で自然に diff へ行ける | workspace、拡張、サイドバー、ステータス過多 |
| GitHub rich diff | Markdown の rendered diff | prose diff / rendered diff を重要機能にする。source diff と rendered diff を切替可能にする | PR / Web / repository 前提 |
| Obsidian | local Markdown、version history、ファイル資産 | local-first、ファイルが主役、履歴の価値 | vault、graph、backlink、plugin 管理 |
| Marked | Markdown preview / review / export | 「好きな editor で書いて、mddiff で確認する」使い方。previewer としての静けさ | export / validation / advanced tooling の肥大化 |
| iA Writer | 静かな writing 環境 | 余計な UI を消す。読む/書く領域を主役にする | writing app 化、文体支援や集中モードの作り込みすぎ |
| Typora | live preview 的な自然さ | Source と Preview の心理的距離を縮める。編集中も rendered result が見える | Source を隠しすぎて Markdown の制御感を失うこと |
| Milkdown / ProseMirror 系 | Markdown-aware WYSIWYG | Markdown を意識せず軽く直せる mode。Source / Live Preview と切替できる rich editing | WYSIWYG だけを正とする設計、表記正規化の無自覚な発生 |
| ghostwriter | distraction-free Markdown | focus した画面、余白、最小 chrome | writing 機能中心に寄りすぎる |
| Zettlr | Markdown + research / publishing | document-first、ローカルファイル尊重 | citation、project、publishing 方向への拡張 |
| Glow | terminal Markdown reader | TUI で Markdown を気持ちよく読む。pager 的操作、stdin/file 対応 | ノート stash / discovery のような管理機能 |
| delta | terminal Git diff pager | terminal diff の色、行内強調、読みやすい wrapping | Git pager 専用化 |
| lazygit | terminal Git UI | keyboard driven な diff navigation、pane 切替 | Git client 化 |
| difftastic | syntax-aware diff | Markdown 構造差分や noise reduction の将来参考 | 言語構造 diff を初期 MVP に入れる |

## 3. 取り入れるべき使用感

### 3.1 Single File First

VS Code / Obsidian と逆に、mddiff は「開いた 1 ファイル」が常に主役。

- 起動直後に本文を表示する
- ファイルツリーを常設しない
- タブを増やさない
- workspace や vault を作らせない
- 最近使ったファイルや履歴は必要時だけ出す

### 3.2 Quiet Chrome

iA Writer、ghostwriter、Marked から学ぶ点。

- ツールバーは最小限
- 常時表示はファイル名、dirty 状態、mode 程度
- Git branch、差分件数、履歴状態は Diff mode または status popover でだけ見せる
- 設定画面を前面に出さない
- plugin system は作らない

### 3.3 Preview As A First-Class Mode

Markdown は source だけでなく、読む文書として扱う。

- Preview を編集の副産物にしない
- `mddiff file.md` は Preview または前回 mode で開く
- Source へ切り替えたときにスクロール位置をなるべく維持する
- TUI でも Preview が読めることを重視する

### 3.4 Live Preview And WYSIWYG As Separate Modes

Typora から取り入れたいのは、Markdown source と rendered result の距離が近い感覚。
一方で、Markdown source を意識せずに軽く直せる WYSIWYG mode も欲しい。
両者は同じものにせず、用途で分ける。

- Source mode は raw Markdown を正確に編集できる場所として残す
- Preview mode は読み専用として残す
- Live Preview mode は「現在行は source、周辺は rendered」に近い体験を目指す
- WYSIWYG mode は Markdown 記法を意識せずに rich editing したい時に使う
- 見出し、強調、リンク、リスト、コードブロックなど、効果が大きい構文から段階的に inline render する
- Live Preview では表記を勝手に正規化しない。`*foo*` と `_foo_` は source のまま保持する
- WYSIWYG では表記正規化が起こりうるため、切替時に破壊的変更を避け、必要なら通知する
- Live Preview が不安定な構文では source 表示へ fallback する
- TUI では Live Preview を無理に再現しない。Preview と Source の切替で十分とする

Live Preview と WYSIWYG はどちらも「静かに読む/少し直す」体験を強くするための機能。
Source を正の保存形式として保ち、ユーザーが mode を選べるようにする。

### 3.5 Diff As A Reading Mode

Git GUI ではなく Markdown reviewer として diff を見せる。

- Source diff: raw 変更確認
- Highlight diff: 現在文書上で変更箇所だけ示す
- Rendered diff: 読み物として何が変わったかを見る
- Side-by-side は必要な時だけ。常時 2 pane にしない
- 行単位だけでなく、将来的には見出し/段落単位の変更一覧を出す

GitHub の rendered prose diff は重要な参考。mddiff ではローカル・軽量・TUI でも使える方向に落とす。

### 3.6 CLI/TUI Should Feel Native

Glow、delta、lazygit から学ぶ点。

- `q` で閉じる
- `j/k`, arrow, page up/down で移動
- `/`, `n`, `N` で検索
- `?` で help
- `Tab` または `Ctrl+E` で mode 切替
- `mddiff cat`, `mddiff diff`, `mddiff flatten`, `mddiff pack` を shell script から使える

TUI は GUI の劣化版ではなく、端末で自然に使える viewer / reviewer にする。

## 4. 取り込まない方がよいもの

mddiff の使い心地を壊しやすい方向:

- ファイルツリー常設
- vault / workspace / project 概念
- graph view
- backlink / daily note
- plugin marketplace
- full Git client
- publishing / citation manager
- AI chat UI の内蔵
- WYSIWYG だけに寄せて Source を二級扱いにする
- 設定項目の大量追加

これらは便利だが、VS Code / Obsidian / Zettlr に近づきすぎる。

## 5. mddiff に落とす機能候補

### Phase A: 使用感の核

- 起動を速くする
- Preview / Source / Diff の 3 mode を磨く
- Git 管理下なら Diff mode を自然に有効化
- 非 Git なら Diff mode は静かに disabled
- ファイルツリーなしの 1 file UI
- TUI preview と TUI diff を実用レベルにする

### Phase B: Review 強化

- Rendered diff
- 見出し単位の変更一覧
- 変更ブロック間ジャンプ
- diff base selector
- source/rendered/highlight の diff submode
- Live Preview mode の MVP（GUI）
- WYSIWYG mode の MVP（GUI）

### Phase B.5: Live Preview / WYSIWYG

- 現在行は source 表示、非アクティブ行は軽く rendered 表示
- 見出し、強調、インラインコード、リンク、リストから対応
- コードブロック、テーブル、画像は無理に inline 編集せず fallback
- Source と Preview への即時切替を維持
- Live Preview は表記正規化をしない
- WYSIWYG は別 mode として追加し、Markdown 記法を意識しない軽い編集に使う
- WYSIWYG の round-trip で source が変わる場合は通知する

### Phase C: Handoff

- `.mddiff` package
- `pack`, `unpack`, `cat`, `flatten`, `verify`
- 履歴つき共有時の warning
- 履歴なし export

### Phase D: Optional

- `ai-context`
- token budget 付き context export
- commit summary / changed sections
- compact / redact history

## 6. プロダクト文言案

短い説明:

> mddiff は、1 つの Markdown を静かに読んで、少し直して、差分を見るための軽量 viewer / reviewer。

`.mddiff` を含める説明:

> mddiff は、Git やクラウドなしでも Markdown の履歴を package として持ち運べる。

より CLI 寄り:

> mddiff sits between `less`, Markdown preview, and `git diff`.

## 7. Sources

- VS Code Markdown: <https://code.visualstudio.com/docs/languages/markdown>
- GitHub non-code file diffs / prose documents: <https://docs.github.com/en/repositories/working-with-files/using-files/working-with-non-code-files>
- Obsidian: <https://obsidian.md/>
- Obsidian Sync version history: <https://help.obsidian.md/Obsidian+Sync/Version+history>
- Marked: <https://markedapp.com/>
- iA Writer: <https://ia.net/writer>
- Typora: <https://typora.io/>
- Milkdown: <https://milkdown.dev/>
- ghostwriter: <https://ghostwriter.kde.org/>
- Zettlr: <https://zettlr.com/>
- Glow: <https://github.com/charmbracelet/glow>
- delta: <https://github.com/dandavison/delta>
- lazygit: <https://github.com/jesseduffield/lazygit>
- difftastic: <https://difftastic.wilfred.me.uk/>
