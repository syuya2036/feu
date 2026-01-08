# Feu — Honoに着想を得たRust Webフレームワーク

**feu**（発音は「foo」）は、**Hono**のような小さく速いミドルウェア指向の体験を取り込みつつ、Rustの強みである**型安全性**、**合成可能なサービス**、**マルチランタイム対応**に重点を置いた次世代Webフレームワークです。

このREADMEは**想定される公開APIとアーキテクチャ**（仕様書的なREADME）を記述しており、実装がそれに追従できるようにするためのものです。

---

## なぜfeuなのか？

* **Hono風DX**：単一の`Ctx`（`c`）を中心にハンドラを書き、`c.text(...)`や`c.json(...)`などでレスポンスを返す。
* **ミドルウェア・ファースト**：「玉ねぎモデル」で、`next`の前後に処理を差し込める。
* **内部はTower**：内部では`Service/Layer`で合成しつつ、外部APIはHonoライク（`c, next`）。
* **高速ルーティング**：静的/パラメータ（`:id`）/ワイルドカードに対応したラディックストライ。
* **マルチランタイム**：コアはランタイム非依存。Hyper/AWS Lambda/Cloudflare Workersなどにアダプタで接続。
* **バッテリー同梱**：logger、CORS、セキュアヘッダ、タイムアウト、request-id、ETag、圧縮など。
* **OpenAPI & RPC**：ルートメタデータを第一級に扱い、OpenAPIやTS型/クライアント生成を可能にする設計。

---

## ステータス

* `Ctx`のスタイルとミドルウェア形状は**設計固定**（後述）。
* 実装はフェーズ分けで進められる（ルータ → ミドルウェア → アダプタ → OpenAPI/RPC）。

---

## クイックスタート（概念）

> 以下は**目指す使用感**のサンプルです。

### 1) Hello, feu

```rust
use feu::prelude::*;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut app = App::new();

    app.get("/", |c: Ctx| async move {
        Ok(c.text("Hello, feu!"))
    });

    feu::adapters::hyper::serve(app, "0.0.0.0:3000").await?;
    Ok(())
}
```

### 2) パスパラメータ

```rust
app.get("/users/:id", |c: Ctx| async move {
    let id = c.param("id").unwrap_or("0");
    // c.textはコンテキストを消費する
    Ok(c.text(format!("User ID: {}", id)))
});
```

### 3) ミドルウェア

```rust
use feu::middleware::{logger, cors};

app.use_mw(logger::default());
app.use_mw(cors::Cors::permissive());

app.get("/healthz", |c: Ctx| async move {
    Ok(c.text("ok"))
});
```

---

## コアの考え方

### `Ctx`が中心（Honoライク）

ハンドラは`Ctx`を受け取り、`c.text()`や`c.json()`などでレスポンスを生成します。

### `Ctx`は「Aプラン」を採用（固定）

`c.status(...)`と`c.header(...)`は**コンテキストを更新**し、**保留中のレスポンス設定**を保持します。`c.text/json/html/...`を呼ぶと、それらが消費されます。

Honoと同様のフローになります：

```rust
app.post("/posts", |mut c: Ctx| async move {
    c.status(StatusCode::CREATED);
    c.header("x-created", "1");
    // c.jsonは"json" featureで利用可能。ここではtextを使用。
    Ok(c.text("{\"ok\": true}"))
});
```

**消費ルール（固定）**

* `c.text/json/html/body/redirect`を呼ぶと、保留中のステータス/ヘッダを**消費**し、**リセット**します。
* これにより、同一ハンドラ内で複数回レスポンス生成した際の「ヘッダの粘着」を防ぎます。

### ミドルウェア：内部はTower、外部はHono（固定）

内部ではTower風の`Service/Layer`で合成しつつ、外からはHonoライクに書けます：

```rust
app.use_fn(|mut c: Ctx, next: feu::middleware::Next| async move {
    // 前処理
    c.header("x-before", "1");

    let mut res = next.run(c).await?;

    // 後処理
    // headersはparse済みの型
    res = res.with_header("x-after".parse().unwrap(), "1".parse().unwrap());
    Ok(res)
});
```

Towerの型を直接触る必要はなく、`next`は内部チェーンのラッパです。

---

## API概要

### `App`

* ルーティング：

  * `get/post/put/delete/patch/options/head(path, handler)`
  * `any(path, handler)`
  * `on(methods, path, handler)`
* 構成：

  * `route(prefix, sub_app)`（ネスト/グループ化）
  * `base_path(prefix)`
  * `mount(prefix, service)`（高度/後日）
* ミドルウェア：

  * `use_mw(mw)`（グローバル）
  * `use_at(path, mw)`（スコープ付き）
  * `use_fn(|c, next| ...)`（Hono風関数ミドルウェア）
* エラー：

  * `not_found(handler)`
  * `on_error(handler)`
* ランタイム入口（アダプタ用）：

  * `handle(req, env, runtime_ctx) -> FeuResponse`

### `Ctx`（“c”オブジェクト）

リクエスト参照：

* `c.req() -> &http::Request<FeuBody>`
* `c.method()`, `c.path()`
* `c.query("k") -> Option<&str>`
* `c.param("id") -> Option<&str>`

型安全なリクエストスコープストレージ（TypeMap）：

* `c.insert<T: 'static>(value: T)`
* `c.get<T: 'static>() -> Option<&T>`
* `c.extensions() / c.extensions_mut()`

保留中レスポンス設定（Aプラン）：

* `c.status(code) -> &mut Ctx`
* `c.header(name, value) -> &mut Ctx`
* `c.content_type(mime) -> &mut Ctx`

レスポンスヘルパ（Honoライク）：

* `c.text(body) -> FeuResponse`
* `c.html(body) -> FeuResponse`
* `c.json(value) -> FeuResponse` *(feature: `json`)*
* `c.body(bytes) -> FeuResponse`
* `c.redirect(location) -> FeuResponse` *(デフォルト302。`c.status(307)`等で上書き)*
* `c.not_found() -> FeuResponse` *(簡易404)*

Env/ランタイムフック（アダプタ提供）：

* `c.env() -> &Env`
* `c.runtime() -> &dyn RuntimeCtx`（例：`wait_until`、peer情報）

---

## レスポンスレイヤ

### 標準型

feuの標準リクエスト/レスポンス型：

* `http::Request<FeuBody>`
* `http::Response<FeuBody>`

### `FeuResponse`

`FeuResponse`は`http::Response<FeuBody>`の薄いラッパで、操作性とミドルウェア互換性を提供します。

想定されるヘルパ：

* `FeuResponse::text(...)`, `::html(...)`, `::json(...)`
* `with_header(...)`, `with_status(...)`, `with_cookie(...)` *(feature-gated)*
* `into_inner()`（アダプタ変換用）

### `FeuBody`

ランタイム依存を避けるための統一ボディ表現：

* `Empty`
* `Bytes`
* `Text`
* `Stream` *(feature: `streaming`)*

---

## 内蔵ミドルウェア（バッテリー同梱）

各ミドルウェアは`feu::middleware::*`に提供予定。

### 観測/運用

* **logger** — メソッド/パス/ステータス/レイテンシをログ出力。
* **request_id** — リクエストIDの生成/伝播。コンテキストに挿入。
* **timing** — `Server-Timing`等のメトリクスを付与。
* **timeout** — タイムアウトを強制し、タイムアウトレスポンスを返す。

### セキュリティ

* **cors** — CORSヘッダとプリフライト処理。
* **secure_headers** — 標準的なセキュリティヘッダを追加。
* **csrf** — CSRF保護（戦略ベース）。
* **ip_restriction** — IP/CIDRで許可/拒否（アダプタがpeer情報提供）。

### パフォーマンス/キャッシュ

* **etag** — ETag生成と条件付きリクエスト処理（304）。
* **cache** — Cache-Controlのヘルパ/ポリシー。
* **compress** — レスポンス圧縮（gzip/br/deflate）*(feature: `compress`)*。
* **body_limit** — リクエストサイズ超過を早期拒否。

### 互換性/ルーティング

* **method_override** — ヘッダ/クエリでHTTPメソッドを上書き。
* **trailing_slash** — 末尾スラッシュの正規化/強制。

### DX/表示

* **pretty_json** — 開発向けのJSON整形。
* **language** — ロケールネゴシエーション補助。
* **context_storage** — タスクローカルで`Ctx`にアクセス（注意して使用）。
* **combine** — 複数ミドルウェアを合成。
* **jsx_renderer** — SSRレンダリングフック（Rustではテンプレートエンジン連携）。

### 認証

* **auth_basic** — Basic認証。
* **auth_bearer** — Bearerトークン認証。
* **jwt** — JWT検証とクレーム注入 *(feature)*。
* **jwk** — JWT用JWKの取得/キャッシュ *(feature)*。

---

## OpenAPI

feuは**ルート+型メタデータ**からOpenAPI仕様を生成できる設計です。

想定機能：

* ルート/パラメータ/リクエスト/レスポンススキーマからOpenAPI 3.xを生成
* `/openapi.json`を提供
* Swagger UI / Redoc（feature-gated）
* 安定した出力順序で差分を綺麗に

---

## RPC（Rust ↔ TypeScript）

目標：Honoのような「エンドツーエンド型安全」をRustサーバとTSクライアント間で実現。

* Rustの型/ルートから`.d.ts`を生成
* 軽量なfetchベースTSクライアント生成
* `feu-cli codegen`でモノレポ統合

---

## アダプタ（マルチランタイム）

コアはランタイム非依存。アダプタがプラットフォーム型を標準`http`型に変換します。

予定アダプタ：

* **hyper** — 主要開発/実行ベースライン（第一級）
* **aws-lambda** — イベント↔リクエスト/レスポンスの変換
* **cloudflare-workers (wasm)** — Request/Response変換 + `wait_until`を`RuntimeCtx`で提供

---

## ディレクトリ構成（拡張前提）

```text
feu/
  Cargo.toml
  README.md
  CHANGELOG.md
  LICENSE

  crates/
    feu/                     # re-export / prelude（利用者向け）
    feu-core/                # App / Ctx / handler / middleware wrapper / response helpers
    feu-router/              # 高速ルータバックエンド
    feu-middleware/          # バッテリー同梱ミドルウェア
    feu-adapters/
      hyper/
      lambda/
      cloudflare-workers/
    feu-openapi/             # OpenAPI生成 + 配信
    feu-rpc/                 # TS型 + クライアント生成
    feu-cli/                 # 開発/コード生成/スキャフォールド

  examples/
    hello-hyper/
    restful-api/
    middleware-stack/
    openapi-rest/
    rpc-monorepo/
    hello-lambda/
    hello-workers/

  docs/
    architecture.md
    middleware.md
    openapi.md
    rpc.md
    adapters.md

  benches/
    router.rs
    middleware_chain.rs
```

---

## Featureフラグ

* `default` — 最小コア（routing + ctx + text/html）
* `json` — `c.json`, `Json<T>`エクストラクタ, serde_json統合
* `cookie` — cookie解析/設定ヘルパ
* `compress` — gzip/br/deflate圧縮
* `streaming` — ストリーミングレスポンス
* `schema` — JSON Schema導出（OpenAPI/RPC向け）
* `openapi` — OpenAPI生成 + 配信
* `rpc` — TS型/クライアント生成

---

## テスト & ベンチマーク

最低限のテスト対象：

* ルーティング：静的/パラメータ/ワイルドカード、メソッド分離、ネスト
* `Ctx`：保留中ステータス/ヘッダの消費とリセット
* ミドルウェア：前後順、短絡動作
* 組み込み：logger/cors/timeout/etagの基本挙動
* OpenAPI：安定した出力順序のゴールデンテスト

ベンチマーク対象：

* ルータスループット（静的/パラメータ/ワイルドカード；ルート数スケール）
* ミドルウェアチェーンのオーバーヘッド（N層）
* feature影響（json/compress/openapi）

---

## ライセンス

FeuはMIT LICENSEでライセンスされています。
