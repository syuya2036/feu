# feu Implementation Plan (Phased, Comprehensive Checklist)

> Everything is written in `- [ ]` checklist format (no normal bullet lists).
> This plan assumes the **locked-in design**:
> - `Ctx` is **A-plan** (pending status/headers consumed by `c.text/json/html/...`)
> - Middleware is **Tower internally**, but **Hono-like externally** (`use_fn(|c, next| async move { ... })`)
> - Canonical I/O is `http::Request/Response<FeuBody>` with `FeuResponse` as a thin wrapper
> - Adapters: Hyper first, then Lambda, then Workers
> - OpenAPI sits **after adapters** and shares route metadata IR with RPC where possible

---

## Phase 0 — Repo & Workspace Foundation

- [x] Create workspace root `Cargo.toml` with members
- [x] Create crate skeletons
- [x] Create `crates/feu` (user-facing re-export/prelude)
- [x] Create `crates/feu-core` (App/Ctx/Response/Middleware wrapper)
- [x] Create `crates/feu-router` (fast router backend)
- [x] Create `crates/feu-middleware` (built-in middleware set)
- [x] Create `crates/feu-adapters/hyper`
- [x] Create `crates/feu-adapters/lambda` (stub)
- [x] Create `crates/feu-adapters/cloudflare-workers` (stub)
- [x] Create `crates/feu-openapi` (stub, gated)
- [x] Create `crates/feu-rpc` (stub, gated)
- [x] Create `crates/feu-cli` (stub)
- [x] Add repository files
- [x] Add `README.md` (spec README aligned to locked-in design)
- [x] Add `CHANGELOG.md` (Keep a Changelog)
- [x] Add `LICENSE` (choose MIT)
- [x] Add `.gitignore` (Rust + editors)
- [x] Add formatting + lint configs
- [x] Add `rustfmt.toml` (optional)
- [x] Decide clippy policy and document it
- [x] Add CI workflow
- [x] Add `cargo fmt --check` to CI
- [x] Add `cargo clippy -- -D warnings` to CI
- [x] Add `cargo test --workspace` to CI
- [x] Add `cargo test --workspace --all-features` to CI
- [x] Add `cargo test --workspace --no-default-features` to CI
- [x] Add `cargo doc --workspace` to CI (optional warn-as-error)
- [x] Establish feature policy (document in `crates/feu/Cargo.toml`)
- [x] Ensure default features are minimal core
- [x] Add feature flags placeholders: `json`, `cookie`, `compress`, `streaming`, `schema`, `openapi`, `rpc`
- [x] Create `docs/` folder with placeholders (architecture/middleware/adapters/openapi/rpc)
- [x] Confirm workspace builds with `cargo build` and passes CI locally

---

## Phase 1 — feu-core Minimal Kernel (Hello World)

- [x] Add core dependencies (`http`, `bytes`, minimal error crates) to `feu-core`
- [x] Implement `Error` type (core)
- [x] Define `ErrorKind` (Route/Parse/Timeout/NotFound/Internal/etc.)
- [x] Implement `Display` for `Error`
- [x] Implement `std::error::Error` for `Error`
- [x] Define `Result<T>` alias
- [x] Define canonical types
- [x] Define `FeuRequest = http::Request<FeuBody>`
- [x] Define `InnerResponse = http::Response<FeuBody>`
- [x] Implement `FeuBody` (minimal)
- [x] Add `FeuBody::Empty`
- [x] Add `FeuBody::Bytes(bytes::Bytes)`
- [x] Add `FeuBody::Text(String)`
- [x] Add conversions into `FeuBody`
- [x] Implement `From<()> for FeuBody`
- [x] Implement `From<Bytes> for FeuBody`
- [x] Implement `From<String> for FeuBody`
- [x] Implement `From<&'static str> for FeuBody`
- [x] Implement `FeuResponse` wrapper
- [x] Add `pub struct FeuResponse(pub http::Response<FeuBody>)`
- [x] Add `into_inner/as_inner/as_inner_mut`
- [x] Add minimal constructors on `FeuResponse`
- [x] Add `FeuResponse::empty(StatusCode)`
- [x] Add `FeuResponse::text(...)` with content-type
- [x] Add `FeuResponse::html(...)` with content-type
- [x] Add `FeuResponse::redirect(location, code)` with Location header
- [x] Add fluent methods on `FeuResponse`
- [x] Add `with_status(code)` (non-mut style)
- [x] Add `with_header(name, value)` (non-mut style)
- [x] Add `with_content_type(mime)` (non-mut style)
- [x] Implement `IntoResponse` trait
- [x] Implement `IntoResponse for FeuResponse`
- [x] Implement `IntoResponse for http::Response<FeuBody>`
- [x] Implement `IntoResponse for (StatusCode, T: IntoResponse)`
- [x] Implement `IntoResponse for Result<T: IntoResponse, Error>`
- [x] Implement `Ctx` minimal (no routing params yet)
- [x] Add `Ctx` fields: `req`, `extensions`, `pending_status`, `pending_headers`, `pending_content_type`
- [x] Implement `c.req()`, `c.method()`, `c.path()`
- [x] Implement `c.extensions()/extensions_mut()`
- [x] Implement `c.insert<T>()` and `c.get<T>()`
- [x] Implement A-plan pending response configuration (locked-in)
- [x] Implement `c.status(code) -> &mut Ctx` that sets pending status
- [x] Implement `c.header(name, value) -> &mut Ctx` that appends pending header
- [x] Implement `c.content_type(mime) -> &mut Ctx` that sets pending content-type
- [x] Implement pending consumption rule (locked-in)
- [x] Implement internal method `c.take_pending()` that returns (status, headers, content-type) and resets them
- [x] Implement response helpers on `Ctx`
- [x] Implement `c.text(body) -> FeuResponse` consuming pending settings
- [x] Implement `c.html(body) -> FeuResponse` consuming pending settings
- [x] Implement `c.body(bytes) -> FeuResponse` consuming pending settings
- [x] Implement `c.redirect(location) -> FeuResponse` consuming pending settings (default 302 unless pending overrides)
- [x] Implement `c.not_found() -> FeuResponse` (simple 404)
- [x] Implement `App` minimal
- [x] Implement `App::new()`
- [x] Define handler type erasure strategy (BoxFuture or trait object)
- [x] Implement a minimal GET-only route table (temporary until router phase)
- [x] Implement `app.get(path, handler)` using the temporary table
- [x] Implement `app.handle(req, env, runtime)` minimal entrypoint
- [x] Implement default 404 when route missing
- [x] Add unit tests
- [x] Test `c.status()+c.header()+c.text()` behavior (consumption + reset)
- [x] Test `c.text()` content-type correctness
- [x] Test `c.redirect()` sets Location header
- [x] Test minimal `App` GET route returns expected response
- [x] Ensure `no-default-features` build passes

---

## Phase 2 — Router Engine (Fast Matching + Params) & Full Method API

- [x] Create `feu-router` crate API surface
- [x] Define `Router` trait (method+path -> match)
- [x] Define `HandlerId` type (stable index)
- [x] Define `RouteMatch { handler_id, params }`
- [x] Define `Params` storage format (small vec of pairs)
- [x] Implement `Params::get(name) -> Option<&str>`
- [x] Implement fast router backend (radix-trie style; matchit-like)
- [x] Implement per-method router tables (GET/POST/PUT/DELETE/PATCH/OPTIONS/HEAD)
- [x] Implement insert(method, path, handler_id)
- [x] Implement recognize(method, path) -> RouteMatch
- [x] Support :param capture
- [x] Support wildcard * capture
- [x] Integrate router into App
- [x] Replace temporary GET-only table with router backend
- [x] Implement handler registry in App (Vec of handlers)
- [x] Update App::handle to call router and dispatch handler by id
- [x] Add param support to Ctx
- [x] Implement c.param(key)
- [x] Decide param conversion rules (string-only first)
- [x] Expand routing API (post, put, delete, etc.)
- [x] Add app.any and app.on
- [x] Add composition API (app.route)
- [ ] Add app.base_path(prefix) (Optional/Phase 3)
- [x] Define route merging rules
- [ ] Add 404 customization hooks (Phase 3)
- [x] Add router tests
- [x] Test static path match
- [x] Test param match :id
- [x] Test wildcard match *
- [x] Test nested routes
- [x] Test all methods
- [x] Validate c.param("id") returns expected value

---

## Phase 3 — Middleware Engine (Tower Core) + Hono-like `use_fn` Wrapper + Error Hooks

- [x] Choose internal middleware architecture (Tower-like Service/Layer)
- [x] Define internal `Service` trait bounds (Next/BoxFuture)
- [x] Define internal `Layer` composition strategy (App::use_mw)
- [x] Implement `RouterService` (closure in handle)
- [x] Implement `AppService` pipeline builder (Next chaining)
- [x] Implement `app.use(mw)` for built-in middleware layers
- [x] Implement `app.use_at(path, mw)` (Phase 4/Future)
- [x] Implement `app.base_path` support
- [x] Add middleware tests
- [x] Test execution order
- [x] Test short-circuiting
- [x] Test base_path routing
- [x] Define path-scoped middleware matching approach (use_at wrapper middleware)
- [x] Implement `use_fn` (Hono-like middleware wrapper)
- [x] Define `Next` wrapper type
- [x] Implement `Next::run(c: Ctx) -> Result<FeuResponse, Error>` calling inner service
- [x] Implement `use_fn` accepting `Fn(Ctx, Next) -> Future<Result<FeuResponse, Error>>`
- [x] Ensure user does not see Tower types in `use_fn` API
- [x] Ensure middleware supports before/after and short-circuit
- [x] Ensure ordering is deterministic (documented)
- [x] Implement `not_found` handler support
- [x] Implement `app.not_found(handler)` storing a fallback handler
- [x] Ensure not_found runs after router miss (after middleware or within pipeline; define)
- [x] Implement `on_error` handler support
- [x] Implement `app.on_error(handler)` storing global error handler
- [x] Define error propagation policy through middleware
- [x] Define default error response when on_error missing
- [x] Add Env/Runtime hooks framework
- [x] Introduce `App<Env, State>` generics (or equivalent)
- [x] Add `RuntimeCtx` trait
- [x] Add no-op `RuntimeCtx` implementation
- [x] Provide `Ctx::env()` and `Ctx::runtime()`
- [x] Update adapters entrypoint signature to pass env+runtime
- [x] Add middleware engine tests
- [x] Test before header injection affects final response (via pending or response mutation)
- [x] Test after response mutation (e.g., add header on returned response)
- [x] Test short-circuit returns early without calling next
- [x] Test on_error catches handler error
- [x] Test not_found handler returns custom response
- [x] Confirm `use_fn` does not leak internal types (public API check via docs)

---

## Phase 4 — Extractors + JSON + Cookie + Streaming (DX to Practical Level)

- [ ] Define extractor framework in core (recommended)
- [ ] Define `FromRequest` trait (async extraction)
- [x] Define structured extraction errors (400 vs 422 policy)
- [x] Implement basic extractors (no features)
- [x] Implement `Query<T>` extractor (serde_urlencoded or equivalent)
- [x] Implement `Path<T>` extractor (deserialize from params)
- [x] Implement `Header<T>` extractor (simple typed header parsing - `HeaderMap` implemented)
- [x] Implement `Bytes` extractor (collect full body)
- [x] Implement `Text` extractor (UTF-8)
- [x] Extractors (define Trait and impls)
  - [x] `FromRequest` trait (async)
  - [x] `Bytes` / `String`
  - [x] `Query` (serde)
  - [x] `Path` (serde)
  - [x] `Json` (serde, feature-gated `json`)
  - [x] `Cookie` (feature-gated `cookie`)
- [x] JSON
  - [x] `json` feature flag in `feu-core`
  - [x] `Ctx::json` helper
- [x] Cookie
  - [x] `cookie` feature flag
  - [x] `FeuResponse` helper (`with_cookie`)
  - [x] `Ctx` helper (`set_cookie`)
- [x] Streaming
  - [x] `streaming` feature flag
  - [x] `FeuBody::Stream` variant
  - [x] Plumbing for stream response (BoxStream or HttpBody-like adapter layer)
- [x] Define mutation constraints for streaming responses (Documented in `Ctx`)
- [x] Document: response cannot be replaced after streaming starts
- [x] Add tests for JSON extract/response
- [x] Add tests for cookie set/remove
- [x] Add tests for streaming plumbing where possible (at least type-level)
- [x] Ensure feature matrix builds
- [x] Ensure minimal core builds without json/cookie/streaming enabled
- [x] Ensure `--all-features` builds and tests pass

---

## Phase 5 — Built-in Middleware Set (Batteries Included) + Docs

- [x] Implement `feu-middleware` crate structure
- [x] Provide a common middleware config pattern (builder structs)
- [x] Implement ops/observability middleware first
- [x] Implement `logger` middleware (method/path/status/latency)
- [x] Implement `request_id` middleware (generate/propagate + inject into Ctx)
- [ ] Implement `timing` middleware (Server-Timing headers)
- [x] Implement `timeout` middleware (cancel/timeout response)
- [x] Implement security middleware
- [x] Implement `cors` middleware (preflight + headers)
- [x] Implement `secure_headers` middleware (standard security headers)
- [ ] Implement `csrf` middleware (minimal viable strategy)
- [ ] Implement `ip_restriction` middleware (allow/deny CIDR)
- [x] Implement perf/caching middleware
- [x] Implement `etag` middleware (If-None-Match -> 304)
- [ ] Implement `cache` middleware (Cache-Control policies)
- [ ] Add `compress` feature
- [ ] Implement `compress` middleware (gzip first; br/deflate optional)
- [ ] Implement `body_limit` middleware (reject oversize)
- [ ] Implement compatibility middleware
- [ ] Implement `method_override` middleware
- [ ] Implement `trailing_slash` middleware
- [ ] Implement DX/presentation middleware
- [ ] Implement `pretty_json` middleware (dev formatting policy)
- [ ] Implement `language` middleware (locale negotiation)
- [ ] Implement `context_storage` middleware/util (task-local access) + document caveats
- [ ] Implement `combine` helper (compose multiple middleware)
- [ ] Implement `jsx_renderer` hook (template adapter placeholder)
- [ ] Implement auth middleware
- [ ] Implement `auth_basic`
- [ ] Implement `auth_bearer`
- [ ] Add `jwt` feature
- [ ] Implement `jwt` verification middleware
- [ ] Add `jwk` feature
- [ ] Implement `jwk` fetching/caching for jwt
- [x] Middleware tests
- [x] Test logger runs and does not break response
- [x] Test request_id sets header and is accessible from Ctx extensions
- [x] Test cors preflight behavior
- [x] Test secure_headers adds expected headers
- [x] Test timeout returns timeout response
- [x] Test etag returns 304 when matched
- [ ] Test compress modifies response encoding (feature)
- [ ] Test body_limit rejects big payloads
- [ ] Middleware docs (required)
- [ ] Write `docs/middleware.md` with one-line description per middleware
- [ ] Add config options + minimal examples per middleware
- [ ] Document ordering notes (e.g., compress after etag or vice versa policy)
- [ ] Ensure docs stay consistent with exported APIs

---

## Phase 6 — Adapters (Hyper First, Then Lambda & Workers)

- [ ] Hyper adapter (primary)
- [ ] Implement `serve(app, addr)` function
- [ ] Convert Hyper request -> `http::Request<FeuBody>`
- [ ] Convert `FeuResponse`/`http::Response<FeuBody>` -> Hyper response
- [ ] Support streaming responses when `streaming` enabled
- [ ] Provide peer address info to `RuntimeCtx` (for ip_restriction)
- [ ] Hyper integration tests
- [ ] Add example `examples/hello-hyper`
- [ ] Add example `examples/restful-api` (json/query/path)
- [ ] Lambda adapter
- [ ] Implement event -> canonical request conversion
- [ ] Implement canonical response -> lambda response conversion
- [ ] Provide env/runtime mappings
- [ ] Add example `examples/hello-lambda`
- [ ] Workers adapter (WASM)
- [ ] Implement Request/Response conversion
- [ ] Implement `RuntimeCtx::wait_until` using Workers execution context
- [ ] Handle wasm-specific bounds/feature gating
- [ ] Add example `examples/hello-workers`
- [ ] Confirm the same App runs across adapters (portable core)

---

## Phase 7 — OpenAPI (After Adapters)

- [ ] Add `openapi` feature flag
- [ ] Create a shared route metadata IR (internal representation)
- [ ] Define `OperationMeta` (summary/description/tags/operation_id)
- [ ] Define `ParamMeta` (path/query/header/cookie)
- [ ] Define `RequestBodyMeta` (content-types + schema)
- [ ] Define `ResponseMeta` (status -> schema/content-type)
- [ ] Define `SecurityMeta` (basic/bearer/jwt)
- [ ] Ensure IR can be collected at route registration time
- [ ] Add API to attach OpenAPI metadata to routes
- [ ] Provide `route.openapi(|op| op.summary(...).tag(...))`-style builder (conceptual)
- [ ] Add `schema` feature for JSON Schema derivation
- [ ] Integrate schema derivation (schemars or equivalent)
- [ ] Map Rust types -> JSON Schema -> OpenAPI schema objects
- [ ] Handle common Rust patterns (Option/enums/newtypes/Vec/map)
- [ ] Build OpenAPI document generator
- [ ] Choose OpenAPI version support (3.0.x or 3.1) and document it
- [ ] Generate `paths` from router IR
- [ ] Generate `components/schemas` with stable naming
- [ ] Add stable output ordering to minimize diffs
- [ ] Serve `/openapi.json` via helper route or middleware
- [ ] Add optional Swagger UI/Redoc (feature-gated)
- [ ] OpenAPI tests
- [ ] Add golden tests for generated spec JSON
- [ ] Test all routes appear in spec
- [ ] Test params are correctly typed and placed
- [ ] Test request bodies/responses are correctly generated
- [ ] Test security schemes appear correctly
- [ ] OpenAPI docs
- [ ] Write `docs/openapi.md`
- [ ] Add example `examples/openapi-rest`

---

## Phase 8 — CLI (Dev / Codegen / Scaffolding)

- [ ] Implement `feu-cli` skeleton with subcommands
- [ ] Implement `feu dev` (hyper runner)
- [ ] Add options: host/port/log-level
- [ ] Add optional `--watch` mode (can be later)
- [ ] Implement `feu codegen` command (foundation)
- [ ] Add `--out` flag (default to `target/feu/`)
- [ ] Add `--openapi` output mode (write openapi.json)
- [ ] Add `--rpc` output mode (reserved for Phase 9)
- [ ] Add example integration docs in `docs/` or README
- [ ] Implement `feu new` scaffolding (optional)
- [ ] Template: hyper hello + optional features
- [ ] Ensure CLI is tested (basic command parsing + smoke tests)

---

## Phase 9 — RPC (Rust ↔ TypeScript Types + Client Generation)

- [ ] Add `rpc` feature flag
- [ ] Decide whether RPC reuses OpenAPI IR or has a parallel IR
- [ ] If shared: ensure OpenAPI IR captures all RPC needs (route structure + IO)
- [ ] Capture handler IO type metadata (request/response types) during route registration
- [ ] Define TS type mapping rules
- [ ] Map primitives, Option, Vec, maps
- [ ] Map enums (tagged/untagged) into TS unions
- [ ] Map newtypes
- [ ] Generate TS types output (d.ts or .ts)
- [ ] Ensure stable ordering and deterministic output
- [ ] Generate TS client
- [ ] Fetch wrapper with base URL config
- [ ] Support headers/auth hooks
- [ ] Typed path/query/body composition
- [ ] Typed response decoding and error mapping
- [ ] Integrate with `feu-cli codegen --rpc`
- [ ] Add example `examples/rpc-monorepo`
- [ ] Write `docs/rpc.md`
- [ ] Add tests (golden tests for TS output + small runtime smoke test)

---

## Phase 10 — Documentation, Examples, Benchmarks, Release Readiness

- [ ] Keep README aligned with real API names and signatures
- [ ] Write `docs/architecture.md` (flow of App→MW→Router→Handler→Response)
- [ ] Write `docs/adapters.md` (hyper/lambda/workers)
- [ ] Expand `docs/middleware.md` with configuration and ordering notes
- [ ] Ensure examples cover core features
- [ ] Add `examples/middleware-stack` (logger/cors/timeout/etag/compress)
- [ ] Add `examples/restful-api` (Json/Query/Path)
- [ ] Add `examples/openapi-rest` (OpenAPI spec + UI)
- [ ] Add `examples/rpc-monorepo` (TS type/client usage)
- [ ] Add benchmarks
- [ ] Add `benches/router.rs` (route count scaling, static vs params vs wildcards)
- [ ] Add `benches/middleware_chain.rs` (N layer overhead)
- [ ] Add benches for feature impacts (json/compress/openapi)
- [ ] Establish release policy
- [ ] Define semver policy for 0.x
- [ ] Define MSRV and document it
- [ ] Publish feature matrix (default/min/all-features)
- [ ] Add crate-level docs (`//!`) to all public modules
- [ ] Add security policy (optional)
- [ ] Run final audit: minimal build size and dependency review
- [ ] Cut first alpha release tag

---
