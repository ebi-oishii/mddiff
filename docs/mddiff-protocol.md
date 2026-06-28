# .mddiff プロトコル設計

`.mddiff` は、Markdown 文書を履歴ごと持ち運ぶための portable package である。
最新版の Markdown 本文と、過去 snapshot / checkpoint / metadata を 1 ファイルに同梱する。

目的は、`.git` ディレクトリや外部サービスがない状態でも、受け取った 1 ファイルだけで
過去版との差分を確認できるようにすること。

`.mddiff` は日常的な authoring format ではない。通常の編集作業では `.md` を主ファイルとし、
共有・提出・アーカイブ時に `.mddiff` へ pack する運用を主導線とする。

## 1. 設計原則

- **Markdown first**: 最新版の本文は先頭から平文 Markdown として読める。
- **Portable package**: 履歴・メタデータ・過去スナップショットは `.mddiff` ファイル内に保持する。
- **Authoring は `.md` 優先**: 普段は `.md` と sidecar 履歴で作業し、配布時に `.mddiff` へ pack する。
- **Git 風だが Git 依存ではない**: commit DAG、parent、content hash、author、message を持つが、
  `.git` や git コマンドには依存しない。
- **本文を取り出せる**: `cat` / `flatten` により、履歴なしの通常 Markdown をいつでも得られる。
- **受信側で検証可能**: 本文 hash、commit hash、bundle checksum を検証し、履歴と現在本文の
  整合性を判定できる。
- **壊れても本文優先**: package block が壊れても Markdown 本文は取り出せる。
- **AI は副次用途**: raw `.mddiff` を AI に直接読ませることは主目的にしない。AI には本文、
  revision summary、選択 diff を抽出して渡す。
- **既存 Git diff と独立**: 現在の `mddiff-core::git` / Tauri git command とは別の履歴 provider
  として実装し、Claude 側の Git 拡張と干渉させない。

## 2. ファイル構造

`.mddiff` は UTF-8 テキストファイルで、最新版本文と 1 つの MDV package block から成る。

識別子:

- 拡張子: `.mddiff`
- 推奨 MIME: `text/markdown; charset=utf-8; profile="https://mddiff.dev/profiles/mddiff-v1"`
- 代替 MIME: `application/vnd.mddiff+markdown; charset=utf-8`
- magic marker: `<!-- mddiff:v1` で始まる HTML コメント

拡張子だけで判定せず、package block がない場合は通常の Markdown として扱う。
`.mddiff` は Markdown として表示可能だが、巨大な履歴を含みうるため、通常の Markdown ファイルと
完全に同じ authoring 体験を保証することは目標にしない。

```md
# Title

Current Markdown body.

<!-- mddiff:v1
codec: zstd+base64
bundle-sha256: sha256:0123456789abcdef...
bundle-size: 12345
payload:
KLUv/QBY...
-->
```

### 2.1 本文

本文は、MDV package block を取り除いた残りのバイト列である。

通常の Markdown エディタやビューアは HTML コメントを無視するため、`.mddiff` は本文部分を
Markdown として読める。履歴に対応しないツールで保存した結果、コメントが削除された場合は
履歴は失われるが、本文は維持される。

### 2.2 Package Block

Package block は HTML コメントとして格納する。

- 開始行は `<!-- mddiff:v1`
- 終了行は `-->`
- `payload:` 以降、終了行までが base64 ペイロード
- ペイロードは ASCII 空白と改行を無視して連結する
- 標準 base64 を使う。文字集合に `-` がないため、HTML コメント内の `--` を生成しない

Writers は package block をファイル末尾に置く。Readers は最後に見つかった well-formed な
MDV package block を採用し、そのブロックを除いた内容を現在本文として扱う。

## 3. Envelope Header

Package block 内のヘッダは以下を持つ。

| Field | Required | Meaning |
|---|---:|---|
| `codec` | yes | `identity+base64` または `zstd+base64` |
| `bundle-sha256` | yes | codec 復号後の canonical bundle bytes の SHA-256 |
| `bundle-size` | yes | codec 復号後の canonical bundle bytes length |
| `payload` | yes | codec 適用済み bundle bytes の base64 |

Readers は最低限 `identity+base64` と `zstd+base64` を読む。Writers は通常
`zstd+base64` を使う。

`mddiff:v1` 以外の marker を見つけた reader は、その package block を未知バージョンとして扱う。
本文は開けるが、履歴の書き換えはしない。v1 reader は未知の JSON field を無視してよいが、
checkpoint で bundle を書き直す場合は未知 field を保持する。

## 4. Bundle

ペイロードを復号すると、JSON Canonicalization Scheme (RFC 8785) に従った JSON bundle が
得られる。

```json
{
  "format": "mddiff-bundle",
  "version": 1,
  "repo_id": "urn:uuid:6f0f5d72-8c6e-4a3a-bbc0-1c4a9a8db2b4",
  "head": "c:9e1f...",
  "refs": {
    "main": "c:9e1f..."
  },
  "commits": {
    "c:9e1f...": {
      "type": "commit",
      "parents": ["c:1b2c..."],
      "body": "b:3a5d...",
      "author": {
        "name": "Taiichi",
        "email": null
      },
      "created_at": "2026-06-27T10:20:30Z",
      "message": "Update introduction",
      "source": {
        "app": "mddiff",
        "version": "0.1.0"
      },
      "signatures": []
    }
  },
  "snapshots": {
    "b:3a5d...": {
      "type": "snapshot",
      "size": 42,
      "text": "# Title\n\nCurrent Markdown body.\n"
    }
  }
}
```

### 4.1 Object IDs

Object ID は canonical body bytes から計算する。v1 の canonical body bytes は UTF-8 text の
line ending を LF に正規化した bytes である。末尾改行の有無は本文の意味を変えうるため、
自動では追加・削除しない。

本文 snapshot ID:

```text
b:<hex sha256(canonical body bytes)>
```

Commit ID:

```text
c:<hex sha256("mddiff.commit.v1\0" + JCS(commit_without_signatures))>
```

`signatures` は commit ID 計算に含めない。署名は commit ID を対象にする。

### 4.2 Commit

Commit は Git commit に近い履歴ノードである。

| Field | Required | Meaning |
|---|---:|---|
| `type` | yes | 常に `commit` |
| `parents` | yes | 親 commit ID。root commit は空配列、merge commit は複数 |
| `body` | yes | この commit の本文 snapshot ID |
| `author` | yes | 表示用の名前と任意 email |
| `created_at` | yes | UTC ISO 8601 |
| `message` | yes | checkpoint message。自動保存なら `"Autosave"` など |
| `source` | yes | writer app 情報 |
| `signatures` | no | 将来用。Ed25519 などの detached signature |

### 4.3 Snapshot

v1 では各 commit の本文をフル snapshot として保存する。

差分を保存しない理由:

- Markdown 文書は一般に小さく、bundle 全体を zstd 圧縮すれば重複はかなり縮む
- patch 適用失敗で履歴全体が壊れるリスクを避けられる
- 受信側は任意の 2 revision を直接 diff できる

将来の v2 で delta pack を追加しても、v1 reader はフル snapshot だけを読めばよい。

## 5. 現在本文と履歴の関係

`.mddiff` には常に現在本文がプレーン Markdown として存在する。Bundle の `head` は最後に
package 化された commit を指す。

最新本文は、互換性のために `.mddiff` の先頭へ平文で入り、履歴 bundle 内にも snapshot として
保持される。つまり最新本文は package 内で二重保持される。これは通常 Markdown として読める
ことを優先した設計上の代償である。

Reader は以下を計算する。

```text
current_body_id = b:sha256(canonical current Markdown body bytes)
head_body_id    = bundle.commits[bundle.head].body
```

- 一致する場合: 現在本文は `head` と一致している
- 不一致の場合: package 作成後に通常エディタ等で本文が編集された状態として扱う

不一致時も履歴は破棄しない。UI では Git の working tree と同様に
「head からの未 checkpoint 変更」として差分表示できる。

## 6. 操作モデル

### 6.1 基本コマンド

`.mddiff` は package なので、以下の操作を基本 API として定義する。

| Command | Meaning |
|---|---|
| `mddiff pack file.md` | `.md` と管理中の履歴から `file.mddiff` を作る |
| `mddiff unpack file.mddiff` | 本文 `.md` と管理用履歴へ展開する |
| `mddiff cat file.mddiff` | 最新本文だけを stdout に出す |
| `mddiff flatten file.mddiff` | 履歴なしの通常 `.md` を出力する |
| `mddiff history file.mddiff` | package 内の revision 一覧を表示する |
| `mddiff diff file.mddiff [rev]` | 現在本文と指定 revision、または head との差分を表示する |
| `mddiff verify file.mddiff` | bundle checksum、commit ID、snapshot ID を検証する |

### 6.2 Authoring Workflow

推奨ワークフロー:

```text
作業中:
  spec.md
  .mddiff/spec.history

共有・提出・アーカイブ:
  spec.mddiff
```

作業中の sidecar 形式はアプリ実装の内部形式であり、v1 protocol の互換性対象にはしない。
Protocol が保証する境界は `.mddiff` package の読み書きである。

### 6.3 Pack

`.md` から `.mddiff` を作る。

1. 現在の Markdown text から canonical body bytes を作り、snapshot ID を計算する
2. sidecar 履歴がなければ root commit を作る
3. sidecar 履歴がある場合は現在本文を新 snapshot として取り込み、必要なら commit を追加する
4. bundle を作り、package block として本文末尾に追加する

`pack` は配布用 artifact を作る操作であり、元の `.md` を置き換えなくてよい。

### 6.4 Unpack

受け取った `.mddiff` を作業用 `.md` と履歴へ展開する。

1. package を verify する
2. 本文を `file.md` として書き出す
3. bundle を app 管理下の sidecar 履歴へ展開する
4. 以後の authoring は `.md` を主ファイルとして行う

### 6.5 Cat / Flatten

`cat` は本文だけを stdout に出す。AI、CLI pipe、通常 Markdown tool へ渡す用途を想定する。

`flatten` は本文だけを `.md` として保存する。履歴は意図的に破棄されるため、共有前の
privacy 対策としても使う。

### 6.6 受け取った `.mddiff` を開く

1. 最後の MDV package block を探す
2. ペイロードを復号し、`bundle-size` と `bundle-sha256` を検証する
3. commit ID と snapshot ID を検証する
4. 現在本文 hash と `head` の snapshot ID を比較する
5. タイムライン、revision picker、diff を有効化する

検証に失敗した場合は、本文のみを通常 Markdown として開き、履歴は read-only invalid として扱う。

### 6.7 Checkpoint

Checkpoint は sidecar authoring 履歴、または `.mddiff` を直接編集する特殊ケースで使う。

1. 現在本文の snapshot がなければ `snapshots` に追加する
2. 新 commit を作り、`parents = [old_head]` とする
3. `head` と `refs.main` を新 commit に更新する
4. `pack` 時に最新 bundle として package block を再生成する

保存のたびに自動 checkpoint するか、ユーザー操作で checkpoint するかはアプリ UX の判断とする。
Protocol としては、単なる本文保存と checkpoint を分離して扱える。

### 6.8 2 つの `.mddiff` を統合する

同じ `repo_id` の bundle 同士は、`commits` と `snapshots` を union できる。

- `head` が片方の祖先なら fast-forward
- どちらも祖先でないなら複数 head として保持し、UI で merge または選択を促す
- merge commit は `parents` を複数持つ

異なる `repo_id` の場合は別文書として扱う。ユーザーが明示した場合のみ、片方を import して
graft commit を作る。

## 7. Diff

Diff は package 内の保存済み snapshot と現在本文から計算する。

代表的な比較:

- `head` vs current body: 未 checkpoint 変更
- selected commit vs current body: 受け取ったファイルと過去版の差分
- commit A vs commit B: 履歴内の差分

差分アルゴリズムは既存の `mddiff-core::diff` を利用できる。`.mddiff` プロトコルは
「比較対象の old/new text を供給する provider」として振る舞う。

## 8. AI Projection

AI 対応は副次用途であり、raw `.mddiff` 全体を LLM に直接読ませることは目標にしない。
Package block は圧縮済み payload を含むため、そのまま入力すると context を消費し、意味も
読めない。

AI や外部 tool には、`.mddiff` から必要な情報だけを projection して渡す。

| Command | Output |
|---|---|
| `mddiff cat file.mddiff` | 現在本文のみ |
| `mddiff ai-context file.mddiff` | 現在本文 + 直近 revision summary + 選択 diff |
| `mddiff ai-context file.mddiff --since <rev>` | 指定 revision 以降の要約と差分 |
| `mddiff ai-context file.mddiff --budget <tokens>` | token budget に収まるように履歴量を制限 |

AI 用 projection は package format の一部ではなく、reader / CLI の出力形式である。
将来 `summary` や `changed_sections` を commit metadata に追加できるが、v1 では必須にしない。

## 9. 既存実装との境界

Claude 側の Git 拡張と干渉しないため、実装は追加モジュールとして分ける。

推奨構成:

```text
crates/mddiff-core/src/
  mddiff_file.rs      # .mddiff package parse/write/verify/pack
  history.rs       # HistoryProvider trait と共通型
  git.rs           # 既存 Git provider。直接変更しない
  diff.rs          # 既存 diff。old/new text を受けるだけ
```

想定 API:

```rust
pub trait HistoryProvider {
    fn revisions(&self) -> Result<Vec<Revision>, HistoryError>;
    fn head(&self) -> Result<Option<RevisionId>, HistoryError>;
    fn text_at(&self, rev: &RevisionId) -> Result<String, HistoryError>;
    fn current_text(&self) -> &str;
}
```

`.mddiff` ファイルを開いた場合:

- デフォルトの履歴 provider は `MddiffPackageProvider`
- ファイルが Git 管理下にもある場合は、UI で `Embedded history` と `Git history` を切り替え可能にする
- 既存の `git_diff`, `git_full_diff`, `git_list_bases` command はそのまま維持する
- `.mddiff` 用 command は `mddiff_package_*` / `mddiff_ai_context` など別 namespace にする

## 10. 互換性と劣化動作

| 状況 | 期待動作 |
|---|---|
| `.mddiff` 対応アプリで開く | 本文、履歴、差分、unpack、flatten が使える |
| 通常 Markdown ビューアで開く | 本文部分が Markdown として表示される |
| 通常エディタで編集する | 履歴は残り、次回 mddiff で package 後の未 checkpoint 変更として検出される |
| コメントを削除するツールで保存する | 履歴は失われるが、本文は残る |
| package block が壊れる | 本文のみ開き、履歴は無効扱い |
| raw HTML を表示する特殊ビューア | コメント内容が source 上で見える可能性がある |
| AI / CLI に渡す | `mddiff cat` または `mddiff ai-context` で必要部分だけ出力する |

## 11. Privacy / Security

`.mddiff` は過去 snapshot を含むため、削除済みの文章もファイル内に残る。

必須機能:

- **Flatten export**: package block を取り除いた `.md` を出力する
- **Redact history**: 現在本文だけを root commit とする新しい `repo_id` で再生成する
- **Verify before trust**: commit ID、snapshot ID、bundle checksum を検証する
- **Package warning**: `.mddiff` 共有時に履歴を含むことを UI で明示する

Reader 側の安全制限:

- payload decode 後の bundle size 上限を設ける
- snapshot 数、commit 数、本文サイズに上限を設ける
- Markdown HTML は既存 Preview と同様に sanitize する
- 履歴 metadata を実行しない。外部 path、URL、command は解釈しない

## 12. MVP 範囲

最初の実装で入れるもの:

- `.mddiff` package parse/write/verify
- `zstd+base64` bundle
- full snapshot commit DAG
- `pack`, `unpack`, `cat`, `flatten`
- `history`, `diff`
- `head` vs current diff
- selected commit vs current diff
- broken package 時の本文 fallback

後回し:

- 署名
- delta pack
- 複数 head merge UI
- 履歴の部分削除 UI
- 他 `.mddiff` からの import/union
- AI projection の高度化（token budget 最適化、commit summary 自動生成）

## 13. Open Questions

- 自動 checkpoint の粒度: 保存ごと、一定時間ごと、明示操作のみのどれを標準にするか
- author 情報の初期値: OS ユーザー名を使うか、初回だけ設定させるか
- `.mddiff` が Git 管理下にある場合のデフォルト diff provider: embedded を優先するか、
  既存 Git diff を優先するか
- sidecar 履歴の内部保存場所: `.mddiff/` directory、OS app data、または文書横の hidden file
- package が巨大化したときの UI: 警告、compact、redact のどれをどの閾値で出すか
- `.mddiff` を直接編集した場合、保存時に自動 checkpoint するか、明示的 checkpoint だけにするか
