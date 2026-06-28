# 取り組み候補リスト

ロードマップで先送りした項目、既知の小バグ、UX 改善、将来の機能候補を
集約。優先度はざっくり ★★★（高）/ ★★（中）/ ★（低 or 大物）。

最終更新: 2026-06-28

---

## バグ・既知の問題

| # | 内容 | 優先 |
|---|---|---|
| ~~B1~~ | ~~WYSIWYG のタスクリストはチェックボックスを CSS のみで描画しており、クリックで toggle できない~~ ✓ 完了（F1 で実装、クリックで checked attr を toggle） | ★★ |
| B2 | Side-by-Side のハイライト粒度はブロック単位（段落内 1 字違いも段落全体が塗られる） | ★★ |
| B3 | WYSIWYG 正規化で `savedText` が disk の実バイト列と一時的にずれる（既知のトレードオフ）| ★ |
| B4 | Android 実機 / エミュレータ起動後のレイアウトが崩れている | ★★ |
| B5 | iOS scaffold が cocoapods 未インストールで未生成 | ★ |

## UX ポリッシュ

| # | 内容 | 優先 |
|---|---|---|
| ~~P1~~ | ~~Source ↔ Preview ↔ Live Preview ↔ WYSIWYG の切替時にスクロール位置を維持~~ ✓ 完了 | ★★★ |
| ~~P2~~ | ~~GUI Side-by-Side のペイン間スクロール同期~~ ✓ 完了（toggle 付き） | ★★ |
| P3 | TUI Side-by-Side の左右ペイン独立スクロール | ★★ |
| ~~P4~~ | ~~エラーハンドリング詳細化（ファイル消失検知、再読込ボタン）~~ ✓ 完了（消失検知は P9 経由、Reload from disk + ⌘⇧R、エラー翻訳） | ★★ |
| ~~P5~~ | ~~大容量ファイル制限を「警告 + override」に変更~~ ✓ 完了（GUI: 5MB 超でモーダル / TUI: `--force`、100MB は両方 hard cap） | ★ |
| ~~P6~~ | ~~Preview / Live Preview / Diff にも検索（⌘F）~~ ✓ 完了（Source/Live は CodeMirror search、Preview/Diff は DOM find） | ★★ |
| P7 | モバイル：ボトムタブ UI、左右スワイプでモード切替 | ★★ |
| ~~P8~~ | ~~unsaved 状態でアプリ / ウィンドウを閉じようとしたら確認~~ ✓ 完了（OS の close 要求すべてに発火） | ★★ |
| ~~P9~~ | ~~外部エディタで開いてるファイルが変更されたら検知して再読込を提案~~ ✓ 完了（clean=silent reload / dirty=banner / Compare / 削除検知 / Settings 切替） | ★ |

## 機能追加

| # | 内容 | 優先 |
|---|---|---|
| ~~F1~~ | ~~WYSIWYG タスクリストのクリック toggle~~ ✓ 完了（NodeView ではなく container 経由の click handler + ProseMirror setNodeMarkup） | ★★ |
| ~~F2~~ | ~~Settings に項目追加：Soft wrap、行番号 on/off、タブ幅、Diff debounce、Side-by-Side default~~ ✓ 完了（5 項目すべて + CM Compartment で hot-reload） | ★★ |
| F3 | エディタ配色テーマ（GitHub Light/Dark, Solarized, Dracula 等） | ★ |
| F4 | TUI コマンドモード拡張：`:e <path>`、`:w <path>`、`:export <fmt> <path>` | ★★ |
| F5 | TUI 検索（`/`, `n`, `N`） | ★★ |
| F6 | TUI ヘルプ（`?`） | ★ |
| F7 | TUI からの export (`:export html <path>` 等) | ★ |
| F8 | 自動更新通知（Tauri updater） | ★ |

## `.mdv` プロトコル発展

| # | 内容 | 優先 |
|---|---|---|
| M1 | 受信した `.mdv` の履歴を読み取る「Versions」ビュー（base picker と統合） | ★★★ |
| M2 | 同 `repo_id` の 2 つの `.mdv` をマージ（コミット union） | ★★ |
| M3 | 厳密 JCS (RFC 8785) 準拠の JSON 正規化 | ★ |
| M4 | checkpoint / sidecar 履歴の永続化（連続編集の自動 commit） | ★★ |
| M5 | `.mdv` 内ハンドルの user 設定（現在 author name = "ebi-oishii" ハードコード）| ★★ |
| M6 | TUI から pack / extract | ★ |
| M7 | 受信時の bundle 検証エラー UI（壊れた package block の取り扱い） | ★ |

## 配布・運用

| # | 内容 | 優先 |
|---|---|---|
| D1 | パッケージ署名と配布（GUI: dmg / msi / AppImage、TUI: cargo install / Homebrew） | ★（ユーザー判断待ち）|
| D2 | CI: cargo test / npm run check に加えて release build matrix | ★★ |
| D3 | 国際化（i18n）の足場 | ★ |

## ストレッチ / 長期

| # | 内容 |
|---|---|
| S1 | リアルタイム共同編集（CRDT 系） |
| S2 | AI 拡張：要約、変更点解説、文体提案 |
| S3 | プラグインシステム |
| S4 | 開いてる文書全体での全文検索 |
| S5 | 大容量ファイル：仮想化 + 段階レンダリングで制限緩和 |
| S6 | Side-by-Side：文字単位 inline diff（difftastic 風） |

---

## 完了済み

直近の主要マイルストーン（細かい修正は [git log](https://github.com/ebi-oishii/mdv/commits) 参照）：

- Phase 0〜0.5: scaffold、Cargo ワークスペース化
- Phase 1: Source / Preview（GUI + TUI）
- Phase 2: Diff Highlight Only / Full（GUI + TUI）+ base picker
- Phase 2.5: Diff Side-by-Side（GUI + TUI 簡易版）
- Phase 3A: Live Preview（GUI）
- Phase 3B: WYSIWYG (Milkdown)（GUI）
- Phase 4: モバイル土台（Android scaffold、レスポンシブ CSS）
- Phase 5: 大容量ガード、TUI Diff キャッシュ、TUI コマンドモード（:w :q :wq :q!）
- Phase 6: HTML / PDF / Plain text / DOCX エクスポート
- Phase 7: `.mdv` パッケージ入出力（pack / extract）
- Phase 8: Settings + テーマ切替（light / dark / auto）+ font size + default mode
