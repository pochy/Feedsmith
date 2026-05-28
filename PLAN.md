# Feedsmith MVP 実装計画

## Summary

  Rust + axum + Askama + htmx で、RSS 非対応サイトから CSS selector ベースで RSS 2.0 を生成する軽量セルフホストアプリを実装する。MVP では CSS selector のみ動作対象にし、DB/domain には
  selector_type を残して XPath 追加に備える。外部 URL 取得は SSRF 対策を最初から組み込む。

  採用技術:

- Backend: axum, tokio, tower-http, tracing, serde, thiserror, anyhow
- Fetch/parse/RSS: reqwest, scraper, rss, url, chrono
- DB: sqlx + SQLite
- HTML: Askama templates
- Frontend: htmx + 素の CSS + selector picker 用の最小 Vanilla JS
- Deploy: Docker multi-stage build + Docker Compose + SQLite volume

## Directory Structure

```
  Feedsmith/
    Cargo.toml
    Dockerfile
    docker-compose.yml
    .env.example
    README.md

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
        picker.rs

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
        sanitizer.rs

      db/
        mod.rs
        feed_repository.rs

      templates/
        layout.html
        index.html
        feeds/
          new.html
          edit.html
          _feed_table.html
          _preview.html
        picker.html

    static/
      app.css
      htmx.min.js
      selector-picker.js
```

  Alpine.js は MVP では導入しない。必要になった時だけ追加する。

## DB Schema

```sql
  CREATE TABLE feeds (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    site_url TEXT NOT NULL,
    feed_description TEXT,
    selector_type TEXT NOT NULL DEFAULT 'css'
      CHECK (selector_type IN ('css', 'xpath')),

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

  SQLite は PRAGMA foreign_keys = ON を接続時に有効化する。

## Key Modules

- config: APP_HOST, APP_PORT, DATABASE_URL, fetch timeout, body size limit, redirect limit, user agent を環境変数から読む。
- error: AppError に HTTP error, DB error, template error, fetch error, validation error を集約する。
- db::feed_repository: feeds CRUD、RSS 生成時の feed_items upsert、fallback 日付取得を担当する。
- services::url_guard: http/https のみ許可し、localhost/private/link-local/loopback を拒否する。DNS 解決後の IP も検証する。
- services::fetcher: redirect を手動追跡し、各 redirect 先を url_guard に通す。timeout、body size limit、Content-Type チェック、User-Agent を適用する。
- services::extractor: scraper で CSS selector を適用し、item/title/link/date/content を抽出する。相対 link は site_url 基準で絶対 URL 化する。
- services::rss_builder: rss crate で RSS 2.0 XML を生成する。
- services::selector_suggester: 指定 trait と NoopSelectorSuggester を用意する。
- services::sanitizer: picker 用 HTML から script, iframe, object, embed, event handler 属性を除去する簡易 sanitizer。
- routes::feeds: 一覧、新規、作成、編集、更新、削除。
- routes::preview: 抽出 preview と picker 用 fetch preview API。
- routes::rss: /feeds/:id/rss.xml。
- routes::picker: selector picker 画面。

## Routes

  GET  /                         Feed 一覧
  GET  /feeds/new                Feed 作成フォーム
  POST /feeds                    Feed 作成
  GET  /feeds/:id/edit           Feed 編集フォーム
  POST /feeds/:id                Feed 更新
  POST /feeds/:id/delete         Feed 削除

  POST /feeds/:id/preview        抽出結果 preview
  GET  /feeds/:id/rss.xml        RSS XML

  GET  /picker                   selector picker 画面
  POST /api/fetch-preview-html   sanitization 済み HTML を返す
  POST /api/suggest-selectors    Noop selector suggestion

  変更系はすべて POST にする。CSRF は MVP では未実装だが、form helper と middleware を後から差し込める構成にして README に明記する。

## Extraction Behavior

- MVP は selector_type = css のみ実装する。
- item_selector に一致する各要素を記事候補にする。
- title_selector は item 内で評価し、text を trim する。
- link_selector があれば該当要素の href を使う。
- link_selector が空なら、item 自身が a[href] ならそれを使い、なければ item 内の最初の a[href] を使う。
- link は site_url を base に絶対 URL 化する。
- date_selector が空または抽出不能なら、既存 feed_items の first_seen_at を使い、なければ現在時刻を保存して使う。
- preview では DB の feed_items は更新しない。RSS 生成時のみ first seen を保存する。

## Security Defaults

- 許可 scheme: http, https
- 拒否 host/IP: localhost, loopback, private, link-local, unspecified, multicast
- DNS 解決後の全 IP を検証し、拒否 IP が含まれる場合は fetch しない
- redirect は reqwest 自動追跡を無効化し、最大 5 回まで手動追跡する
- 各 redirect 先 URL も同じ validation を通す
- timeout default: 10 秒
- response body limit default: 2 MiB
- Content-Type は text/html, application/xhtml+xml, application/xml, text/xml を許可
- User-Agent default: Feedsmith/0.1
- fetch API は HTML preview/suggestion 目的に限定し、任意 URL の raw proxy にはしない

## UI Plan

- SSR HTML を Askama で生成し、htmx で form submit と preview fragment 更新を行う。
- 素の CSS で管理画面向けの低装飾・高可読な UI にする。
- Feed 一覧には name, site URL, RSS URL, edit/delete を表示する。
- 新規/編集フォームには selector 入力と preview ボタンを置く。
- Preview は _preview.html fragment として記事候補を最大 20 件表示する。
- Selector picker は srcdoc sandbox iframe に sanitization 済み HTML を入れ、hover highlight と click selector generation を最小 JS で実装する。

## Implementation Steps

  1. Rust project skeleton を作成し、依存関係、config、error、app router、tracing を整える。
  2. SQLite migrations と FeedRepository を実装し、CRUD と feed_items upsert を追加する。
  3. url_guard と fetcher を実装し、SSRF 対策、redirect 検証、timeout、body limit を入れる。
  4. extractor を実装し、CSS selector 抽出、相対 URL 解決、preview 用 pure function を作る。
  5. rss_builder と RSS route を実装し、first seen fallback を repository と連携する。
  6. Askama templates と htmx ベースの CRUD/preview UI を実装する。
  7. selector picker の簡易画面、sanitizer、selector-picker.js、fetch preview API を追加する。
  8. NoopSelectorSuggester と stub API を追加する。
  9. Dockerfile、docker-compose.yml、.env.example、README を整備する。
  10. unit tests を追加し、cargo test と可能なら cargo check を通す。

## Tests

  最低限の unit test:

- URL validation: scheme 拒否、localhost/private/link-local/loopback 拒否、public URL 許可
- private IP 判定: IPv4/IPv6 の代表ケース
- relative URL resolution: /post/1, post/1, absolute URL
- HTML extraction: item/title/link/content 抽出、空 link selector fallback
- RSS XML generation: channel/item/guid/pubDate/description を含む
- date fallback: 既存 first seen 使用、新規時 now 相当の値を使用

  可能なら追加:

- invalid CSS selector が user-facing validation error になる
- unsupported selector_type = xpath は明示エラーになる
- fetch body limit 超過で失敗する

## Assumptions

- 既存 repo は README/IDEA のみなので、新規 Rust project としてルート直下に実装する。
- CSS は素の CSS を採用し、Tailwind/Node build は使わない。
- htmx は static/htmx.min.js として同梱する前提にする。実装時にネットワーク取得が使えない場合は CDN 参照ではなく最小フォールバック方針を README に記載する。
- TypeScript build は MVP では導入せず、selector picker は手書きの Vanilla JS で実装する。将来 TS 化できるよう frontend/ は今回は作らないか、README 上の拡張余地として扱う。
- 認証と CSRF は MVP 非対象だが、POST route と shared app state/middleware 構成で後から追加可能にする。
