# mddiff Quick Look extension (macOS)

macOS 用の Quick Look Preview Extension。Finder で `.md` を選んで Space
（または `⌘Y`）を押すと、mddiff の Preview と同じ見た目のプレビューを
表示する。

- 対応 UTI: `net.daringfireball.markdown`, `public.markdown`
- macOS 11.0+
- レンダリングは `markdown-it` を bundled JS として同梱し WKWebView 内で
  実行する — アプリ本体の Preview と同じパイプラインを使うことで
  「アプリの表示」と「Quick Look の表示」を一致させる

## ディレクトリ

```
macos-extensions/QuickLook/
├── build.sh           # .appex を組み立てる (swiftc 直呼び)
├── install.sh         # 指定した mddiff.app に .appex を注入
├── src/
│   ├── PreviewProvider.swift        # QLPreviewingController 実装
│   ├── Info.plist                    # NSExtension 宣言 + UTI
│   └── MddiffQuickLook.entitlements  # sandbox + read-only file access
└── resources/
    ├── preview.html    # WKWebView に流し込む HTML テンプレート
    └── preview.css     # アプリ Preview と同系統の GitHub 風スタイル
```

`markdown-it.min.js` はリポジトリにコミットせず、build 時に
`node_modules/markdown-it/dist/markdown-it.min.js` からコピーする。
`npm install` を先に済ませておくこと。

## ローカルで試す

```bash
# 1. 拡張を組み立てる
./macos-extensions/QuickLook/build.sh
# → macos-extensions/QuickLook/build/MddiffQuickLook.appex

# 2. インストール済みの mddiff.app に注入
./macos-extensions/QuickLook/install.sh /Applications/mddiff.app

# 3. Finder で .md ファイルを選び Space
```

`install.sh` は `lsregister -f` で LaunchServices の再スキャンを促し、
Quick Look サーバを `pkill` するので、ログアウト無しで反映される。

登録状況の確認:

```bash
pluginkit -m -p com.apple.quicklook.preview | grep -i mddiff
```

## 環境変数

`build.sh` で使える上書き変数:

| 変数 | 既定 | 用途 |
|---|---|---|
| `ARCH` | `arm64` | `arm64` / `x86_64` / `universal` |
| `MIN_MACOS` | `11.0` | 最低 macOS バージョン |
| `IDENTITY` | `-` | codesign identity。配布時は `Developer ID Application: ...` |

## Tauri ビルドへの取り込み (未実装)

現時点では手動注入のみ。将来的には Tauri の `afterBundleCommand` から
`build.sh + install.sh` を呼び出し、`mddiff.app/Contents/PlugIns/` に
自動同梱する予定。
