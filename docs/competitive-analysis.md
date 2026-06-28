# mdv 競合調査レポート

調査日: 2026-06-28
方法: 21 agents による並列調査 (8 製品プロファイル + 7 観点の横断調査 +
フットプリント実測ベース推定 + 3 視点 adversarial critique)。

---

## 結論サマリ (TL;DR)

mdv は **「Tauri 2 ベースの軽量 + Git diff を一級機能として持つ Markdown
エディタ」** という、現状他に競合がほぼいないニッチを占める。Electron 系
（Typora / Obsidian / VS Code）の機能性と、ネイティブ Mac アプリ
（iA Writer / MarkEdit）に近い軽量さを両立し、加えて **GUI 5 モード +
独立 TUI バイナリ** を備える点は唯一無二。一方、**画像クリップボード貼り付け /
TOC サイドバー / Mermaid / KaTeX / アウトライン** といった他社で標準化済みの
機能が抜けており、ここを埋めない限り「使い始めの第一印象」で脱落する。
直近の優先 Top 3 は **(1) 画像クリップボード貼り付け、(2) Mermaid 描画、
(3) アウトライン / TOC サイドバー**。押し出すべき強み Top 3 は
**(1) Git-aware Diff モード（3 サブモード + base picker）、
(2) .mdv ポータブルバンドル、(3) GUI+TUI のフィーチャ対称性**。

> Note: 本レポートではレビュアーから指摘された数値の誤り（競合の RAM /
> インストールサイズ / cold start など）を可能な範囲で訂正し、出典に
> 裏付けのない箇所は「実測未確認」と明記している。

---

## 比較マトリクス

mdv が直接競合しうる、または評価時に並べられがちなツールを以下に並べる。
**MarkText は 2022 年以降ほぼ未更新**のため参考枠とし、
**AppFlowy（Tauri 系）/ Foam（VS Code 拡張）/ MarkEdit（Mac ネイティブ軽量）/
Ghostwriter（Qt ベース Linux）/ Frogmouth（モダン TUI）** を追加した。

| 機能 / 指標 | mdv | Typora | Obsidian | iA Writer | VS Code (+Foam) | AppFlowy | MarkEdit (Mac) | Ghostwriter | Frogmouth (TUI) |
|---|---|---|---|---|---|---|---|---|---|
| Framework | Tauri 2 | Electron | Electron | Native (Mac/iOS/Win/Android) | Electron | Tauri | Native (Swift/Mac) | Qt6 | Textual (Python TUI) |
| Installer (実測未確認、概算) | ~10 MB | ~80–120 MB | ~80 MB | ~50–80 MB | ~100 MB | ~30 MB | ~10 MB | ~30 MB | ~20 MB |
| Idle RAM (concrete doc) | ~80–180 MB※ | ~200–300 MB | ~250–450 MB | ~120–150 MB | ~180–300 MB | ~150 MB | ~40 MB | ~50 MB | ~25 MB |
| Cold start (warm cache, Mac arm64) | ~0.5s※ | ~1.5s | ~2.5s | ~0.4s | ~1.5–2s | ~1s | ~0.3s | ~0.6s | ~0.4s |
| Source mode (CM6) | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | — |
| Live Preview (inline) | ✓ | ✓✓ | ✓✓ (hybrid) | — | △ ext | △ | — | △ | — |
| WYSIWYG | ✓ (Milkdown) | ✓ | △ (Live Preview と統合) | — | — | ✓ | — | — | — |
| Preview pane | ✓ | — | ✓ | ✓ | ✓ | △ | ✓ | ✓ | ✓ |
| **Git Diff mode (1st class)** | ✓✓ (3 sub + base picker) | — | ◉ plugin | — | △ (Source Control + Timeline) | — | — | — | — |
| TUI バイナリ同梱 | ✓ (~2.3 MB) | — | — | — | — | — | — | — | n/a |
| Mobile (iOS/Android) | scaffold | — | ✓ | ✓ | — | ✓ | — | — | — |
| Math (KaTeX / MathJax) | — | ✓ (MathJax) | ✓ (MathJax 3) | ✓ | ◉ ext | ✓ | ✓ | ✓ | △ |
| Mermaid 図 | — | ✓ | ✓ | ✓ (v7+) | ◉ ext | ✓ | ✓ | ✓ | — |
| Wiki-link [[note]] | — | △ | ✓ | △ | ✓ (Foam) | ✓ | — | — | — |
| Backlinks | — | — | ✓ | — | ✓ (Foam) | ✓ | — | — | — |
| Plugin system | — | — | ✓ (~2,000) | — | ✓ (massive) | △ | — | — | — |
| Find & Replace + regex/case | △ (regex/case 未) | ✓ | ✓ | △ | ✓ | ✓ | ✓ | ✓ | △ |
| 外部変更検知 + dirty-aware banner | ✓✓ | △ | ✓ | ✓ | ✓ | △ | ✓ | △ | n/a |
| 5MB warn / 100MB cap | ✓ | — | — | — | — | — | — | — | — |
| 画像クリップボード貼り付け | — | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | — |
| Export: HTML/PDF/DOCX | ✓ | ✓ | ✓ | ✓ | ◉ ext | △ | ✓ | ✓ | — |
| Pandoc 経由 (LaTeX/EPUB) | — | ✓ | ◉ | — | ◉ | — | — | — | — |
| **.mdv portable bundle (履歴埋め込み)** | ✓✓ | — | — | — | — | — | — | — | — |
| Real-time 共同編集 | — | — | ◉ Relay | — | ◉ Live Share | ✓ | — | — | — |
| Cloud sync (built-in) | — (Git で代替) | — | ✓ ($) | — (iCloud) | — | ✓ | iCloud | — | — |
| AI 支援 | — | — | ◉ ext | — | ✓ (Copilot / Cursor) | △ | — | — | — |
| Multi-file workspace / file tree | — | △ | ✓ | ✓ | ✓ | ✓ | △ | ✓ | ✓ (browse) |
| TOC / outline サイドバー | — | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| Frontmatter YAML UI | — | △ | ✓ (Properties) | △ | ◉ | △ | — | — | — |
| Vim mode | TUI only | — | ◉ ext | — | ◉ ext | — | ✓ | ✓ | — |
| Spell check | — | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | — |
| git blame inline | — | — | — | — | ✓ (GitLens) | — | — | — | — |
| Markdown lint / Vale | — | — | ◉ | — | ◉ | — | — | — | — |
| Open source | (前提) | × | × | × | ✓ | ✓ | ✓ | ✓ | ✓ |

凡例: ✓ = native / ✓✓ = best-in-class / ◉ = plugin/extension / △ = partial / — = なし

※ mdv の RAM / cold start は macOS arm64 でのおおよその実測値で、
**Windows (WebView2 helper 3-5 プロセス) では 150–300 MB / 1.5-3s** が
現実的なレンジ。クロスプラットフォーム単一値で示すのは不適切なので注意。

**マトリクス改訂理由（critique 反映）:**

- VS Code を「Markdown-aware diff なし」と断じた箇所を撤回。Monaco の
  TextMate grammar が diff editor にも適用されるため diff 内でも MD
  ハイライトが効く。差別化は「Markdown-aware かどうか」ではなく
  **「Diff を一級モードとして持ち、サブモード + Git base picker を
  統合したか」** に再定義。
- Obsidian の WYSIWYG 欄を `–` から `△ (Live Preview と統合)` に修正。
- iA Writer は Mac 専用ではなく Mac/iOS/Win/Android クロスプラットフォーム。
  「native は Mac only」言説を撤回。
- VS Code 関連で Foam を独立列として扱う（wiki-link / backlinks の
  実質的な開発体験は Foam が担当）。
- Marker・MarkText は live competitor として扱わず、参考のみ。
- **「他に TUI Markdown エディタは存在しない」は誤り**。Frogmouth /
  Glow (TUI mode) / mdcat / lowdown / Helix + render-markdown.nvim が
  既存。mdv の TUI 独自性は **「TUI + 編集 + Git diff の組み合わせ」**
  に限定する。
- **棄却した critique**: 「mdv の .mdv 形式は Pandoc self-contained HTML
  と同等で unique ではない」→ 部分的に正しいが、**Git 履歴を
  round-trip 可能な HTML コメントとして埋め込み、生 Markdown としても
  valid** な点は依然唯一であるため、unique 主張は範囲を狭めて維持する。

---

## 不足機能（gap）— 取り組み価値で優先度付け

ユーザー痛みの頻度（毎回の作業で踏むか / 稀か）× 軽量哲学との適合性で並べる。

| Priority | Feature | Why（痛み頻度 / 戦略適合） | Effort | Fit |
|---|---|---|---|---|
| **P0** | 画像クリップボード貼り付け + ドラッグ&ドロップ → `.assets/` 自動保存 | Typora / Obsidian 経験者の #1 離脱要因。毎ドキュメントで踏む | M (Tauri D&D API がプラットフォーム差あり) | good |
| **P0** | TOC / アウトラインサイドバー | 全競合が実装済の標準ナビ。長文 review で頻発する痛み | S | good |
| **P0** | Mermaid 図（fenced code block） | GitHub / GitLab / Hugo / MkDocs で標準化済。技術文書で最頻出 | M (lazy load 必須、SBS diff の line anchor を壊さない設計が要) | good |
| **P1** | 数式レンダリング (KaTeX) | 技術 / 学術用途では table-stakes。fonts 含めると ~1.2 MB 解凍 | S | good |
| **P1** | Spell check (WKWebView / WebView2 native) | Mac で WebView 標準機能、Win は WebView2 で対応。レビュー作業で毎回必要 | S-M (CM6 は contentEditable ではないので別途 lint 経由) | good |
| **P1** | Find & Replace の regex / case / whole-word トグル | パワーユーザーの基本期待。CM6 `@codemirror/search` で大部分が対応済 | S (whole-word のみ regex で wrap) | good |
| **P1** | git blame inline gutter (Source / Live Preview) | 「reviewer's editor」を名乗るなら標準機能。git2 で実装可 | M | good |
| **P1** | Markdownlint 統合（外部バイナリ検出） | docs-as-code の標準ツール。レビュー文脈に直結 | S (Tauri command でバイナリ呼び出し) | good |
| **P2** | Multi-file workspace / Git repo file tree | 単一ファイル起動の速度を残しつつ opt-in 可能 | M | good |
| **P2** | Wiki-link `[[name]]` + on-demand backlinks（git ls-files / git grep ベース） | 「vault じゃなく Git が index」という mdv 流の表現として面白い | M | good |
| **P2** | カスタマイズ可能キーバインド | JSON config 先行、UI は後。CM6 が `Prec.highest` で対応 | M | good |
| **P2** | GUI Vim mode (opt-in) | dev / TUI ユーザー親和性高。`@replit/codemirror-vim` 追加のみ | S | good |
| **P2** | Frontmatter YAML 構造化エディタ | Hugo / Astro / Jekyll 利用者に刺さる | M | good |
| **P2** | broken link checker（Cmd+Shift+L で全リンクスキャン） | reviewer 機能として自然 | S (reqwest + 並列) | good |
| **P3** | Pandoc 検出ベース export (LaTeX / EPUB / ODT) | 同梱はせず PATH 検出のみ | M | ok |
| **P3** | モバイル read-only Reviewer (iOS/Android) | scaffold あり。**Working Copy が直接競合**するため「diff 専門ビューワ」に絞る | L | ok |
| **P3** | AI: 「typo / 整形ずれの差分修正提案」「画像 alt text 生成」「リポジトリのスタイルに合わせて書き換え」 | Cursor / Zed が既にやっているため差別化は厳しい。BYOK + Ollama で local-first | M | ok |
| **P3** | 構造（AST / tree-sitter）ベースの diff（difftastic 連携） | line diff の限界（表 / list 順入替）を超える本物の Markdown-aware diff | L | good (将来の差別化) |

### Philosophy に合わせて「採用しない / 制限する」もの

| 機能 | 方針 |
|---|---|
| Obsidian 風プラグインマーケットプレイス | **採用しない** — 後段「拡張性方針」参照。アイデンティティ希薄化 + セキュリティ + 巨大工数 |
| Knowledge graph 可視化 | 採用しない。PKM 方向への引きずられ。Obsidian / Logseq に勝てない |
| Cloud sync built-in | 採用しない。local-first 違反。Git + iCloud / Dropbox で代替 |
| Excalidraw / 白板 | 採用しない。Markdown インライン形式でない |
| 引用 / 参考文献 (.bib) | 採用しない。Zettlr のニッチ。Pandoc 検出で間接対応のみ |
| Real-time 共同編集 (Yjs) | 採用しない。**信号サーバ必須 = local-first 違反**。Git の async コラボ（コミット紐付けレビュースレッドのような Git-native UX）で代替検討 |

---

## 拡張性方針（追記）

レポート結論では Obsidian / VS Code 風のマーケットプレイス型プラグインを
否定したが、それは**全ての拡張性を否定する意図ではない**。
mdv が目指すのは **WezTerm / Vim 系の "設定としての柔軟性"** であり、
これは長期で残す方向。

### 否定する vs. 残す

| | Obsidian / VS Code 型 | WezTerm / Vim 型（mdv 採用） |
|---|---|---|
| 配布 | マーケットプレイス、ワンクリックインストール | 各自の dotfiles / config / プラグインマネージャ |
| 形態 | JS バンドル | Lua / Vimscript / 設定 DSL |
| 信頼境界 | サードパーティ任意コード | 自分で書く / 厳選した repo |
| 範囲 | 機能の総取り換え | 設定としての拡張 |

### 短期 (~v1.5) に出せる拡張ポイント

- **キーバインド全カスタマイズ** — JSON / TOML config 先行、UI は後追い。
  CM6 は `Prec.highest` で上書き可。preset (`vim` / `emacs` / `default`)
  を最初に提供
- **markdown-it plugin の declarative 登録** — 設定ファイルに
  `markdown.plugins: ["footnote", "table-of-contents", "mark"]`
  のような flag 形式で許可リスト方式（任意 JS は読み込まず、同梱した
  公式 plugin のみ）
- **シンタックステーマ追加** — config ディレクトリに `themes/*.json`
  (`--mdv-syntax-*` の CSS 変数を上書き) を置けば自動検出
- **Export パイプライン hook** — `export.pdf.command: "pandoc ..."` の
  ような shell コマンド連鎖。同梱せず PATH 検出のみ

### 中期 (~v2) で検討する Lua hook

- 採否は要再評価。やるなら `mlua` crate 経由で Lua 5.4 を埋め込み、
  hook API は最小に：
  - `on_save(path, text)` → 整形・lint
  - `on_open(path)` → frontmatter 初期化
  - `command(name, fn)` → コマンドパレット追加
- **明示的に引かない線**: 外部 URL から Lua / JS / Rust をロードして
  実行する仕組みは作らない。設定ファイル経由のローカル拡張のみ。

### なぜこの線引きか

- **アイデンティティ**: 「reviewer's editor」を維持しつつ "私の好み" を
  反映できる。dotfiles 文化と整合
- **セキュリティ**: ローカル設定なら supply-chain 攻撃面なし。Lua は
  ファイルアクセスを制限可能 (`mlua` の安全 mode)
- **工数**: マーケットプレイスを作らない分、~10% の労力で 80% の
  パワーユーザー満足を狙える

---

## 追加できそうな機能（候補）— 実装スケッチ付き

具体的な実装方針を 1-2 行で。優先度は前節と独立に、技術的に面白い順。

1. **Mermaid（fenced ```mermaid）**: markdown-it のカスタム fence で
   `<div class="mermaid">` を出力 → 初回遭遇時に `mermaid` を lazy import
   → `mermaid.run()`。**Diff モードでは `<pre>` のまま固定して SBS
   line-anchor を守る**（commit 27f87fc の sub-block sync を壊さない）。
   テーマは light/dark で `mermaid.initialize({theme})` 切替。
2. **KaTeX**: `markdown-it-katex` を採用。フォント 1.2 MB の解凍コスト
   考慮で、`$...$` / `$$...$$` 検出時のみ lazy load。Milkdown 側は
   math node 既存対応。Source mode は raw 表示維持。
3. **画像クリップボード貼り付け**: `tauri-plugin-clipboard-manager` で
   取得 → `${docDir}/${docBaseName}.assets/YYYY-MM-DD-HHMMSS.png` に
   保存 → `![](relative/path.png)` 挿入。**WebP / AVIF 自動変換オプション**
   を追加すれば Typora（PNG only）に対する優位点。orphan 画像
   クリーンアップは v2。
4. **TOC / アウトラインサイドバー**: markdown-it AST か CM6 syntax tree
   から heading を抽出。⌘⇧O で開閉、クリックで該当行へジャンプ
   （既存 line-tracking を再利用）。全 5 モードで共有 UI、JS 量は数 KB。
5. **Wiki-link 解決（Git scope）**: `git ls-files '*.md'` で
   `HashMap<NoteTitle, PathBuf>` を構築 → `[[name]]` をクリッカブル化。
   Backlinks は ⌘⇧B で on-demand に `git grep -l` を走らせるだけ。
   **watcher / 永続 index は持たない**。TUI でも popup で対応可能。
6. **`.mdv` ポータブルバンドルの拡張（画像同梱）**: `docs/mdv-protocol.md`
   に attachments セクション追加。AST walk で画像参照を抽出して bytes を
   zstd+base64 で埋め込み。インポート側は temp dir 展開 or `asset://`
   プロトコル経由でサーブ。**画像貼り付けと組み合わせて初めて真の
   「単一ファイルで配布可能」になる**。
7. **Spell check**: Milkdown の contentEditable には `spellcheck=true`
   属性で WKWebView / WebView2 のネイティブ機能を有効化。**CM6 (Source /
   Live) は contentEditable ではないため**、別途 `@codemirror/lint` +
   辞書（hunspell データ）で対応。Linux WebKitGTK は粒度が荒いので
   静かに degrade。
8. **GUI Vim mode**: `@replit/codemirror-vim` を CM6 に追加。Settings の
   `vimMode: false`（default）で opt-in。TUI が既に `:w :q :wq` なので
   一貫性あり。
9. **git blame gutter**: Rust 側で `git2::Blame` を計算、各行のコミット
   情報を CM6 の `lineMarkerExtension` に渡す。クリックでコミット詳細を
   popover 表示。
10. **broken link checker**: ⌘⇧L で `[text](path)` を AST 抽出 →
    ローカルパスは `tokio::fs::metadata`、URL は `reqwest::head` を並列。
    結果はサイドパネル or sticky banner。
11. **structural diff (difftastic 連携)**: 設定で
    `diff.engine: similar | difftastic` を選択可能に。difftastic が
    インストールされていれば優先使用。**「Markdown-aware diff」の主張を
    真に実現する**唯一の道。
12. **AI: 整形ずれの差分修正サジェスト**: BYOK + Ollama。Diff モードで
    「whitespace-only / heading level 変更 only」のハンクを検出し、
    適用ボタンを出すだけ。最小実装で十分に reviewer 価値が出る。
    Cursor / Zed と被らない niche。

---

## 差別化ポイント — 押し出すべき強み

mdv が押し出すべき rare / unique な機能を、**他がやっていない理由**
とセットで示す。

### 1. Git-aware Diff モードを「一級編集モード」として持つ

3 サブモード（Highlight Only / Full / Side-by-Side）+ Git base picker
（HEAD / branches / tags / recent commits with file-changed フィルタ）+
sub-block 精度の SBS スクロール同期。

**他がやらない理由**: VS Code は generic diff editor を持ち、JetBrains は
VCS と diff が一級だが、**Markdown 専用の編集体験と統合された diff モードは
存在しない**。Obsidian は community plugin、Typora にはない。GitHub /
GitLab Web UI はあるが local-first ではない。

### 2. .mdv ポータブルバンドル（履歴埋め込み）

1 つの .md に zstd+base64 圧縮した Git 履歴を HTML コメントとして
埋め込む。受信側は Git なしで本文を valid Markdown として読め、mdv なら
履歴も再生できる。

**他がやらない理由**: Pandoc `--self-contained` HTML や Quarto は近いが、
**履歴を round-trippable に埋め込みかつ生 Markdown としても valid** な
形式は前例なし。Typora / Obsidian / Joplin はファイルを vault/folder
から出すと壊れる。**注意**: 攻撃面（zstd bomb）への対策（payload size cap /
確認ダイアログ）を伴うこと。

### 3. GUI + TUI のフィーチャ対称性

GUI 5 モード（Source / Live Preview / WYSIWYG / Preview / Diff）と
独立 TUI バイナリ（Source / Preview / Diff + `:w :q :wq`）。TUI は
~2.3 MB のシングルバイナリ。

**他がやらない理由**: Glow / Frogmouth / mdcat は read-only に近く、
Helix + プラグインは general editor。**「TUI + 編集 + Git diff」を
統合して提供しているのは mdv のみ**。SSH 経由のサーバ作業や CI 環境で
生きる。

### 4. Tauri 2 × Svelte 5 × CodeMirror 6 × Milkdown の組み合わせで Electron 機能性 + ネイティブ近似フットプリント

ベンチマーク値（要再計測）: macOS arm64 で installer ~10 MB /
idle ~80 MB / cold start ~0.5s。Windows WebView2 は 150–300 MB / 1.5-3s
（cold cache）と差が出るが、Electron 系（Typora / Obsidian / VS Code）
よりは 1.5-3x 優位。

**他がやらない理由**: AppFlowy が同じ Tauri スタックで存在するが、
**diff workflow がない**。MarkEdit は Mac 専用、Ghostwriter は Qt +
Linux 中心。**「Tauri 2 で Markdown 編集 + Git diff」を満たすのは現状
mdv のみ**。

### 5. 外部ファイル変更の dirty-aware banner（VSCode-grade UX）

notify-debouncer-mini で検知、clean なら静かに reload、dirty なら
`[Revert to disk] [Compare] [Dismiss]` バナー。削除には
`[Save (recreate)] [Dismiss]`。Rust エラーを人間言語に翻訳。

**他との差**: Obsidian / Joplin も近いことはやるが、**`[Compare]` で
即座に Diff モードに飛べる**のは mdv の Diff が一級ゆえ。

### 6. ファイルサイズの安全装置（5MB warn / 100MB cap）+ 大規模ファイル耐性

これは competitor のほぼ全員がカバーしていない。Obsidian / Typora は
大きい .md でフリーズする。**「50K 行の .md を開いても落ちない」を
保証できれば、リリースノートや schema dump レビュー用途で唯一の選択肢**
になりうる（要ベンチ確立）。

### 7. **「Reviewer's Markdown Editor」というポジショニング自体**

他は全員 writer (Typora / iA Writer) か PKM hoarder (Obsidian / Logseq /
Joplin) を向いている。**「Git リポジトリ内の Markdown の変更をレビューする」**
という workflow は存在するのに、専用ツールがない。これを明示的に名乗る
ことが最大の戦略的差別化。

**critique 反映**: 「scroll position retention」を differentiator から
外した。検証不能で防御的に響くため。「Live Preview」も Obsidian と
機能領域が被るので強み列から除外し、Diff / Portable / TUI / Tauri /
Reviewer ポジションに集約。

### Reviewer ポジションの workflow trigger（具体）

抽象的な「reviewer 向け」では弱いので、具体的に開いてもらう瞬間を
3 つに絞る:

1. **巨大な生成ドキュメントの review**: GitHub PR UI が遅い / 重い 1MB+
   の Markdown（API spec / changelog / schema dump）の diff を local で
   快適に見る用途。
2. **構造変更の俯瞰**: ドキュメントツリー全体の見出し再編・章移動を
   Side-by-Side で確認。
3. **オフライン / 飛行機の中での review**: PR を `gh pr checkout` で
   落として mdv で見る。

これら 3 つに刺さる UX 強化（PR checkout integration / 「mark as reviewed」
状態 / j/k ファイル間移動）が次の小さな投資先。

---

## 軽量性ポジション

### 定量比較表（実測未確認の値は※を付与）

| Tool | Installer | Idle RAM (concrete doc) | Cold start (warm, Mac arm64) | Cold start (Win, cold cache)※ |
|---|---|---|---|---|
| **mdv (Tauri 2)** | ~10 MB | ~80 MB (Mac) / 150-300 MB (Win)※ | ~0.5s | ~1.5-3s (WebView2 init) |
| MarkEdit (Mac native) | ~10 MB | ~40 MB | ~0.3s | n/a |
| Frogmouth (TUI) | ~20 MB | ~25 MB | ~0.4s | n/a |
| Ghostwriter (Qt) | ~30 MB | ~50 MB | ~0.6s | ~0.8s |
| iA Writer (native) | ~50-80 MB | ~120-150 MB | ~0.4s | ~0.6s |
| Bear 1 (native, Mac) | ~80 MB | ~150 MB | ~0.5s | n/a |
| Typora (Electron) | ~80-120 MB | ~200-300 MB | ~1.5s | ~2.5s |
| Obsidian (Electron) | ~80 MB | ~250-450 MB | ~2.5s | ~3s |
| AppFlowy (Tauri) | ~30 MB | ~150 MB | ~1s | ~1.5s |
| VS Code (Electron, clean) | ~100 MB | ~180-300 MB | ~1.5-2s | ~2-3s |
| Zettlr (Electron) | ~150-200 MB | ~280 MB | ~2.5s | ~3s |
| Joplin (Electron) | ~150 MB | ~300-450 MB※ | ~2.5s | ~3s |
| Logseq (Electron) | ~280 MB | ~600 MB | ~2.5s | ~4s |

**critique 反映による数値訂正**:

- VS Code installer 500 MB → 100 MB（installer）/ 350 MB（拡張ロード後の on-disk）
- Logseq 565 MB → 280 MB に修正（v0.10+ で縮小）
- Typora 150 MB → 80-120 MB（platform 別）
- Obsidian 200 MB → 80 MB（installer）
- iA Writer の 25 MB / 80 MB RAM → 50-80 MB / 120-150 MB（過小評価だった）
- Joplin の 600-800 MB RAM → 300-450 MB（v2.13+ CodeMirror 6 移行で縮小）
- 「15-20x 軽い」は誇張 → **「Electron 系の 1.5-3x 軽い、ネイティブ
  Mac には及ばないが OS 横断で近づける」** が honest claim。

### Tauri 2 を選んだ意義

- **Chromium / Node を bundle せず OS WebView を再利用**: macOS
  WKWebView / Windows WebView2 / Linux WebKitGTK。これだけで Electron
  比 1.5-3x の軽量化が決まる。
- **Rust 製のため Git 統合（git2 / similar / notify）が in-process で
  完結**: Diff / 外部変更検知 / git blame など、競合が plugin で対処する
  機能を built-in で実装可能。
- **メモリプロファイルが OS で大きく異なる正直な開示**:
  - macOS WKWebView: 単一 process、~80 MB idle が現実的
  - Windows WebView2: msedgewebview2.exe ヘルパー 3-5 個を合計すると
    150-300 MB（critique 指摘どおり）
  - Linux WebKitGTK: ディストロ依存（Ubuntu 22.04 で 2.36、CSS / JS の
    機能差あり）
- **「Electron features, native weight」は macOS 限定の honest claim**。
  **Windows / Linux では「Electron 比 1.5-3x 軽量」が正確**。
  マーケティング上、後者を使い分けるべき。
- **CodeMirror 6 エコシステム**を実質無料で享受: vim / emacs binding、
  search、lint、autocomplete、collab。「実装」と言うより「設定」レベル。

### 軽量を語る上で必要な最低スペック宣言

正直さのために、サポート最低スペックを明示する（推奨）:

- macOS 11+, Apple Silicon または Intel Core i5 2015 以降
- Windows 10+ (WebView2 Runtime インストール済), 4 GB RAM 以上
- Linux glibc 2.31+, WebKitGTK 2.36+
- 4 GB RAM / 2015 era CPU で **3-5 MB Markdown まで快適**を SLA とする（要ベンチ）

---

## 実装優先度マップ — Top 10

高 impact × 低 effort を優先。「impact」は Critique で再確認された
**「実際の日次痛みの頻度」** に基づく。

| # | Feature | Impact | Effort | Notes |
|---|---|---|---|---|
| 1 | 画像クリップボード貼り付け + D&D → `.assets/` 自動保存 | 高（毎回踏む） | M | Tauri D&D API のプラットフォーム差に注意。WebP/AVIF 変換は v2。 |
| 2 | TOC / アウトラインサイドバー（⌘⇧O） | 高（長文 review で毎回） | S | heading 抽出 + line-tracking 再利用。新規依存なし。 |
| 3 | Mermaid 図（lazy load, ~2.5 MB gzipped） | 高（技術文書 default） | M | **Diff モードでは raw `<pre>` のまま**で SBS line-anchor 保護。 |
| 4 | KaTeX math（lazy load, fonts 含めて ~1.2 MB） | 中-高 | S | 全 5 モードで対応。Source は raw。 |
| 5 | Find & Replace: regex / case / whole-word | 中 | S | CM6 `@codemirror/search` 標準。whole-word は `\b` wrap で実装。 |
| 6 | Spell check（WKWebView / WebView2 + CM6 lint） | 中 | S-M | Mac/Win は ほぼ無料、Linux は degrade。 |
| 7 | git blame inline gutter | 中（reviewer 直撃） | M | git2 + CM6 lineMarker。 |
| 8 | Multi-file workspace（Git repo file tree, ⌘⇧E） | 中 | M | 非 Git repo では directory scan に fallback。 |
| 9 | Markdownlint / Vale 外部バイナリ検出 | 中（dev 直撃） | S | バンドルせず PATH 検出のみ。 |
| 10 | GUI Vim mode（opt-in, default off） | 中（dev 親和） | S | `@replit/codemirror-vim` 追加のみ。 |

**Top 10 から外したが直近の v1.x で入れる価値があるもの**: broken link
checker / Frontmatter YAML 編集 UI / カスタムキーバインド / Wiki-link +
on-demand backlinks / `gh pr checkout` 統合（reviewer ポジションを名乗る
なら強い）。

**長期 (v2+) で検討するもの**: structural diff (difftastic 連携) /
モバイル read-only Reviewer / AI 差分修正サジェスト / Pandoc 検出
export / .mdv バンドルの画像同梱拡張 / Lua hook（拡張性方針参照）。

### Critique からの数値・前提訂正サマリ（記録のため）

1. ✅ VS Code は diff editor 内でも Markdown ハイライトされる → 差別化文言を修正。
2. ✅ iA Writer はクロスプラットフォーム → 修正。
3. ✅ Obsidian Live Preview は WYSIWYG hybrid → matrix 修正。
4. ✅ Frogmouth / Glow TUI 等 TUI 競合あり → 「TUI + 編集 + Diff」に限定。
5. ✅ AppFlowy は同じ Tauri スタックで存在 → matrix に追加。
6. ✅ 各社 RAM / installer 数値を修正、cold cache vs warm cache を区別、
   macOS vs Windows の差を明示。
7. ✅ KaTeX サイズ 280KB → ~1.2 MB（fonts 込み）に訂正。
8. ✅ Mermaid 1 MB → ~2.5 MB gzipped。
9. ✅ Image paste の effort を small → medium に上方修正。
10. ✅ Mermaid と Diff SBS sync の line-anchor 競合を明示、回避策
    （Diff モードでは raw 維持）を提案。
11. ✅ .mdv portable bundle の supply-chain 攻撃面を認識、size cap /
    確認ダイアログを TODO に追加。
12. ✅ 「scroll retention as differentiator」を撤回。
13. ✅ 「narrate this diff」AI 機能の根拠不十分を認め、「整形ずれ差分の
    修正サジェスト / alt-text 生成 / リポスタイル合わせ」に置き換え。
14. ✅ Yjs リアルタイムコラボの「local-first 維持」誤主張を訂正、Git
    ベース async コラボ（コミット紐付けレビュースレッド）に方向修正。
15. ❌ 棄却: 「.mdv は Pandoc self-contained HTML と同等」→ Git 履歴
    round-trip + 生 MD valid 性は依然 unique のため、unique 主張は範囲を
    狭めて維持。
16. ❌ 棄却: 「Plugin system を即採用すべき」→ アイデンティティ希薄化
    リスク優先、代わりに「拡張性方針」セクションで WezTerm/Vim 系の
    config-driven 拡張を残す。
17. ➕ 追加: CJK / 日本語 IME 対応（Milkdown は歴史的に IME バグあり）
    を非機能要件として明記すべき。
18. ➕ 追加: アクセシビリティ（WCAG AA, screen reader）の明示目標を
    positioning に組み込み。
19. ➕ 追加: ライセンス / ビジネスモデル（OSS, 無料想定）を明示すべき
    （positioning の anchor）。
20. ➕ 追加: 大規模ファイル（50K 行）のパフォーマンス保証を測定値で示す
    と、reviewer ポジションを実証できる。

---

このレポート単体で、mdv の現在位置・直近何に投資すべきか・どこを押し出す
べきかを判断できる粒度に揃えた。次のアクションとして推奨するのは、
**(a) Top 10 のうち #1-#3（画像貼り付け / TOC / Mermaid）を v1.x の
早い段階で実装**、**(b) 軽量性ベンチを macOS / Windows / Linux で実測し
ドキュメント化**、**(c) 「reviewer's editor」を README と onboarding
文言に明示**、の 3 点。
