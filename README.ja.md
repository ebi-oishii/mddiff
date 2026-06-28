# mddiff

軽量・クロスプラットフォームの Markdown ビューア兼エディタ。
GUI（Desktop / Mobile）と TUI を 1 つの core 実装で動かす。

## 何ができる

- **5 つのモード**: Source / Live Preview / WYSIWYG / Preview / Diff を切替
  - Source: CodeMirror 6
  - Live Preview: 現在行は raw、他は inline 装飾（Typora 風）
  - WYSIWYG: Milkdown ベースの rich editing
  - Preview: 読み取り専用レンダリング
  - Diff: Git 管理下なら任意 revision との差分
- **Diff モードに 3 サブモード**:
  - Highlight Only — 編集中の文脈を崩さず変更箇所だけ色帯で表示
  - Full — GitHub 風の +/− 統合表示
  - Side-by-Side — 旧/新を左右で並べてブロック単位ハイライト
- **比較対象を選べる**: HEAD / HEAD~N / ブランチ / タグ / 直近コミットから選択、
  変更がある base には ● マーカー（同 blob 連続区間は最古だけに集約）
- **TUI も同等機能**: Source / Preview / Diff（Highlight / Full / Side-by-Side 簡易版）+
  ratatui の popup で base ピッカー + vim 風 `:w` `:q` `:wq` `:q!`
- **Export to HTML / PDF / Plain text / DOCX**: GUI の Export ▾ ボタンから。
  PDF は OS のプリントダイアログ経由（追加依存なし）
- **`.mddiff` パッケージ入出力**: Git 配下のファイルを履歴ごと 1 ファイルに
  bundle（zstd+base64 を HTML コメント内）。受信側は Git なしで本文を
  読める。仕様は [docs/mddiff-protocol.md](docs/mddiff-protocol.md)
- **配布形態**:
  - GUI = Tauri 2（Mac / Windows / Linux / iOS / Android）
  - TUI = 単独バイナリ（リリースビルドで 2.3MB）

## アーキテクチャ

Cargo ワークスペースで 3 crate に分割：

```
mddiff/
├── crates/mddiff-core/   # UI 非依存の純粋ロジック（diff, git, fs, doc state）
├── crates/mddiff-tui/    # ratatui ベースの端末 UI
└── src-tauri/         # Tauri 2 シェル + Svelte 5 + CodeMirror / Milkdown
```

## 開発

### GUI（Tauri）
```sh
npm install
npm run tauri dev                   # Desktop
npm run tauri android dev           # Android（要 Android Studio + NDK + JAVA_HOME）
npm run tauri ios dev               # iOS（要 Xcode + cocoapods + iOS targets）
```

初回モバイル用には `npm run tauri ios init` / `npm run tauri android init`。
iOS は `brew install cocoapods` も必要。

### TUI
```sh
cargo run -p mddiff-tui                       # ファイル指定なし
cargo run -p mddiff-tui -- README.md          # ファイル指定
cargo run -p mddiff-tui -- --diff-base HEAD~3 README.md  # 任意 revision と比較
cargo run -p mddiff-tui -- --read-only README.md         # 読み取り専用
```

### 品質チェック

```sh
npm run check              # Svelte + TypeScript
cargo check --workspace    # Rust 全 crate
cargo test  --workspace    # Rust テスト
```

## ライセンス

MIT — [LICENSE](LICENSE) 参照。

## ドキュメント

- [docs/design.md](docs/design.md) — アーキテクチャと各モードの設計
- [docs/decisions.md](docs/decisions.md) — 技術選定 (14 ADRs)
- [docs/roadmap.md](docs/roadmap.md) — フェーズと進捗
- [docs/tooling-research.md](docs/tooling-research.md) — 既存ツールから取り入れる使用感
- [docs/mddiff-protocol.md](docs/mddiff-protocol.md) — `.mddiff` portable package format v1
- [docs/issues.md](docs/issues.md) — 取り組み候補リスト（バグ・ポリッシュ・機能候補）
