# Roadmap（フェーズ分割）

各フェーズの完了条件（Definition of Done）を明示。

## Phase 0 — 足場（1〜2日）
- [ ] Tauri 2 + Svelte 5 のプロジェクト雛形
- [ ] `npm run tauri dev` で空ウィンドウが起動
- [ ] CI（GitHub Actions）で Mac/Win/Linux のビルドが通る
- [ ] フォーマッタ（rustfmt, prettier）と lint 設定

**DoD**: `cargo build --release` と `npm run tauri build` がローカルで成功する。

## Phase 1 — Source / Preview / 切替（1週間程度）
- [ ] ファイルを開く・保存する（Tauri plugin-fs）
- [ ] DocStore（Svelte runes）の実装
- [ ] SourceView（CodeMirror 6 + markdown 言語）
- [ ] PreviewView（markdown-it + DOMPurify）
- [ ] ModeBar による切替
- [ ] スクロール位置の同期（行番号ベースの素朴な実装）

**DoD**: 1MB の MD ファイルを開いて編集・保存でき、Preview と Source を遅延なく往復できる。

## Phase 2 — Diff モード（1週間程度）
- [ ] Rust 側 `git_diff` コマンド（git2 + similar）
- [ ] HunkSummary 型の IPC
- [ ] DiffView の Highlight Only 表示（CM6 デコレーション）
- [ ] DiffView の Full 表示（GitHub 風）
- [ ] サブモード切替 UI

**DoD**: Git 管理下の MD を編集中に、HEAD との差分が両モードで表示される。

## Phase 3 — WYSIWYG（2週間程度）
- [ ] Milkdown 統合（Svelte ラッパ）
- [ ] DocStore との双方向バインド
- [ ] CJK / リスト / コードブロック / 画像の round-trip 検証
- [ ] 表記正規化が起きるケースのユーザ通知

**DoD**: WYSIWYG モードで編集した結果が Source モードに反映され、ファイルとして保存できる。

## Phase 4 — モバイル（並行可、別ブランチ）
- [ ] iOS / Android で起動
- [ ] レスポンシブ UI（ボトムタブ）
- [ ] OS のファイル選択経由でドキュメントを開く
- [ ] Git は読み取り（diff）のみ

**DoD**: TestFlight / 内部配布で実機動作確認。

## Phase 5 — 仕上げ
- [ ] テーマ（ライト/ダーク/エディタ配色）
- [ ] 設定画面
- [ ] 大容量ファイルガード
- [ ] エラー時の挙動（ファイル消失、Git なしリポジトリなど）
- [ ] パッケージ署名と配布
