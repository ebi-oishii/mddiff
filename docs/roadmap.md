# Roadmap（フェーズ分割）

各フェーズの完了条件（Definition of Done）を明示。

## Phase 0 — 足場 ✓ 完了
- [x] Tauri 2 + SvelteKit (SPA) のプロジェクト雛形
- [x] `npm run check` と `cargo check` が通る
- [x] CI（GitHub Actions, Mac/Win/Linux）

## Phase 0.5 — ワークスペース化 ✓ 完了
- [x] ルート `Cargo.toml` でワークスペース定義
- [x] `crates/mddiff-core/` を新設し型と共通ロジックを配置
- [x] `src-tauri/` から `mddiff-core` を path 依存
- [x] `crates/mddiff-tui/` のスケルトン（clap + ratatui Hello）
- [x] `cargo build --workspace` 成功、CI を workspace 対応

## Phase 1 — Source / Preview / 切替 ✓ 完了
### GUI ✓
- [x] ファイル開閉（custom Rust command + plugin-dialog）
- [x] DocStore（Svelte 5 runes）
- [x] SourceView（CodeMirror 6）
- [x] PreviewView（markdown-it + DOMPurify）
- [x] ModeBar 切替、⌘O/S/1/2 のショートカット
- [ ] スクロール位置同期（行番号ベース） — Phase 5 へ先送り

### TUI ✓
- [x] clap + crossterm + ratatui の足場
- [x] SourceView（tui-textarea 0.7）
- [x] PreviewView（pulldown-cmark → ratatui Text 自作レンダラ）
- [x] Ctrl+E でモード巡回、Ctrl+S 保存、Ctrl+Q 終了
- [ ] `:w` `:q` コマンドモード — Phase 5 へ先送り

## Phase 2 — Diff モード（Highlight Only / Full） ✓ 完了
- [x] `mddiff-core` に `line_diff` / `full_diff` / git ラッパ
- [x] GUI DiffView（Highlight Only と Full、debounce 自動更新）
- [x] TUI DiffView（同上、ratatui で色付け）
- [x] ModeBar に Diff タブ、Git 配下でないと disabled

## Phase 2.5 — Side-by-Side Diff ✓ 完了

### 共通基盤 ✓
- [x] `HunkSummary` を `{ kind, new_start, new_end, old_start, old_end }` に拡張
- [x] 既存 Highlight Only / Full の派生計算を新フィールド経由に置換
- [x] removed_count() / added_count() ヘルパ追加
- [x] テスト追加（Added/Modified/Removed の両側行範囲、複数行置換を検証）

### GUI Side-by-Side ✓
- [x] `mddiff-core::git::side_by_side_against_base` で 1 IPC ラウンドにまとめ
- [x] Tauri command `git_side_by_side`
- [x] markdown-it の token.map をフックして `mddiff-changed-{kind}` クラスを注入
- [x] `views/diff/SideBySideView.svelte`：2 ペイン、独立スクロール、
  PreviewView と統一感のあるスタイル + 変更ブロック背景色
- [x] DiffView サブトグルに「Side-by-Side」追加

### TUI Side-by-Side（簡易版） ✓
- [x] `views/diff.rs` に `Submode::SideBySide` を追加
- [x] 横幅 < 100 桁ではメッセージ表示にフォールバック
- [x] 左ペイン = OLD Source（Removed 赤帯 / Modified 黄帯 / 「N lines added」マーカー）
- [x] 右ペイン = NEW Source（Added 緑帯 / Modified 黄帯 / 「N lines removed」マーカー）
- [x] スクロールは両ペイン共通（MVP）。独立スクロールは Phase 5

## Phase 3 — Live Preview + WYSIWYG（GUI）
Live Preview と WYSIWYG は両方入れる。ただし役割を分ける。

- Live Preview: Markdown source を保ったまま、編集中も rendered result に近い見え方にする
- WYSIWYG: Markdown source を意識せずに軽く直したい時の rich editing mode

### Phase 3A — Live Preview ✓ 完了
- [x] Live Preview mode を追加（Source / Live Preview / WYSIWYG / Preview / Diff）
- [x] 現在行 / 選択範囲のかかる行は raw source、それ以外は inline render
- [x] 見出し、強調（bold/italic）、インラインコード、リンクに対応
- [x] コードブロック、テーブル、画像、引用、リストは raw のまま fallback
- [x] 表記正規化をしない（CodeMirror 上の decoration のみで source は無加工）
- [x] EditorView.atomicRanges で隠した範囲をカーソル移動で一気に飛ぶ
- [ ] Source / Preview とのスクロール位置・選択位置の移行 — Phase 5 へ
- [ ] CJK / IME 入力での検証 — ユーザー実機確認

### Phase 3B — WYSIWYG ✓ 完了
- [x] Milkdown 7.21.2 を framework-agnostic に統合
- [x] DocStore との双方向バインド（listenerCtx + replaceAll）
- [x] 起動時に getMarkdown で正規化結果を検査、差分があれば banner 通知
  + 正規化後の text を upstream に push
- [x] Source / Live Preview / WYSIWYG / Preview / Diff の切替で内容保持
  （DocStore.text 経由）
- [ ] CJK / リスト / コードブロック / 画像の round-trip 詳細検証 — ユーザー実機確認

**DoD**: Live Preview では source 表記を保持して編集でき、WYSIWYG では rich editing から Markdown として保存できる。✓

## Phase 4 — モバイル（並行可、別ブランチ）
- [x] Android scaffold (src-tauri/gen/android, gitignore 済み)
- [ ] iOS scaffold — cocoapods 要インストール後にユーザー実行
- [x] レスポンシブ UI（ヘッダ wrap、ModeBar 横スクロール、
  Side-by-Side を狭幅で縦 2 段）
- [x] git 系コマンドが無い環境での graceful degradation
- [ ] 実機 / シミュレータでの動作確認 — ユーザー実行
- [ ] ボトムタブ UI（より深いモバイル体験） — Phase 5 へ送る
- [ ] OS のファイル選択経由でドキュメントを開く — 実機検証で確認
- [ ] Git は読み取り（diff）のみ — desktop 側で確認、モバイルでは disabled
- ※ TUI バイナリはモバイル配布対象外

**DoD**: TestFlight / 内部配布で実機動作確認。実機ステップは
ユーザー側で `npm run tauri android dev` / `... ios dev` を実行。

## Phase 8 — Settings + テーマ ✓ 完了（MVP）
- [x] Settings store（Svelte 5 runes + localStorage 永続化）
- [x] SettingsDialog（⚙ ボタンから起動、変更は即反映）
- [x] テーマ切替：auto（OS 追従） / light / dark
- [x] エディタ font size：small / medium / large
- [x] Default mode on open
- [x] 「Restore defaults」で設定リセット

未対応の細かい項目は [docs/issues.md](issues.md) に集約

## Phase 7 — `.mddiff` パッケージ入出力 ✓ 完了（MVP）

仕様: [`docs/mddiff-protocol.md`](mddiff-protocol.md)

- [x] `mddiff-core::pack::pack(file, current_text, base)`：HEAD → base の
  履歴を bundle 化、現在テキストを virtual head として追加、
  zstd+base64 で HTML コメント包装
- [x] `mddiff-core::pack::extract_body`：受信側で trailing package block を剥がす
- [x] Tauri command `mddiff_pack` / `mddiff_extract_body`
- [x] GUI: MddiffExportDialog で base picker + Save .mddiff、Export ▾ から起動
- [x] 受信: `.mddiff` を Open すると自動で本文だけを表示（履歴 bundle は破棄）
- [x] ファイル名フィルタに `.mddiff` を追加

未対応 (後続候補):
- [ ] 厳密 JCS 準拠（現在は serde_json + BTreeMap で sorted keys）
- [ ] received `.mddiff` の履歴を「versions」ビューで閲覧
- [ ] 2 つの `.mddiff` の bundle マージ（同一 repo_id）
- [ ] checkpoint / sidecar 履歴の永続化
- [ ] TUI からの pack/extract

## Phase 6 — Export to other formats ✓ 完了
GUI から複数形式へのエクスポート機能。`src/lib/export.ts` で共通化。

- [x] HTML — markdown-it + DOMPurify、埋め込み CSS でスタンドアロン
  .html を生成
- [x] PDF — オフスクリーン iframe + window.print()、OS の「PDF として
  保存」ダイアログを利用（追加依存ゼロ、3 OS 共通）
- [x] Plain text — markdown-it のトークンを walk して構造を残しつつ
  inline 装飾を除去（リスト・見出し・コードブロック・引用を保持）
- [x] DOCX — @turbodocx/html-to-docx を dynamic import、base64 経由の
  Tauri command で binary 書き出し
- [x] Export ▾ dropdown UI、外側クリックで自動クローズ
- [x] 既存ファイル名から拡張子を差し替えた default filename を提案

未対応:
- [ ] TUI からの export（GUI のみ。`:export <format> <path>` は将来検討）
- [ ] EPUB / その他フォーマット

## Phase 5 — 仕上げ
- [ ] テーマ（ライト/ダーク手動切替、エディタ配色、TUI は ANSI カラースキーム）
  ※ 現状 OS の color-scheme に自動追従
- [ ] 設定画面
- [x] 大容量ファイルガード（5MB 超は拒否）
- [x] TUI Diff キャッシュ（フレームごとの再計算を回避）
- [x] TUI コマンドモード（:w :q :wq :q!）
- [ ] 残り先送り項目:
  - [ ] GUI スクロール同期（Source / Preview / Live Preview 間）
  - [ ] Side-by-Side の文字単位 inline diff（現状はブロック単位）
  - [ ] GUI Side-by-Side のペイン間スクロール同期
  - [ ] TUI Side-by-Side の左右独立スクロール
  - [ ] エラー時の詳細ハンドリング（ファイル消失検知、再読込ボタン等）
- [ ] WYSIWYG タスクリストのクリック toggle
- [ ] パッケージ署名と配布（GUI: dmg/msi/AppImage、TUI: cargo install / Homebrew formula）
