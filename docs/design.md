# 設計ドキュメント

## 1. 目的と非目標

### 目的
- Markdown を **見る** ことも **書く** こともできる、軽量なクロスプラットフォームアプリ
- GUI（デスクトップ / モバイル）と **TUI（端末）の両方** を一級の表示手段として提供
- Git 管理下のファイルでは、編集中のファイルの差分が一目でわかる

### 非目標（少なくとも v1 では扱わない）
- マルチカーソル、Vim/Emacs キーバインドの完全再現
- リアルタイム共同編集
- クラウド同期（端末内 + 既存 Git リポジトリへの依存のみ）
- プラグインシステム
- TUI モードでの Live Preview / WYSIWYG 編集（端末では表現困難なため）

---

## 2. アーキテクチャ概要

Cargo ワークスペースで Rust コードを 3 つの crate に分割し、GUI と TUI 双方から共通ロジックを呼ぶ。

```
┌────────────────────────────────────────────────────────────────────┐
│  mddiff-core (Rust library, no UI deps)                                │
│  - File I/O 抽象                                                     │
│  - Git diff (git2 + similar)                                        │
│  - Markdown 解析（pulldown-cmark, AST レベルの操作が必要な場合）       │
│  - 型: DocState, HunkSummary, GitContext                            │
└────────────────────────────────────────────────────────────────────┘
            ▲                                          ▲
            │ depends on                               │ depends on
            │                                          │
┌──────────────────────────────┐    ┌──────────────────────────────┐
│  mddiff-gui (= src-tauri/)       │    │  mddiff-tui                      │
│  Tauri 2 シェル                │    │  ratatui + crossterm          │
│  - IPC commands               │    │  - 直接ターミナル描画           │
│  - WebView (SvelteKit SPA)    │    │  - キーボード操作               │
│    が UI を担う                │    │  - mddiff-core を直呼びする        │
└──────────────────────────────┘    └──────────────────────────────┘
            │
            │ IPC commands / events
            ▼
┌──────────────────────────────────────────────────┐
│  WebView（SvelteKit + adapter-static, SPA）         │
│                                                    │
│  ┌────────────────────────────────────────────┐  │
│  │ ModeBar: [Source][Preview][Live][WYS][Diff] │  │
│  │           Diff サブモード: [Full|Highlight] │  │
│  └────────────────────────────────────────────┘  │
│  ┌────────────────────────────────────────────┐  │
│  │ Editor Surface                              │  │
│  │  - SourceView   (CodeMirror 6)              │  │
│  │  - PreviewView  (markdown-it → HTML)        │  │
│  │  - LivePreviewView (CM6 decorations)         │  │
│  │  - WysiwygView  (Milkdown)                  │  │
│  │  - DiffView     (CM6 + decorations)         │  │
│  └────────────────────────────────────────────┘  │
│  ┌────────────────────────────────────────────┐  │
│  │ DocStore (Svelte runes: 単一情報源)          │  │
│  │  - text: string                              │  │
│  │  - path: string | null                       │  │
│  │  - dirty: boolean                            │  │
│  │  - gitDiff: HunkSummary[] | null             │  │
│  └────────────────────────────────────────────┘  │
└──────────────────────────────────────────────────┘
```

### 単一情報源の原則
編集中のテキストは GUI 側は `DocStore.text`、TUI 側は `mddiff_tui::AppState.text` が真実。
両者とも背後の **mddiff-core** が提供する `DocState` 型を保持し、Markdown パース・差分計算結果・Git コンテキストは crate 共通の型で表現される。

GUI 側の View 同期：
- SourceView は CodeMirror の状態と DocStore.text を双方向バインド
- PreviewView は DocStore.text から HTML を派生（読み取り専用）
- LivePreviewView（Phase 3）は CodeMirror 6 decorations を使い、DocStore.text を正の source として扱う
- WysiwygView（Phase 3）は ProseMirror 文書と DocStore.text の同期を Milkdown が担う

これにより「ボタンで切り替え」がただの View 切り替えになり、モード切替時のロストを防ぐ。

---

## 3. モード仕様

### Source モード
- CodeMirror 6 + `@codemirror/lang-markdown`
- 行番号、シンタックスハイライト、ソフトラップ切替
- 検索（Ctrl/⌘+F）

### Preview モード
- markdown-it（CommonMark + GFM）
- DOMPurify でサニタイズ（任意の HTML 埋め込みに備える）
- スクロール位置を Source モードと共有（ヒューリスティック: 行ベース）

### Live Preview モード（GUI のみ、Phase 3）
- Typora 的な「編集中もレンダリング結果が見える」感覚を取り入れる
- フル WYSIWYG ではなく、Source と Preview の中間 mode として扱う
- 現在行は raw source、非アクティブ行は軽く rendered 表示
- 見出し、強調、インラインコード、リンク、リストから対応
- コードブロック、テーブル、画像は初期実装では source fallback
- 表記を勝手に正規化しない（`*foo*` / `_foo_` 等はそのまま保持）
- TUI 非対応（端末では Source / Preview 切替で代替）

### WYSIWYG モード（GUI のみ、Phase 3）
- Markdown source を意識せずに軽く直したい時の rich editing mode
- Milkdown（ProseMirror ベース）を候補にする
- 内部 AST → markdown シリアライズで DocStore.text を更新
- 表記揺れ（`*foo*` vs `_foo_`）は正規化されうるため、Source / Live Preview と役割を分ける
- Source を正の保存形式とし、WYSIWYG は編集体験の 1 mode として扱う
- TUI 非対応（端末では Source / Preview 切替で代替）

### Diff モード（3 サブモード）
本アプリの特色。Git 管理下のファイルを開いたときのみ有効。

| サブモード | 内容 | 用途 | GUI | TUI |
|---|---|---|---|---|
| **Highlight Only** | 変更があった行範囲をマージンの色帯で示すのみ。中身は現状のテキスト | 編集中の文脈を崩さず「どこを触ったか」だけ把握したいとき | ✓ | ✓ |
| **Full** | GitHub 風の追加（緑）・削除（赤）行を 1 ペインに統合表示 | 差分の内容を確認したいとき | ✓ | ✓ |
| **Side-by-Side** | OLD（HEAD）プレビューと NEW（現バッファ）プレビューを左右で並べ、変更ブロックをハイライト | レビュー的な見比べ。Markdown のレンダリング結果として「どこが見栄えに影響したか」を視覚的に把握 | ✓ | △ (簡易) |

#### Side-by-Side の詳細

GUI：
- 左ペイン = HEAD ブロブをそのまま markdown-it で HTML 化
- 右ペイン = 現在のエディタバッファを HTML 化
- 各ペインで **変更されたブロック**（段落・見出し・コードブロック等）に `mddiff-changed-{added|removed|modified}` クラスを注入し、CSS で背景色を付ける
- スクロールはまず**独立**（Phase 2.5 MVP）。両ペイン同期は Phase 5 で再検討

TUI：
- 端末では MD レンダリング 2 カラムは可読性が厳しいため、簡易版として
  **Source テキスト同士を左右に並べ、Highlight Only と同じ行ベースの色付け** を行う
- 横幅 < 100 桁の端末では Side-by-Side サブモードは自動で無効化（Full にフォールバック）

#### ハイライトの実装方針（GUI）

「どの行範囲が、レンダー後の HTML のどのブロックに対応するか」のマッピングが必要。
**markdown-it の token.map を使う方式（外部依存なし）** を採用：

1. `md.parse(text, env)` でトークン列を取得（各トークンの `token.map = [start_line, end_line]` あり）
2. トップレベルトークン（block_open 系）を走査し、map がハンクの行範囲と重なるかチェック
3. 重なる場合 `token.attrJoin("class", "mddiff-changed-{kind}")` でクラスを追加
4. `md.renderer.render(tokens, md.options, env)` で HTML 化

長所：
- 外部プラグイン不要
- ブロック単位ハイライトで構造を壊さない
- 段落・見出し・リスト・コードブロック・テーブル等にそのまま効く

短所：
- 段落内の数文字変更でも段落丸ごとが色付く（文字単位差分は Phase 5 以降）

#### 差分の基準
- デフォルトは `HEAD` との差分
- 設定で `インデックス（ステージ済み）` / `作業ツリー全体` を切替可能（Phase 5）

#### データモデル（HunkSummary 拡張）

Side-by-Side のために、ハンクは **OLD と NEW 両方の行範囲** を持つ必要がある：

```rust
pub struct HunkSummary {
    pub kind: HunkKind,        // Added | Modified | Removed
    pub new_start: usize,      // 1-based、空範囲は 0
    pub new_end: usize,
    pub old_start: usize,
    pub old_end: usize,
}
```

派生：
- `removed_count()` = `old_end - old_start + 1` if non-empty
- Added: `old_start = old_end = 0`
- Removed: `new_start = new_end = 0`、`old_*` が削除範囲
- Modified: 両方 non-empty

既存の Highlight Only / Full もこの新型から派生する。

#### 差分計算
- `mddiff-core` で `similar` crate を使い行ベース diff、`git2` で HEAD ツリーを取得
- 結果は `HunkSummary` のリスト
- GUI: IPC で渡し、Highlight Only は CSS 色帯、Full は 4 カラム描画、Side-by-Side は markdown-it token.map 経由でクラス注入
- TUI: 同じ型を ratatui で描画

---

## 4. TUI モード

### コマンド体系

| コマンド | 挙動 |
|---|---|
| `mddiff` | GUI 起動（ファイル未指定） |
| `mddiff path/to/file.md` | GUI でそのファイルを開く |
| `mddiff-tui` | TUI 起動、カレントディレクトリのファイル選択 |
| `mddiff-tui path/to/file.md` | TUI でそのファイルを開く |
| `mddiff-tui --mode preview file.md` | 初期モードを指定（source / preview / diff） |
| `mddiff-tui --read-only file.md` | 読み取り専用 |
| `mddiff-tui --diff-base HEAD~1 file.md` | 差分基準を指定 |

### TUI Diff サブモード

GUI と同じ Diff サブモードを Tab / Ctrl+D で巡回：
- **Highlight Only**：行番号脇の `▎` 色帯と本文の色付け
- **Full**：旧/新行番号 + +/− + 本文の単一カラム
- **Side-by-Side**（簡易版）：Source テキストを左右に並べ、ハンクで色付け
  （MD レンダリングはせず raw テキスト）。横幅 < 100 桁の端末では非表示

### TUI レイアウト

```
┌─ mddiff-tui  README.md  [modified]  ─────────────────────────────┐
│ Mode: [Source] Preview Diff(Full|Highlight)         q: quit   │
├───────────────────────────────────────────────────────────────┤
│  1 │ # Title                                                  │
│  2 │                                                          │
│  3 │ Some text                                                │
│ ●4 │ Edited line                                              │   ← Highlight Only: 行番号脇の色帯
│  5 │ More text                                                │
│                                                                │
└─ INSERT │ Ln 4, Col 12 │ branch: main │ +12 -3 ───────────────┘
```

- フッターにステータス（モード, カーソル位置, Git ブランチ, 差分数）
- ヘッダーに ModeBar + ファイル名 + ヘルプ短縮表示

### キーバインド（初期案）

| キー | 動作 |
|---|---|
| `Tab` | 次のモードに切替 |
| `Shift+Tab` | 前のモードに切替 |
| `Ctrl+S` | 保存 |
| `Ctrl+O` | ファイルを開く |
| `Ctrl+Q` / `q`（コマンドモード時） | 終了 |
| `Ctrl+D` | Diff サブモード切替（Full ↔ Highlight Only） |
| `:` | コマンドプロンプト（vim 風、`:w`, `:q`, `:e file` など最小限） |

Source モードでは挿入モード（テキスト入力）が基本、`Esc` でコマンドモード。
Vim 完全互換は非目標、視覚的な操作感の借用に留める。

### TUI ライブラリ

| 役割 | crate |
|---|---|
| メインフレーム | **ratatui** 0.29+ |
| 端末バックエンド | **crossterm** 0.28+ |
| Source モード編集 | **edtui** または最小自作（要評価） |
| Markdown → ANSI 描画 | **termimad** または `pulldown-cmark` から自前変換 |
| 引数解析 | **clap** v4 (derive macro) |

### TUI 性能目標

| 指標 | 目標 |
|---|---|
| 起動時間（ファイル指定あり） | < 100ms |
| バイナリサイズ（リリース） | < 5MB（strip & lto 後） |
| アイドル時 CPU | 実質ゼロ（イベントドリブン） |

GUI に比べて非常に軽い起動が可能なので、`grep -l "TODO" *.md | xargs mddiff-tui` のような使い方が現実的。

---

## 5. レスポンシブ / モバイル対応

| 画面幅 | レイアウト |
|---|---|
| `≥ 1024px` | ModeBar 横並び、サイドペイン（ファイルツリー）あり |
| `768–1024px` | サイドペインはドロワー |
| `< 768px` | ModeBar をボトムタブ化、ジェスチャ（左右スワイプ）でモード切替 |

モバイル固有の留意点：
- IME（日本語入力）と CodeMirror の相性は実機検証必須
- ファイル選択は OS のドキュメントプロバイダ経由（Tauri mobile の plugin-fs）
- Git 操作はモバイルでは **読み取り（差分表示）のみ** を v1 のスコープに。コミット等は将来
- mddiff-tui バイナリはモバイルでは配布しない（端末という前提が成り立たないため）

---

## 6. パフォーマンス指標（目安）

| 指標 | 目標 |
|---|---|
| 起動時間（Mac M1, cold） | < 500ms |
| 10 万行 MD の Source モード入力遅延 | < 50ms |
| Preview レンダリング（10 万行） | 仮想化なしで動かない想定 → 段階レンダリング |
| バイナリサイズ | Mac < 15MB, Win < 12MB |
| アイドル時メモリ | < 100MB |

施策：
- LivePreviewView / WysiwygView / DiffView は遅延ロード（dynamic import）
- markdown-it の出力をワーカーで生成し UI スレッドを塞がない
- 1MB 超のファイルは Source モードで開く（Preview を抑制し選択式に）

---

## 7. ファイル / モジュール構成（予定）

Cargo ワークスペースとして Rust コードを 3 crate に分割。Tauri 既存の `src-tauri/` は名前を保ったままワークスペースの 1 メンバとする（Tauri ツーリングの慣習を壊さないため）。

```
mddiff/
├── Cargo.toml                     # ワークスペースルート
├── src-tauri/                     # ワークスペースメンバ（= mddiff-gui）
│   ├── Cargo.toml                 # name = "mddiff"
│   ├── src/
│   │   ├── main.rs
│   │   ├── lib.rs
│   │   └── commands/              # IPC ハンドラ
│   │       ├── fs.rs
│   │       └── git.rs
│   ├── capabilities/
│   └── tauri.conf.json
├── crates/
│   ├── mddiff-core/                  # UI 非依存の純粋ロジック
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── doc.rs             # DocState 型
│   │       ├── diff.rs            # similar + HunkSummary
│   │       └── git.rs             # git2 ラッパ
│   └── mddiff-tui/                   # TUI バイナリ
│       ├── Cargo.toml             # name = "mddiff-tui"
│       └── src/
│           ├── main.rs            # clap で引数解析
│           ├── app.rs             # AppState, イベントループ
│           ├── views/             # source / preview / diff
│           └── input.rs           # キーバインド
├── src/                           # Svelte 5 (SvelteKit)
│   ├── lib/
│   │   ├── stores/doc.svelte.ts   # DocStore
│   │   ├── views/
│   │   │   ├── SourceView.svelte
│   │   │   ├── PreviewView.svelte
│   │   │   ├── LivePreviewView.svelte (Phase 3)
│   │   │   ├── WysiwygView.svelte (Phase 3)
│   │   │   └── DiffView.svelte
│   │   ├── components/
│   │   │   ├── ModeBar.svelte
│   │   │   └── FileTree.svelte
│   │   └── ipc/                   # Tauri command ラッパ
│   └── routes/+page.svelte        # シェル
├── docs/
└── package.json
```

ワークスペース化の利点：
- `mddiff-core` を共通の型・ロジックの単一情報源にできる
- `cargo build -p mddiff-tui --release` で TUI だけ単独配布可能（cargo install / Homebrew formula）
- CI で個別にテスト・ビルド可

---

## 8. リスクと対策

| リスク | 対策 |
|---|---|
| Live Preview の decorations が入力を阻害する | 現在行は raw source に戻し、複雑な構文は source fallback にする |
| Milkdown の round-trip で MD が変質する | Phase 3B 開始時に主要ケース（CJK、リスト、コードブロック）の保存検証を行い、正規化が起きる場合は通知する |
| Tauri mobile が Beta から Stable に上がるまでは不安定 | Desktop を先に Stable リリース、Mobile は別ブランチで追従 |
| CodeMirror 6 + 日本語 IME の挙動 | 初期段階でモバイル含め実機テスト |
| 大きな Git リポジトリでの diff 計算が遅い | git2 のフックを使い変更ファイルのみ対象に絞る |
| TUI 編集ウィジェット（edtui 等）の日本語幅計算が壊れがち | 早期に CJK + 絵文字テストを書き、必要なら自作にフォールバック |
| GUI と TUI で挙動・キーバインドの一貫性を保つコスト | `mddiff-core` を介し共通ロジックは 100% 共有、UI 固有はモード単位で実装方針を別途定める |
| Side-by-Side のブロック単位ハイライトが「段落内の一字違い」を段落全体で塗ってしまう | Phase 5 で文字単位 inline diff のオプションを検討。MVP では許容 |
| Side-by-Side の HunkSummary API 拡張が既存 Highlight Only / Full の実装に波及 | Phase 2.5 の最初に diff モジュールを集中改修、テストで全 3 サブモードを検証 |
