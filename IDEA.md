あなたはシニア Rust / Web アプリケーションエンジニアです。

これから、既存の Go + React 構成の RSS 生成ツールを参考にしつつ、Rust バックエンド + 軽量フロントエンド構成で、新しいセルフホスト型 RSS Generator を設計・実装してください。

# 背景

参考にしているのは、RSS を提供していない Web サイトから RSS フィードを生成するセルフホストツールです。

元の実装イメージは以下です。

* バックエンド: Go
* DB: SQLite
* フロントエンド: React
* 配信: nginx + Docker Compose
* 主な機能:

  * RSS がないサイトの URL を登録する
  * 対象サイトの HTML を取得する
  * title / link / date / content などの XPath を指定する
  * 指定した XPath から記事一覧を抽出する
  * RSS XML を生成する
  * 管理画面で設定を追加・編集・削除する
  * 対象ページをプレビューし、クリックした HTML 要素から XPath を取得する
  * AI を使って selector / XPath 候補を提案する

今回作りたいものは、元実装の単純な移植ではありません。

目的は、より軽量・安全・保守しやすい構成で、Rust を使って再設計することです。

# 目的

Rust を使って、軽量でセルフホストしやすい RSS Generator を作成してください。

重視することは以下です。

1. 低リソースで常時稼働できること
2. 小規模な個人サーバーで動くこと
3. Docker で簡単に起動できること
4. フロントエンドを過剰に重くしないこと
5. React SPA ではなく、サーバーサイド HTML + htmx 中心にすること
6. クリックで selector / XPath を選ぶ部分だけ、必要最小限の JavaScript / TypeScript を使うこと
7. 外部 URL を取得するため、SSRF や open proxy 的なリスクに最初から配慮すること
8. 将来的に AI selector suggestion を追加しやすい設計にすること
9. まず MVP を作り、その後に機能追加しやすい構成にすること

# 推奨技術構成

基本方針は以下です。

## Backend

* Rust
* axum
* tokio
* tower-http
* tracing
* serde
* thiserror
* anyhow
* reqwest
* scraper
* rss
* sqlx
* SQLite

## HTML rendering

* Askama

理由:

* Rust から type-safe に HTML template を扱いたい
* React SPA にせず、サーバーが HTML を返す構成にしたい
* htmx と相性の良い HTML fragment を返しやすい

## Frontend

* htmx
* Alpine.js は必要最小限
* selector picker 部分だけ Vanilla TypeScript
* React / Vue / Svelte / Solid は使わない

理由:

* このアプリの多くは CRUD 管理画面であり、完全な SPA は不要
* htmx でフォーム送信、一覧更新、プレビュー更新を HTML fragment として実現する
* 状態管理ライブラリを導入しない
* ブラウザ側で複雑な状態を持たない
* selector picker のように DOM 操作が必要な部分だけ TypeScript で書く

## CSS

以下のどちらかで提案してください。

第一候補:

* 素の CSS
* 軽量な class 設計

第二候補:

* Tailwind CSS CLI
* ただし Node.js 依存を最小限にする

今回の目的は軽量化なので、巨大なフロントエンドビルド環境は避けてください。

## Deployment

* Docker
* Docker Compose
* SQLite の volume mount
* Rust アプリ単体で静的ファイルも配信する
* nginx は必須にしない
* 外部公開する場合のみ Caddy / nginx の利用を検討する

# 実装方針

まず MVP を作ってください。

MVP に含める機能は以下です。

## MVP 機能

1. Feed 一覧画面
2. Feed 新規作成画面
3. Feed 編集画面
4. Feed 削除
5. 対象 URL の HTML 取得
6. CSS selector または XPath による記事抽出
7. 抽出結果の preview
8. RSS XML の生成
9. SQLite への設定保存
10. 基本的な SSRF 対策
11. Docker Compose で起動

AI selector suggestion は MVP では stub または設計だけでよいです。

# Selector 方針

可能であれば XPath より CSS selector を第一候補にしてください。

理由:

* Rust の scraper crate と相性が良い
* ブラウザの querySelector / querySelectorAll と対応しやすい
* フロントエンドとバックエンドで同じ selector を扱いやすい
* XPath より UI に表示しやすい

ただし、将来的に XPath にも対応できるように、DB schema や domain model は selector_type を持たせてください。

例:

* selector_type: "css" | "xpath"
* item_selector
* title_selector
* link_selector
* date_selector
* content_selector

MVP では CSS selector のみ実装しても構いません。
ただし、設計上は XPath を追加できるようにしてください。

# 重要なセキュリティ要件

このアプリは外部 URL を取得するため、必ず以下を考慮してください。

1. URL scheme は http / https のみに制限する
2. file://, ftp://, gopher:// などは禁止
3. localhost / 127.0.0.1 / ::1 へのアクセスは禁止
4. private IP range へのアクセスは禁止
5. link-local address へのアクセスは禁止
6. redirect 回数を制限する
7. redirect 先も同じ URL validation を通す
8. request timeout を設定する
9. response body size limit を設定する
10. Content-Type を確認する
11. User-Agent を明示する
12. 管理画面に簡易認証を追加できる設計にする
13. 変更系 API は GET ではなく POST を使う
14. CSRF token の導入を検討する
15. open proxy として使われない設計にする

MVP では最低限、URL validation / timeout / redirect 制限 / body size limit を実装してください。

# アプリケーション構成案

以下のような構成を提案してください。

```txt
rss-generator-rs/
  Cargo.toml
  Dockerfile
  docker-compose.yml
  README.md
  .env.example

  migrations/
    0001_create_feeds.sql
    0002_create_feed_items.sql

  src/
    main.rs
    app.rs
    config.rs
    error.rs

    routes/
      mod.rs
      feeds.rs
      preview.rs
      rss.rs
      static_files.rs

    domain/
      mod.rs
      feed.rs
      selector.rs
      extracted_item.rs

    services/
      mod.rs
      fetcher.rs
      extractor.rs
      rss_builder.rs
      url_guard.rs
      selector_suggester.rs

    db/
      mod.rs
      feed_repository.rs

    templates/
      layout.html
      index.html
      feeds/
        list.html
        new.html
        edit.html
        _feed_table.html
        _preview.html

  static/
    app.css
    htmx.min.js
    alpine.min.js
    selector-picker.js

  frontend/
    selector-picker.ts
```

必要に応じて改善して構いません。

# ルーティング案

以下をベースにしてください。

```txt
GET  /                         Feed 一覧
GET  /feeds/new                Feed 作成フォーム
POST /feeds                    Feed 作成
GET  /feeds/:id/edit           Feed 編集フォーム
POST /feeds/:id                Feed 更新
POST /feeds/:id/delete         Feed 削除

POST /feeds/:id/preview        抽出結果 preview
GET  /feeds/:id/rss.xml        RSS XML

GET  /picker                   selector picker 画面
POST /api/fetch-preview-html   対象 URL の HTML preview 用
POST /api/suggest-selectors    AI selector suggestion 用。MVP では stub 可。
```

# DB schema 案

SQLite で以下のような schema を作ってください。

```sql
CREATE TABLE feeds (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  name TEXT NOT NULL,
  site_url TEXT NOT NULL,
  feed_description TEXT,
  selector_type TEXT NOT NULL DEFAULT 'css',

  item_selector TEXT NOT NULL,
  title_selector TEXT NOT NULL,
  link_selector TEXT,
  date_selector TEXT,
  content_selector TEXT,

  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);

CREATE TABLE feed_items (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  feed_id INTEGER NOT NULL,
  item_url TEXT NOT NULL,
  title TEXT NOT NULL,
  first_seen_at TEXT NOT NULL,
  last_seen_at TEXT NOT NULL,

  UNIQUE(feed_id, item_url),
  FOREIGN KEY(feed_id) REFERENCES feeds(id) ON DELETE CASCADE
);
```

必要に応じて改善してください。

# 抽出仕様

対象 URL から HTML を取得し、item_selector に一致する要素を記事候補として扱ってください。

各 item の中で以下を抽出してください。

* title_selector: title
* link_selector: link
* date_selector: date
* content_selector: content

link_selector が空の場合:

* item 自身が a 要素なら href を使う
* item 内の最初の a[href] を使う

link が相対 URL の場合:

* site_url を base として絶対 URL に変換する

date が取れない場合:

* 既存の feed_items に item_url があれば first_seen_at を pubDate に使う
* なければ現在時刻を first_seen_at として保存し、pubDate に使う

# RSS 生成仕様

`GET /feeds/:id/rss.xml` で RSS 2.0 XML を返してください。

最低限含めるもの:

* channel title
* channel link
* channel description
* item title
* item link
* item guid
* item pubDate
* item description

RSS 生成には Rust の rss crate を使う想定です。

# UI 方針

React SPA ではなく、HTML をサーバーでレンダリングしてください。

htmx を使って以下を実現してください。

* Feed 追加後に一覧を部分更新
* Feed 編集後に一覧または詳細を部分更新
* Preview ボタンで抽出結果を部分更新
* Delete ボタンで該当行を削除または一覧を更新

Alpine.js は以下のような軽い UI のみに使ってください。

* modal
* accordion
* tabs
* show / hide
* loading state

複雑な状態管理は避けてください。

# selector picker 方針

selector picker は MVP では簡易実装で構いません。

目的:

* 対象 URL の HTML を preview する
* iframe または sandboxed container に HTML を表示する
* hover した要素を highlight する
* click した要素から CSS selector を生成する
* 生成した selector を親画面の input に反映する

注意:

* 外部サイトをそのまま iframe に入れると X-Frame-Options / CSP の問題が出るため、MVP では取得済み HTML を sandboxed iframe に srcdoc で表示する案を検討する
* script は除去する
* dangerous な tag / attribute は除去する
* 完璧な HTML sanitizer は MVP では難しいが、最低限 script / iframe / object / embed / event handler attributes は除去する
* selector picker は完全実装でなくてもよいが、拡張しやすい構造にする

# AI selector suggestion

将来的に Gemini / OpenAI / local LLM などで selector 候補を提案できるようにしたいです。

MVP では実装不要ですが、以下の trait を用意してください。

```rust
#[async_trait]
pub trait SelectorSuggester {
    async fn suggest(&self, html: &str, url: &str) -> Result<SelectorSuggestion, AppError>;
}
```

MVP では `NoopSelectorSuggester` を実装して、固定のメッセージまたは空の候補を返してください。

将来的に API key を環境変数から読み、Gemini / OpenAI 実装を差し替えられるようにしてください。

# 実装時の注意

以下を守ってください。

1. まず設計を簡潔に説明する
2. その後、ファイル構成を作る
3. MVP が動くところまで実装する
4. できるだけ小さなコミット単位で進めるつもりで作業する
5. 不要に抽象化しすぎない
6. ただし、fetch / extract / rss build / repository は分離する
7. エラーは AppError に集約する
8. tracing で基本ログを出す
9. README に起動方法を書く
10. Docker Compose で `docker compose up --build` できるようにする
11. `.env.example` を用意する
12. テストしやすい pure function を増やす

# テスト方針

最低限、以下の unit test を書いてください。

* URL validation
* private IP 判定
* relative URL から absolute URL への変換
* HTML から item 抽出
* RSS XML 生成
* 日付がない場合の fallback 挙動

可能であれば integration test も追加してください。

# 非目標

MVP では以下はやらなくてよいです。

* 完全な認証
* 複数ユーザー対応
* 高度な権限管理
* 完璧な HTML sanitizer
* 完全な XPath 対応
* 完全な AI selector suggestion
* Kubernetes 対応
* PostgreSQL 対応
* React / Vue / Svelte SPA 化

# 期待する出力

まず以下を出してください。

1. この要件に対する実装計画
2. 採用技術の理由
3. ディレクトリ構成
4. DB schema
5. 主要な Rust module の役割
6. 実装ステップ

その後、実際にコードを生成・編集してください。

作業中に判断が必要な場合は、軽量性・安全性・保守性を優先して、妥当なデフォルトで進めてください。

