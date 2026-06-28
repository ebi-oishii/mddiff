# 技術選定 — 根拠とトレードオフ

各選定について「採用案 / 比較対象 / 決め手」を簡潔に記録する。

## ADR-001: アプリシェルに Tauri 2 を採用

- **採用**: Tauri 2.x（Rust + システム WebView）
- **比較**: Electron / Flutter / 各 OS ネイティブ
- **決め手**:
  - 「軽量」要件にもっとも合致（バイナリ・メモリとも Electron の数分の一）
  - Tauri 2 から iOS/Android 公式対応 → モバイル要件をクリア
  - Rust 側で `git2`, `similar` など堅牢な crate を活かせる
- **代償**:
  - WebView 差異（特に Linux WebKitGTK、iOS WKWebView）の検証コスト
  - Electron に比べ事例・解決済 SO 質問が少ない
- **却下理由**:
  - Electron: バイナリ 80MB+ がモバイル含めた "軽量" 方針と矛盾
  - Flutter: Markdown 周辺ライブラリの厚みが弱い、特に WYSIWYG 系が事実上ない
  - ネイティブ複数言語: 工数が許容外

## ADR-002: フロントは Svelte 5

- **採用**: Svelte 5（runes ベース、SvelteKit static または素の Svelte SPA）
- **比較**: React / Solid / Vue
- **決め手**:
  - ランタイムが極小、Tauri との組み合わせでロード時間を最小化
  - Runes による細粒度リアクティブは DocStore + 多 View の同期に素直
- **代償**:
  - Milkdown / TipTap は React 事例が多い。Svelte バインディングは framework-agnostic ラッパ経由になる
- **却下理由**:
  - React: ランタイムが重い、軽量要件と矛盾
  - Solid: API 自体は良いが MD エディタ統合事例が薄い

## ADR-003: ソースエディタに CodeMirror 6

- **採用**: CodeMirror 6（`@codemirror/lang-markdown` + 必要拡張のみ）
- **比較**: Monaco / textarea + 自前ハイライト
- **決め手**:
  - モジュラー設計でバンドルを必要分に絞れる
  - 拡張で diff デコレーション（Highlight Only モード）を素直に実装できる
  - モバイルでの動作実績あり
- **代償**:
  - 設定が宣言的で学習コストはある
- **却下理由**:
  - Monaco: 大きすぎる（数 MB）、デスクトップ前提のスタイル
  - 自前: スクロール、検索、折りたたみを再発明することになる

## ADR-004: Live Preview と WYSIWYG は別モードとして採用

- **採用**:
  - Live Preview: CodeMirror 6 decorations で実装
  - WYSIWYG: Milkdown（ProseMirror ベース）を候補にする
- **比較**: Live Preview だけ / WYSIWYG だけ / TipTap / Lexical
- **決め手**:
  - Live Preview は Source の正確性を保ちつつ、編集中も rendered result に近い見え方を得られる
  - WYSIWYG は Markdown 記法を意識せずに軽く直したい場面で有効
  - 両方を分けることで、source を保ちたい時と rich editing したい時をユーザーが選べる
  - CodeMirror 6 の既存状態を活かしつつ、必要な rich editing だけ Milkdown に任せられる
- **代償**:
  - GUI mode が増えるため、ModeBar とショートカット設計が複雑になる
  - WYSIWYG の markdown round-trip で表記正規化が起きうる
  - Live Preview と WYSIWYG の状態同期を検証する必要がある
- **却下理由**:
  - Live Preview だけ: Markdown 記法を意識せず直したい用途を満たせない
  - WYSIWYG だけ: source の制御感と Markdown 表記保持が弱くなる
  - TipTap: WYSIWYG としては優秀だが Markdown source を一級に保つには追加実装が重い
  - Lexical: 高性能だが Markdown round-trip のエコシステムが薄い

## ADR-005: Markdown パーサは markdown-it（表示）+ pulldown-cmark（必要時）

- **採用**:
  - フロントの Preview レンダリング: `markdown-it` + `markdown-it-gfm`
  - Rust 側で AST 解析が必要になった場合: `pulldown-cmark`
- **比較**: remark / micromark
- **決め手**:
  - markdown-it はプラグイン構造が枯れていて GFM・脚注など追加しやすい
  - Rust 側のパーサは差分のセクション単位スマート diff（将来）で役立つ
- **代償**: なし（フロントとバックで実装言語が違うため最低限の整合性検証は必要）

## ADR-006: Git 操作は git2 crate を Rust 側で

- **採用**: `git2`（libgit2 bindings）
- **比較**: `git` CLI を shell out / `gitoxide`
- **決め手**:
  - 外部 `git` 不在の環境でも動く
  - APIが安定、`diff_tree_to_workdir` 等で必要な情報が直接取れる
- **却下理由**:
  - CLI shell out: モバイルで使えない
  - `gitoxide`: まだ pure Rust 実装が完了しておらず、書き込み系が弱い

## ADR-007: 差分計算は similar crate

- **採用**: `similar`（Patience / Myers）
- **比較**: `diff` / 自前 LCS
- **決め手**:
  - GitHub と同等の Patience diff を選べる
  - ハンク単位の API が整っている
- **備考**: Highlight Only モードは「変更があった行範囲」のみ送れば十分なので、ハンクから `[startLine, endLine]` を抽出してフロントへ。

## ADR-008: TUI ライブラリは ratatui + crossterm

- **採用**:
  - メインフレーム: `ratatui` 0.29+
  - 端末バックエンド: `crossterm` 0.28+
  - 引数解析: `clap` v4 (derive)
- **比較**: cursive / Tauri TUI 拡張 / 自作
- **決め手**:
  - ratatui は Rust TUI のデファクト。広く採用され、ウィジェット・レイアウト DSL が成熟
  - immediate-mode 描画で AppState を 1 つ持てば良く、`mddiff-core` の DocState と素直に結合できる
  - crossterm は Windows 含めて移植性◎、Tauri デスクトップが対応する全 OS で同じ挙動
- **代償**:
  - 編集ウィジェットは ratatui 純正には存在しない → `edtui` などサードパーティに依存、または自作
  - 日本語幅計算（東アジア全角・絵文字）は Rust エコシステム全体で罠が多く、テストでカバー必須
- **却下理由**:
  - cursive: 高レベルだが ratatui ほど活発でない。今回はカスタム描画（Highlight Only の色帯など）を多用するため低レベル制御が欲しい

## ADR-009: Rust コードは Cargo ワークスペース化し 3 crate に分割

- **採用**: ワークスペース構成
  - `src-tauri/` (member, package name = `mddiff`) — Tauri GUI 用バイナリ
  - `crates/mddiff-tui/` (package name = `mddiff-tui`) — TUI バイナリ
  - `crates/mddiff-core/` (package name = `mddiff-core`) — UI 非依存の純粋ロジック
- **比較**: GUI と TUI を単一バイナリ + `--tui` フラグ / 別リポジトリ / GUI 内部に TUI モジュールを直書き
- **決め手**:
  - GUI と TUI は **UI レイヤーが完全に別** で共有意義が薄い一方、**MD パース・差分・Git** は完全に共通 → core 分離が自然
  - 別バイナリにすることで TUI を `cargo install mddiff-tui` や Homebrew で単独配布できる（サーバ用途）
  - Tauri 既存の `src-tauri/` ディレクトリ名はツーリングの慣習なのでそのまま残し、ワークスペースの 1 メンバとして扱う
- **代償**:
  - ワークスペースルートに別途 `Cargo.toml` が必要
  - `mddiff-core` の API 変更時に両 binary を更新する必要
- **却下理由**:
  - 単一バイナリ + フラグ: Tauri ランタイムを含むため TUI 用途でも数 MB のサイズ増。`tauri::Builder` を起動しない分岐パスの保守も面倒
  - 別リポジトリ: 共有コードを crate として公開する手間が割に合わない（個人プロジェクト規模）

## ADR-010: TUI の起動はデフォルトでフル機能、`--read-only` で読み取り専用

- **採用**: `mddiff-tui file.md` は **編集モード** で起動。読み取り専用は `--read-only` で明示
- **比較**: 逆（デフォルト読み取り、`--edit` で編集）
- **決め手**:
  - GUI と TUI で挙動を揃える方が原則。GUI は常に編集可能
  - "less 代替" としての需要には `--read-only` で応えられる
- **代償**: 誤って書き換える事故の可能性は残る → 保存時は明示的に `Ctrl+S` か `:w` が必要なため実害は小さい

## ADR-011: Diff の Side-by-Side ハイライトは markdown-it token.map を使う

- **採用**: markdown-it の組み込み機能 `token.map = [start_line, end_line]` を
  使い、`parse → ハンクと重なるブロックに class 注入 → render` の 2 段パイプラインで
  実装する
- **比較**:
  - markdown-it-source-map プラグイン（全要素に `data-source-line` を出す）
  - 両側 HTML を DOM diff して `<mark>` 挿入（文字単位）
- **決め手**:
  - 外部依存ゼロ、markdown-it 既存
  - ブロック単位のハイライトで HTML 構造を一切変えない
  - 段落・見出し・リスト・コードブロック・テーブルにそのまま効く
- **代償**:
  - 段落内 1 字違いでも段落全体が塗られる（精度はブロック単位まで）
- **却下理由**:
  - source-map プラグイン: token.map で十分。依存追加の見返りなし
  - DOM diff 方式: HTML 構造を壊しやすく、テーブルや list 内で破綻する

## ADR-012: HunkSummary は NEW と OLD 両側の行範囲を保持する

- **採用**: `HunkSummary { kind, new_start, new_end, old_start, old_end }`
- **背景**: 既存は `start_line, end_line, removed_count` の NEW 中心 API。
  Side-by-Side の OLD 側ハイライトに必要な OLD 行情報が取れない
- **比較**: NEW 用と OLD 用で別関数を用意 / HunkSummary をネスト型に
- **決め手**:
  - 1 つの型で両方の情報を持つほうが呼び出し側がシンプル
  - 空範囲は `start == end == 0` で表現（Added/Removed に対応）
- **代償**: 既存の Highlight Only / Full の実装に小幅な改修が入る（命名と
  派生計算の追加）
- **却下理由**:
  - 別関数: 同じ DiffOp を 2 回走査する無駄、整合性のバグ温床

## ADR-013: TUI の Side-by-Side は MD レンダリングではなく Source テキストの並置

- **採用**: TUI Side-by-Side は左右に Source テキストを並べ、ハンクで色付け
- **理由**:
  - 80 桁端末で MD の 2 ペインレンダリングは可読性破綻
  - 「左右で見比べる」目的だけなら Source 並置でも十分達成できる
- **挙動**: 横幅 < 100 桁の端末では Side-by-Side サブモード自体を無効化
  （Tab で巡回時にスキップ）
- **代償**: GUI と TUI で同じ Diff サブモード名の体験が異なる → ヘルプで
  明示する

## ADR-014: スタイルは Tailwind ではなく素の CSS + CSS 変数

- **採用**: 素の CSS（コンポーネントスコープ） + CSS カスタムプロパティでテーマ
- **比較**: Tailwind / UnoCSS
- **決め手**:
  - Svelte の scoped style と相性が良くバンドル増を避けられる
  - テーマ切替（ライト/ダーク/エディタテーマ）は CSS 変数で完結する規模
- **却下理由**: Tailwind はクラス爆発と PurgeCSS の設定が、本規模では割に合わない
