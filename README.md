# Feedsmith

Feedsmith は、RSS フィードを配信していない Web サイト向けの、軽量な
セルフホスト RSS ジェネレーターです。

フィード定義を SQLite に保存し、SSRF 対策をかけながら対象ページを取得し、
CSS セレクタで繰り返し項目を抽出して、RSS 2.0 XML として配信します。

## 機能

- フィード一覧、作成、編集、削除画面
- Askama によるサーバーレンダリング HTML
- htmx 風の軽量なフォーム更新
- CSS セレクタベースの抽出
- セレクタ変更を保存する前に抽出結果をプレビュー
- `/feeds/:id/rss.xml` で RSS 2.0 を出力
- SQLite による永続化
- Docker Compose デプロイ
- 外部 URL 取得時の基本的な SSRF 保護
- サニタイズ済み `srcdoc` iframe HTML を使った最小限のセレクタピッカー
- 将来の AI セレクタ提案用スタブエンドポイント

現在の MVP は CSS セレクタのみを実装しています。スキーマ上は
`selector_type` で XPath も表現できますが、XPath 抽出は意図的にまだ有効化
していません。

## ローカル実行

```sh
cargo run
```

<http://127.0.0.1:3000> を開きます。

ローカル SQLite ファイルを明示的に指定する場合:

```sh
DATABASE_URL=sqlite://feedsmith.db cargo run
```

## Docker Compose

```sh
docker compose up --build
```

<http://localhost:3000> を開きます。

SQLite データは `/data` にマウントされた `feedsmith-data` Docker ボリュームに
保存されます。

## 設定

`.env.example` を参考にしてください。

| 変数 | デフォルト | 説明 |
| --- | --- | --- |
| `APP_HOST` | `127.0.0.1` | ローカル実行時のバインドホスト |
| `APP_PORT` | `3000` | HTTP ポート |
| `DATABASE_URL` | `sqlite://feedsmith.db` | SQLite データベース URL |
| `FETCH_TIMEOUT_SECS` | `10` | リクエストごとのタイムアウト |
| `FETCH_MAX_BODY_BYTES` | `2097152` | レスポンス本文の最大サイズ |
| `FETCH_MAX_REDIRECTS` | `5` | 手動リダイレクトの最大回数 |
| `USER_AGENT` | `Feedsmith/0.1` | 取得時に使う User-Agent |

Docker Compose では `APP_HOST=0.0.0.0` と
`DATABASE_URL=sqlite:///data/feedsmith.db` が設定されます。

## セレクタモデル

各フィードでは以下を使用します。

- `item_selector`: 繰り返される記事またはコンテナ要素
- `title_selector`: 各項目内のタイトル
- `link_selector`: 各項目内の任意のリンク要素
- `date_selector`: 各項目内の任意の日付要素
- `content_selector`: 各項目内の任意の要約または本文要素

`link_selector` が空の場合、Feedsmith は item 自体が `a[href]` ならそれを使い、
そうでない場合は item 内の最初の `a[href]` を使います。

相対リンクは `site_url` を基準に解決されます。

例:

```txt
site_url: https://example.com/news/
item_selector: article
title_selector: h2
link_selector: a
date_selector: time
content_selector: .summary
```

## セレクタピッカーの使い方

セレクタピッカーは、対象ページを調べ、要素をクリックして CSS セレクタ候補を
生成するための補助ツールです。

ピッカーを開く:

```txt
http://localhost:3000/picker
```

または、同じ LAN 上の別のマシンから開く場合:

```txt
http://<server-ip>:3000/picker
```

基本的な流れ:

1. `URL` フィールドに対象ページの URL を入力します。
2. `Fetch` をクリックします。
3. Feedsmith がサーバー側でページを取得します。
4. 取得した HTML は軽くサニタイズされ、サンドボックス化された iframe 内に表示されます。
5. iframe 内の要素にマウスを移動します。
6. ホバー中の要素がアウトライン表示されます。
7. 目的の要素をクリックします。
8. `Selected selector` に CSS セレクタが表示されます。
9. そのセレクタをフィード作成または編集フォームへコピーします。
10. 編集画面の `Preview` で抽出結果を確認します。

ピッカーは完璧なセレクタを出すものではなく、補助ツールとして使ってください。
ページに入れ子のコンテナや生成されたクラス名が多い場合、生成されるセレクタは
細かすぎることがあります。

### セレクタの選び方

まず `item_selector` から選びます。これはタイトル文字列だけではなく、繰り返し
表示される記事項目全体に一致する必要があります。

よい `item_selector` の例:

```txt
article
.news-item
.entry
li.post
div.card
```

`item_selector` を選んだら、各 item 内で評価されるセレクタを選びます。

```txt
title_selector: h2
link_selector: h2 a
date_selector: time
content_selector: .summary
```

次の HTML の場合:

```html
<article class="post">
  <h2><a href="/news/1">Article title</a></h2>
  <time datetime="2026-05-28">May 28, 2026</time>
  <p class="summary">Article summary.</p>
</article>
```

以下を使います。

```txt
item_selector: article.post
title_selector: h2
link_selector: h2 a
date_selector: time
content_selector: .summary
```

### 生成されたセレクタを簡略化する

ピッカーは次のようなセレクタを生成することがあります。

```txt
body > main > section:nth-of-type(2) > article.post:nth-of-type(3) > h2
```

これは多くの場合、細かすぎます。可能であれば、より短いセレクタを優先します。

```txt
article.post
h2
.summary
```

簡略化したセレクタが正しい要素に一致するかどうかは、ブラウザの DevTools や
Feedsmith のプレビューで確認してください。

### ピッカーでよくある問題

要素がハイライトされない場合:

- ピッカーページをハードリロードし、最新の `selector-picker.js` が読み込まれるようにします。
- ページが取得され、iframe 内に表示されていることを確認します。
- ブラウザコンソールでピッカー初期化エラーを確認します。
- 一部のページはクライアント側 JavaScript に強く依存しています。Feedsmith は HTML を取得しますが、対象サイトのスクリプトは実行しません。

iframe の表示が崩れている場合:

- 対象ページが JavaScript レンダリングを必要としている可能性があります。
- 相対 CSS URL や画像 URL が、元サイトとまったく同じようには解決されない場合があります。
- MVP のサニタイザーは、`script`、`iframe`、`object`、`embed` タグと、インラインイベントハンドラ属性を削除します。

セレクタのプレビュー結果が空の場合:

- `item_selector` が繰り返される記事コンテナに一致しているか確認します。
- `title_selector` がドキュメント全体のルートではなく、各 item の内側で評価されることを確認します。
- 各 item 内の最初のリンクが記事 URL であれば、`link_selector` は空のままにします。
- 生成されたセレクタに多くの `nth-of-type` セグメントが含まれている場合は、より短いセレクタを試します。

## ルート

```txt
GET  /                         フィード一覧
GET  /feeds/new                作成フォーム
POST /feeds                    フィード作成
GET  /feeds/:id/edit           編集フォーム
POST /feeds/:id                フィード更新
POST /feeds/:id/delete         フィード削除
POST /feeds/:id/preview        抽出プレビュー
GET  /feeds/:id/rss.xml        RSS XML
GET  /picker                   セレクタピッカー
POST /api/fetch-preview-html   サニタイズ済みピッカー HTML の取得
POST /api/suggest-selectors    処理なしの AI 提案スタブ
```

## セキュリティメモ

Feedsmith は外部 URL を取得するため、fetcher は MVP として以下の保護を適用します。

- `http` と `https` のみ許可
- localhost、loopback、private、link-local、multicast、unspecified IP アドレスを拒否
- リクエスト前に DNS を解決し、解決された IP アドレスを検証
- 自動リダイレクトを無効化し、各リダイレクト先を手動で検証
- リダイレクト回数を制限
- リクエストタイムアウトを設定
- レスポンス本文サイズを制限
- 基本的な HTML/XML コンテンツタイプを確認
- 明示的な User-Agent を送信

MVP には認証や CSRF 保護は含まれていません。公開する前に、信頼できる
プライベートネットワーク上で実行するか、認証付きリバースプロキシの背後に
配置してください。

## 開発

フォーマットとテストを実行:

```sh
cargo fmt -- --check
cargo test
```

主な実装領域:

- `src/routes`: HTTP ハンドラ
- `src/templates`: Askama テンプレート
- `src/db`: SQLite リポジトリ
- `src/services/fetcher.rs`: 保護付き外部取得
- `src/services/url_guard.rs`: URL/IP 検証
- `src/services/extractor.rs`: CSS 抽出と URL 解決
- `src/services/rss_builder.rs`: RSS XML 生成

## 現在の制限

- CSS セレクタ抽出のみ
- 認証なし
- CSRF トークンミドルウェアなし
- セレクタピッカーのサニタイザーは意図的に最小限
- AI セレクタ提案エンドポイントは固定の処理なしレスポンスを返す
