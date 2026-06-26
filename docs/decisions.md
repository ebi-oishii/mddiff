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

## ADR-004: WYSIWYG は Milkdown（Phase 2）

- **採用**: Milkdown（ProseMirror ベース、MD 双方向特化）
- **比較**: TipTap + markdown 拡張 / Lexical / 自前
- **決め手**:
  - 設計思想が "MD を一級市民として扱う" → Source ↔ WYSIWYG の往復に最適
  - プラグイン分割が細かく、必要機能だけロード可能
- **代償**:
  - ProseMirror 由来の API 複雑性
  - シリアライズで表記揺れが正規化される（`*` / `_` など）→ UI で明示する必要あり
- **却下理由**:
  - TipTap: WYSIWYG としては優秀だが MD への往復は別途実装する必要があり、結局 Milkdown と同等の手間
  - Lexical: 高性能だが MD 双方向のエコシステムが薄い

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

## ADR-008: スタイルは Tailwind ではなく素の CSS + CSS 変数

- **採用**: 素の CSS（コンポーネントスコープ） + CSS カスタムプロパティでテーマ
- **比較**: Tailwind / UnoCSS
- **決め手**:
  - Svelte の scoped style と相性が良くバンドル増を避けられる
  - テーマ切替（ライト/ダーク/エディタテーマ）は CSS 変数で完結する規模
- **却下理由**: Tailwind はクラス爆発と PurgeCSS の設定が、本規模では割に合わない
