# mddiff

> **Alpha · v0.2.1-alpha** — v1 までに API / ファイル形式 / 設定が変わる可能性があります。フィードバックは issue で歓迎します。

**Git diff と「過去のバージョンに戻る」がエディタの中にある Markdown エディタ。**
Typora みたいな書き心地に、`git diff` と `git checkout` を本文の隣に置いた感じ。

🇺🇸 English: [README.md](README.md)

![mddiff main view](docs/img/hero.png)

## こんな人向け

README / 設計ドキュメント / ノート / ADR を **Git で管理しながら書いてる人** が対象。書きながらいちばんよくする「ここ何変えたっけ」「あの頃どう書いてたっけ」が、カーソルからワンクリックで確認できます。

## 何がいいの

### Diff が plugin ではなく一級モード

![Side-by-side diff](docs/img/diff-sbs.png)

- **任意の Git リビジョン** を比較対象に選べる（branch / tag / commit / 自由 revspec）。ディスクとの diff も可
- 3 サブモード：**Highlight Only / Full / Side-by-Side（サブブロック単位のスクロール同期付き）**
- commit picker は「このファイルを変えたコミット」だけに絞る（全表示への切替もあり）
- 外部でファイルが書き換わると banner が出て、その場で Diff モードに飛べる

### 過去の版に時間移動

![履歴ビュー](docs/img/history-view.png)

- picker の「**この版を見る**」で、選んだコミット時点の本文を読み取り専用 Preview で表示
- `←` / `→` で他のコミットへ walk
- **「この版を復元」** で履歴の中身を現在の buffer に取り込み（編集として入るので `⌘Z` で巻き戻せる、保存するまでディスクのファイルは変わらない）

履歴中に **Diff モード** に切り替えると、固定中の版を current 側にして他の任意の版と比較できる。「v1 と v2 の間で、この章は変わったっけ？」に即答できる。

![履歴 + Diff](docs/img/history-diff.png)

### 3 グループのビュー、キーボードで完結

トップは 3 つ、それぞれ raw / read-only のサブモードを持つ：

| グループ | メイン | サブ | ショートカット |
|---|---|---|---|
| Live Preview | Live Preview (Typora 風) | Source（生 markdown） | `⌘1` / `⌘⇧1` |
| WYSIWYG | WYSIWYG (Milkdown) | Preview (HTML 読取専用) | `⌘2` / `⌘⇧2` |
| Diff | Diff | — | `⌘3` |

それから **アウトラインサイドバー** (`⌘⇧O`)、**スプリットペイン** (`⌘\`) で「片方は WYSIWYG、もう片方は Diff」みたいな並列も可能。

![アウトラインサイドバー](docs/img/outline.png)

### GUI と TUI で機能対称

同じ `mddiff-core` crate を共有。TUI は **~2.3 MB の単一バイナリ**、vim 風 `:w :q :wq`、Diff サブモードも同じ。

### `.mddiff` ポータブルバンドル

Git 履歴を zstd 圧縮で HTML コメントに埋め込み、1 ファイルで配布可能。受信側は Git なしで本文を素の Markdown として読め、mddiff なら履歴も再生できる。

### 軽量

Tauri 2 ベース。Chromium / Node は同梱しない。**macOS で idle RAM ≈ 80 MB、cold start ≈ 0.5 秒。** Electron 系競合の 1.5〜3 倍軽量。

## インストール

### 配布バイナリ

[Releases ページ](https://github.com/ebi-oishii/mddiff/releases) から：

| プラットフォーム | ファイル |
|---|---|
| macOS (Apple Silicon) | `mddiff_0.2.1_aarch64.dmg` |
| macOS (Intel) | `mddiff_0.2.1_x64.dmg` |
| Windows | `mddiff_0.2.1_x64-setup.msi` |
| Linux | `mddiff_0.2.1_amd64.AppImage` |
| TUI (任意 OS) | `mddiff-tui_0.2.1_<target>` |

### ソースから

必要：Rust 1.75+、Node 22+、[Tauri prerequisites](https://tauri.app/start/prerequisites/)

```sh
git clone https://github.com/ebi-oishii/mddiff
cd mddiff
npm install
npm run tauri build              # → target/release/bundle/
cargo build --release -p mddiff-tui   # → target/release/mddiff-tui
```

## ショートカット早見表

| ショートカット | 動作 |
|---|---|
| `⌘1` / `⌘⇧1` | Live Preview / Source |
| `⌘2` / `⌘⇧2` | WYSIWYG / Preview |
| `⌘3` | Diff |
| `⌘\` | スプリットペイン |
| `⌘⇧O` | アウトラインサイドバー |
| `⌘O` / `⌘S` / `⌘⇧S` | 開く / 保存 / 別名保存 |
| `⌘⇧R` | ディスクから再読込 |
| `⌘,` | 設定 |
| `Esc`（履歴中） | 履歴ビュー終了 |

Linux / Windows では `⌘` の代わりに `Ctrl`。

## ステータス

**Alpha**：日常使用に耐えるが、v1 までに format / 設定が変わる可能性あり。

既知の制限：
- `.mddiff` フォーマットは v1 までに非互換変更されうる
- Settings localStorage スキーマはバージョン間でリセットされうる
- iOS / Android はまだ scaffold 状態
- Diff base picker は英語ラベルのみ

ロードマップ候補（詳細は [docs/issues.md](docs/issues.md)）：
- Mermaid 図、KaTeX 数式、スペルチェック
- Find & Replace の regex / case-sensitivity
- git blame ガター
- ~~画像クリップボード貼付~~、~~アウトラインサイドバー~~、~~リンククリック移動~~、~~履歴ビュー~~ — **実装済み**

## アーキテクチャ

Cargo ワークスペース、3 crate 構成：

```
mddiff/
├── crates/mddiff-core/   # UI 非依存 (diff / git / fs / pack)
├── crates/mddiff-tui/    # ratatui ベース端末 UI
└── src-tauri/            # Tauri 2 シェル + Svelte 5 + CodeMirror / Milkdown
```

詳細：[docs/design.md](docs/design.md) · ADRs：[docs/decisions.md](docs/decisions.md) · ポジショニング：[docs/competitive-analysis.md](docs/competitive-analysis.md)

## 開発

```sh
npm install
npm run tauri dev                  # GUI デスクトップ開発モード
npm run tauri android dev          # Android (Android Studio + NDK + JAVA_HOME)
npm run tauri ios dev              # iOS (Xcode + cocoapods)

cargo run -p mddiff-tui -- README.md
cargo run -p mddiff-tui -- --diff-base HEAD~3 README.md

npm run check              # Svelte + TypeScript
cargo check --workspace    # Rust
cargo test  --workspace    # Rust テスト
```

## ライセンス

MIT — [LICENSE](LICENSE) 参照。

## ドキュメント

- [docs/design.md](docs/design.md) — アーキテクチャと各モードの設計
- [docs/decisions.md](docs/decisions.md) — 技術選定 ADRs
- [docs/roadmap.md](docs/roadmap.md) — フェーズと進捗
- [docs/competitive-analysis.md](docs/competitive-analysis.md) — 競合分析とポジショニング
- [docs/mddiff-protocol.md](docs/mddiff-protocol.md) — `.mddiff` パック形式
- [docs/issues.md](docs/issues.md) — 取り組み候補
