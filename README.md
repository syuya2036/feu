# Feu — A Hono-inspired, Rust Web Framework

[日本語版 README](README.ja.md)

**feu** (pronounced “foo”) is a next-generation web framework that borrows the *feel* of **Hono**—tiny, fast, middleware-first, web-standards friendly—while leaning into Rust’s strengths: **type safety**, **composable services**, and **multi-runtime adapters**.

This README describes the **intended public API and architecture** (a “spec-style README”) so implementation can track it.

This is an AI-produced PoC, and the features below are the roadmap. Some are implemented and some are not.

---

## Why feu?

* **Hono-like DX**: write handlers around a single `Ctx` (`c`) and return `c.text(...)`, `c.json(...)`, etc.
* **Middleware-first**: “onion model” middleware that can run *before* and *after* the next handler.
* **Tower at the core**: internally composed via `Service/Layer`, but with an outer API that still feels like Hono (`c, next`).
* **Fast routing**: radix-trie style matching for static routes, params (`:id`), and wildcards.
* **Multi-runtime**: core stays runtime-agnostic; adapters connect Hyper / AWS Lambda / Cloudflare Workers.
* **Batteries included**: logger, CORS, secure headers, timeout, request-id, ETag, compression, etc.
* **OpenAPI & RPC**: design includes first-class route metadata so OpenAPI + TS client/typegen can be built on top.

---

## Status

* **Design is “locked”** for `Ctx` style and middleware shape (see below).
* Implementation can roll out features in phases (router → middleware → adapters → OpenAPI/RPC).

---

## Quickstart (Conceptual)

> The code below is the **target** ergonomics.

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

### 2) Path params

```rust
app.get("/users/:id", |c: Ctx| async move {
    let id = c.param("id").unwrap_or("0");
    // c.text consumes the context
    Ok(c.text(format!("User ID: {}", id)))
});
```

### 3) Middleware

```rust
use feu::middleware::{logger, cors};

app.use_mw(logger::default());
app.use_mw(cors::Cors::permissive());

app.get("/healthz", |c: Ctx| async move {
    Ok(c.text("ok"))
});
```

---

## Core Ideas

### `Ctx` is the center (Hono-like)

Handlers receive a `Ctx` and produce a response via `c.text()`, `c.json()`, etc.

### `Ctx` uses the “A plan” (locked-in)

This is important: `c.status(...)` and `c.header(...)` **mutate the context**, storing **pending response settings** that are consumed when you call `c.text/json/html/...`.

That gives the classic Hono-style flow:

```rust
app.post("/posts", |mut c: Ctx| async move {
    c.status(StatusCode::CREATED);
    c.header("x-created", "1");
    // c.json is available with "json" feature. For now using text:
    Ok(c.text("{\"ok\": true}"))
});
```

**Consumption rule (locked-in):**

* Calling `c.text/json/html/body/redirect` **consumes** the pending status/headers and **resets them**.
* This avoids “sticky headers” surprises across multiple response creations inside the same handler.

### Middleware: Tower inside, Hono outside (locked-in)

Internally, feu composes middleware using a Tower-like `Service/Layer` pipeline.
Externally, users can write middleware in a Hono-ish style:

```rust
app.use_fn(|mut c: Ctx, next: feu::middleware::Next| async move {
    // before
    c.header("x-before", "1");

    let mut res = next.run(c).await?;

    // after
    // headers must be parsed types
    res = res.with_header("x-after".parse().unwrap(), "1".parse().unwrap());
    Ok(res)
});
```

You never touch Tower types directly—`next` is a small wrapper around the internal service chain.

---

## API at a Glance

### `App`

* Routing:

  * `get/post/put/delete/patch/options/head(path, handler)`
  * `any(path, handler)`
  * `on(methods, path, handler)`
* Composition:

  * `route(prefix, sub_app)` (nesting / grouping)
  * `base_path(prefix)`
  * `mount(prefix, service)` (advanced / later)
* Middleware:

  * `use_mw(mw)` (global)
  * `use_at(path, mw)` (scoped)
  * `use_fn(|c, next| ...)` (Hono-style functional middleware)
* Errors:

  * `not_found(handler)`
  * `on_error(handler)`
* Runtime entrypoint (for adapters):

  * `handle(req, env, runtime_ctx) -> FeuResponse`

### `Ctx` (the “c” object)

Request access:

* `c.req() -> &http::Request<FeuBody>`
* `c.method()`, `c.path()`
* `c.query("k") -> Option<&str>`
* `c.param("id") -> Option<&str>`

Type-safe request-scoped storage (TypeMap):

* `c.insert<T: 'static>(value: T)`
* `c.get<T: 'static>() -> Option<&T>`
* `c.extensions() / c.extensions_mut()`

Pending response configuration (A plan):

* `c.status(code) -> &mut Ctx`
* `c.header(name, value) -> &mut Ctx`
* `c.content_type(mime) -> &mut Ctx`

Response helpers (Hono-like):

* `c.text(body) -> FeuResponse`
* `c.html(body) -> FeuResponse`
* `c.json(value) -> FeuResponse` *(feature: `json`)*
* `c.body(bytes) -> FeuResponse`
* `c.redirect(location) -> FeuResponse` *(default 302; override via `c.status(307)` etc.)*
* `c.not_found() -> FeuResponse` *(simple 404 response)*

Env/runtime hooks (adapter-provided):

* `c.env() -> &Env`
* `c.runtime() -> &dyn RuntimeCtx` (e.g., `wait_until`, peer info)

---

## Response Layer

### Canonical types

feu’s canonical request/response types are:

* `http::Request<FeuBody>`
* `http::Response<FeuBody>`

### `FeuResponse`

`FeuResponse` is a thin wrapper over `http::Response<FeuBody>` for ergonomics and middleware convenience.

Typical helpers (conceptual):

* `FeuResponse::text(...)`, `::html(...)`, `::json(...)`
* `with_header(...)`, `with_status(...)`, `with_cookie(...)` *(feature-gated)*
* `into_inner()` for adapter conversion

### `FeuBody`

Unified body representation so the core doesn’t leak runtime-specific body types:

* `Empty`
* `Bytes`
* `Text`
* `Stream` *(feature: `streaming`)*

---

## Built-in Middleware (Batteries Included)

Each middleware is provided under `feu::middleware::*`.

### Observability / Ops

* **logger** — Log method/path/status/latency.
* **request_id** — Generate/propagate request IDs; inject into context.
* **timing** — Emit `Server-Timing` (or similar) metrics.
* **timeout** — Enforce request timeouts and return a timeout response.

### Security

* **cors** — CORS headers + preflight handling.
* **secure_headers** — Add standard security headers automatically.
* **csrf** — CSRF protection (strategy-based).
* **ip_restriction** — Allow/deny by IP/CIDR (peer info from adapter).

### Performance / Caching

* **etag** — Generate ETag and handle conditional requests (304).
* **cache** — Cache-Control helpers/policies.
* **compress** — Response compression (gzip/br/deflate) *(feature: `compress`)*.
* **body_limit** — Reject requests exceeding size limits early.

### Compatibility / Routing

* **method_override** — Override HTTP method via header/query.
* **trailing_slash** — Normalize/enforce trailing slash policy.

### DX / Presentation

* **pretty_json** — Dev-friendly JSON formatting.
* **language** — Locale negotiation helpers.
* **context_storage** — Task-local access to `Ctx` (use carefully).
* **combine** — Compose multiple middleware into one.
* **jsx_renderer** — SSR rendering hook (in Rust this typically maps to template engines).

### Auth

* **auth_basic** — Basic Auth guard.
* **auth_bearer** — Bearer token guard.
* **jwt** — JWT verification + claims injection *(feature)*.
* **jwk** — JWK fetching/caching for JWT *(feature)*.

---

## OpenAPI

feu is designed to generate an OpenAPI spec from **route + type metadata**.

Planned capabilities:

* Generate OpenAPI 3.x documents from routes, params, request/response schemas
* Serve `/openapi.json`
* Optional Swagger UI / Redoc (feature-gated)
* Stable output ordering for clean diffs

---

## RPC (Rust ↔ TypeScript)

Goal: Hono-like “end-to-end type safety” across Rust server and TS clients:

* Generate `.d.ts` from Rust types/routes
* Generate a lightweight fetch-based TS client
* Integrate via `feu-cli codegen` for monorepos

---

## Adapters (Multi-runtime)

Core is runtime-agnostic. Adapters translate platform types to/from canonical `http` types.

Planned adapters:

* **hyper** — primary dev/runtime baseline (first-class)
* **aws-lambda** — event ↔ request/response mapping
* **cloudflare-workers (wasm)** — Request/Response mapping + `wait_until` support via `RuntimeCtx`

---

## Directory Layout (Extensible by Default)

```text
feu/
  Cargo.toml
  README.md
  CHANGELOG.md
  LICENSE

  crates/
    feu/                     # re-export / prelude (primary user crate)
    feu-core/                # App / Ctx / handler / middleware wrapper / response helpers
    feu-router/              # fast router backend(s)
    feu-middleware/          # batteries included middleware
    feu-adapters/
      hyper/
      lambda/
      cloudflare-workers/
    feu-openapi/             # OpenAPI generation + serving
    feu-rpc/                 # TS types + client generation
    feu-cli/                 # dev/codegen/scaffold

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

## Feature Flags

* `default` — minimal core (routing + ctx + text/html)
* `json` — `c.json`, `Json<T>` extractor, serde_json integration
* `cookie` — cookie parsing/setting helpers
* `compress` — gzip/br/deflate compression
* `streaming` — streaming response bodies
* `schema` — JSON Schema derivation (for OpenAPI/RPC)
* `openapi` — OpenAPI generation + serving
* `rpc` — TS type/client generation

---

## Testing & Benchmarks

Minimum test coverage targets:

* Routing: static/params/wildcards, method separation, nesting
* `Ctx`: pending status/headers consumption + reset behavior
* Middleware: before/after order, short-circuit behavior
* Built-ins: basic behavior for logger/cors/timeout/etag
* OpenAPI: golden tests for stable spec output

Benchmark targets:

* Router throughput (static vs params vs wildcards; route count scaling)
* Middleware chain overhead (N layers)
* Feature impact (json/compress/openapi)

---

## License

Feu is licensed under MIT LICENSE．
