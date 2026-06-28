# mddiff

> **Alpha · v0.1.0-alpha** — v1 までに API / ファイル形式 / 設定が変わる可能性があります。フィードバックは issue で歓迎します。

軽量・クロスプラットフォームの Markdown エディタ。**diff を一級機能として持つ**のが特徴。Rust の単一 core を、Tauri 2 + Svelte 5 の GUI と ratatui の TUI バイナリで共有します。

🇺🇸 English: [README.md](README.md)

## 何ができる

- **5 つの編集モード**:
  - **Source** — CodeMirror 6 ベース。markdown シンタックスハイライト、行番号、ソフトラップ、タブ幅設定
  - **Live Preview** — Typora 風。編集行は raw、それ以外はインライン装飾
  - **WYSIWYG** — Milkdown / ProseMirror。タスクリストはクリックで toggle
  - **Preview** — 読み取り専用の HTML レンダリング
  - **Diff** — 任意の Git revision、またはディスク版との差分
- **Diff は plugin ではなく一級機能**:
  - 3 サブモード (Highlight Only / Full / Side-by-Side、スクロール同期付き)
  - base picker はファイルを実際に変更したコミットだけにフィルタ
  - 外部変更検知 banner からその場で Diff モードに飛べる
- **GUI + TUI で機能対称** — どちらも同じ `mddiff-core` crate。TUI は ~2.3 MB の単一バイナリ、vim 風 `:w :q :wq` + 同じ Diff サブモード対応
- **`.mddiff` ポータブルバンドル** — Git 履歴を zstd 圧縮で HTML コメントに埋め込み、1 ファイルで配布可能。受信側は Git なしで本文を素の Markdown として読め、mddiff なら履歴も再生可能
- **軽量** — Tauri 2 ベースで Chromium / Node は同梱しない。macOS で idle RAM ≈ 80 MB、cold start ≈ 0.5 秒。Electron 系競合の 1.5〜3 倍軽量

## インストール

### 配布バイナリ（推奨）

[Releases ページ](https://github.com/ebi-oishii/mddiff/releases) からダウンロード:

| プラットフォーム | ファイル |
|---|---|
| macOS (Apple Silicon) | `mddiff_0.1.0_aarch64.dmg` |
| macOS (Intel) | `mddiff_0.1.0_x64.dmg` |
| Windows | `mddiff_0.1.0_x64-setup.msi` |
| Linux | `mddiff_0.1.0_amd64.AppImage` |
| TUI (任意の OS) | `mddiff-tui_0.1.0_<target>` |

### ソースから build

必要: Rust 1.75+、Node 22+、Tauri 用 platform 依存 ([Tauri prerequisites](https://tauri.app/start/prerequisites/))

```sh
git clone https://github.com/ebi-oishii/mddiff
cd mddiff
npm install
npm run tauri build              # dmg / msi / AppImage が target/release/bundle/ に生成
cargo build --release -p mddiff-tui   # TUI 単一バイナリ
```

## ステータスと roadmap

**Alpha**: 日常使用に耐える程度には完成していますが、v1 までに format や設定が変わる可能性があります。

**既知の制限**:
- `.mddiff` フォーマットは v1 までに非互換に変更されうる
- Settings localStorage スキーマはバージョン間でリセットされうる
- iOS / Android はまだ scaffold 状態（バグあり前提）
- Diff base picker は英語ラベルのみ（他の UI は日英対応済み）

**近期ロードマップ**（詳細は [docs/issues.md](docs/issues.md)）:
- 画像クリップボード貼付 / D&D → `.assets/` 自動保存
- TOC / アウトラインサイドバー
- Mermaid 図のレンダリング
- KaTeX 数式
- スペルチェック
- Find & Replace の regex / case-sensitivity トグル
- git blame ガター

## アーキテクチャ

Cargo ワークスペース、3 crate 構成:

```
mddiff/
├── crates/mddiff-core/   # UI 非依存のロジック (diff / git / fs / pack)
├── crates/mddiff-tui/    # ratatui ベース端末 UI
└── src-tauri/            # Tauri 2 シェル + Svelte 5 + CodeMirror / Milkdown
```

詳細は [docs/design.md](docs/design.md)、技術選定の理由は [docs/decisions.md](docs/decisions.md)、ポジショニングの分析は [docs/competitive-analysis.md](docs/competitive-analysis.md) を参照。

## 開発

### GUI (Tauri)

```sh
npm install
npm run tauri dev                  # Desktop 開発モード
npm run tauri android dev          # Android (Android Studio + NDK + JAVA_HOME)
npm run tauri ios dev              # iOS (Xcode + cocoapods)
```

モバイル初回: `npm run tauri ios init` / `android init`。iOS はさらに `brew install cocoapods` が必要。

### TUI

```sh
cargo run -p mddiff-tui                              # ファイル指定なし
cargo run -p mddiff-tui -- README.md                 # ファイル指定
cargo run -p mddiff-tui -- --diff-base HEAD~3 README.md  # 任意 revision との差分
cargo run -p mddiff-tui -- --read-only README.md          # 読み取り専用
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
- [docs/decisions.md](docs/decisions.md) — 技術選定 ADRs
- [docs/roadmap.md](docs/roadmap.md) — フェーズと進捗
- [docs/competitive-analysis.md](docs/competitive-analysis.md) — 競合分析とポジショニング
- [docs/refactor-audit.md](docs/refactor-audit.md) — コード品質 audit
- [docs/mddiff-protocol.md](docs/mddiff-protocol.md) — `.mddiff` portable package format v1
- [docs/issues.md](docs/issues.md) — 取り組み候補リスト
