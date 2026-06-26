# 設計ドキュメント

## 1. 目的と非目標

### 目的
- Markdown を **見る** ことも **書く** こともできる、軽量なクロスプラットフォームアプリ
- Git 管理下のファイルでは、編集中のファイルの差分が一目でわかる
- モバイルでも快適に動作する

### 非目標（少なくとも v1 では扱わない）
- マルチカーソル、Vim/Emacs キーバインドの完全再現
- リアルタイム共同編集
- クラウド同期（端末内 + 既存 Git リポジトリへの依存のみ）
- プラグインシステム

---

## 2. アーキテクチャ概要

```
┌──────────────────────────────────────────────────┐
│  Tauri 2 シェル（Rust）                            │
│  - ファイル I/O（tauri-plugin-fs）                  │
│  - Git 操作（git2 crate）                          │
│  - 差分計算（similar crate）                        │
│  - ネイティブメニュー、ファイルダイアログ            │
└──────────────────────────────────────────────────┘
                    │  IPC commands / events
                    ▼
┌──────────────────────────────────────────────────┐
│  WebView（SvelteKit static / Svelte 5 SPA）        │
│                                                    │
│  ┌────────────────────────────────────────────┐  │
│  │ ModeBar: [Source][Preview][WYSIWYG][Diff]  │  │
│  │           Diff サブモード: [Full|Highlight] │  │
│  └────────────────────────────────────────────┘  │
│  ┌────────────────────────────────────────────┐  │
│  │ Editor Surface                              │  │
│  │  - SourceView   (CodeMirror 6)              │  │
│  │  - PreviewView  (markdown-it → HTML)        │  │
│  │  - WysiwygView  (Milkdown)                  │  │
│  │  - DiffView     (CM6 + decorations)         │  │
│  └────────────────────────────────────────────┘  │
│  ┌────────────────────────────────────────────┐  │
│  │ DocStore (Svelte store: 単一情報源)          │  │
│  │  - text: string                              │  │
│  │  - path: string | null                       │  │
│  │  - dirty: boolean                            │  │
│  │  - gitDiff: HunkSummary[] | null             │  │
│  └────────────────────────────────────────────┘  │
└──────────────────────────────────────────────────┘
```

### 単一情報源の原則
編集中のテキストは **DocStore.text** が唯一の真実。各 View はそこから派生する。

- SourceView は CodeMirror の状態と DocStore.text を双方向バインド
- PreviewView は DocStore.text から HTML を派生（読み取り専用）
- WysiwygView（Phase 2）は ProseMirror 文書と DocStore.text の同期を Milkdown が担う

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

### WYSIWYG モード（Phase 2）
- Milkdown（ProseMirror ベース）
- 内部 AST → markdown シリアライズで DocStore.text を更新
- 表記揺れ（`*foo*` vs `_foo_`）は正規化される旨を UI で明示

### Diff モード（2 サブモード）
本アプリの特色。Git 管理下のファイルを開いたときのみ有効。

| サブモード | 内容 | 用途 |
|---|---|---|
| **Full** | GitHub 風の追加（緑）・削除（赤）行 | 差分の内容を確認したいとき |
| **Highlight Only** | 変更があった行範囲をマージンの色帯で示すのみ。中身は現状のテキスト | 編集中の文脈を崩さず「どこを触ったか」だけ把握したいとき |

差分の基準：
- デフォルトは `HEAD` との差分
- 設定で `インデックス（ステージ済み）` / `作業ツリー全体` を切替可能（Phase 2）

差分計算：
- Rust 側で `similar` crate を使い行ベース diff
- 結果は `HunkSummary { startLine, endLine, kind: "added"|"removed"|"modified" }` のリストとして IPC で渡す
- Highlight Only はこれをそのまま CodeMirror デコレーションに変換
- Full は同じデータから 2 カラムレイアウトを Svelte で組み立て

---

## 4. レスポンシブ / モバイル対応

| 画面幅 | レイアウト |
|---|---|
| `≥ 1024px` | ModeBar 横並び、サイドペイン（ファイルツリー）あり |
| `768–1024px` | サイドペインはドロワー |
| `< 768px` | ModeBar をボトムタブ化、ジェスチャ（左右スワイプ）でモード切替 |

モバイル固有の留意点：
- IME（日本語入力）と CodeMirror の相性は実機検証必須
- ファイル選択は OS のドキュメントプロバイダ経由（Tauri mobile の plugin-fs）
- Git 操作はモバイルでは **読み取り（差分表示）のみ** を v1 のスコープに。コミット等は Phase 3

---

## 5. パフォーマンス指標（目安）

| 指標 | 目標 |
|---|---|
| 起動時間（Mac M1, cold） | < 500ms |
| 10 万行 MD の Source モード入力遅延 | < 50ms |
| Preview レンダリング（10 万行） | 仮想化なしで動かない想定 → 段階レンダリング |
| バイナリサイズ | Mac < 15MB, Win < 12MB |
| アイドル時メモリ | < 100MB |

施策：
- Milkdown / DiffView は遅延ロード（dynamic import）
- markdown-it の出力をワーカーで生成し UI スレッドを塞がない
- 1MB 超のファイルは Source モードで開く（Preview を抑制し選択式に）

---

## 6. ファイル / モジュール構成（予定）

```
mdv/
├── src-tauri/                     # Rust 側
│   ├── src/
│   │   ├── main.rs
│   │   ├── commands/
│   │   │   ├── fs.rs              # 読み書き
│   │   │   └── git.rs             # diff / status
│   │   └── diff.rs                # similar crate ラッパ
│   └── Cargo.toml
├── src/                           # Svelte 5
│   ├── lib/
│   │   ├── stores/doc.svelte.ts   # DocStore
│   │   ├── views/
│   │   │   ├── SourceView.svelte
│   │   │   ├── PreviewView.svelte
│   │   │   ├── WysiwygView.svelte (Phase 2)
│   │   │   └── DiffView.svelte
│   │   ├── components/
│   │   │   ├── ModeBar.svelte
│   │   │   └── FileTree.svelte
│   │   └── ipc/                   # Tauri command ラッパ
│   └── routes/+page.svelte        # シェル
├── docs/
└── package.json
```

---

## 7. リスクと対策

| リスク | 対策 |
|---|---|
| Milkdown の round-trip で MD が変質する | Phase 2 開始時に主要ケース（CJK、リスト、コードブロック）の保存検証を行う |
| Tauri mobile が Beta から Stable に上がるまでは不安定 | Desktop を先に Stable リリース、Mobile は別ブランチで追従 |
| CodeMirror 6 + 日本語 IME の挙動 | 初期段階でモバイル含め実機テスト |
| 大きな Git リポジトリでの diff 計算が遅い | git2 のフックを使い変更ファイルのみ対象に絞る |
