# Middleware

Feu comes with a suite of built-in middleware (`feu-middleware` crate) to handle common tasks.

## Observability

### Logger
Logs request method, path, status, and latency. Uses `tracing`.

```rust
use feu_middleware::Logger;
app.use_mw(Logger::new());
```

### RequestId
Generates or propagates a unique ID for each request. Available in response headers (`x-request-id`) and `Ctx`.

```rust
use feu_middleware::RequestId;
app.use_mw(RequestId::new());
```

### Timeout
Enforces a timeout on request processing. Returns 504 Gateway Timeout if exceeded.

```rust
use feu_middleware::Timeout;
use std::time::Duration;
app.use_mw(Timeout::new(Duration::from_secs(30)));
```

## Security

### Cors
Handles Cross-Origin Resource Sharing (CORS).

```rust
use feu_middleware::Cors;
app.use_mw(Cors::permissive());
// or Cors::new().allow_origin("...")
```

### SecureHeaders
Adds standard security headers (`X-Frame-Options`, `X-Content-Type-Options`, `HSTS`, etc.).

```rust
use feu_middleware::SecureHeaders;
app.use_mw(SecureHeaders::new());
```

### BasicAuth
Sync implementation of HTTP Basic Auth.

```rust
use feu_middleware::BasicAuth;
app.use_mw(BasicAuth::new(|user, pass| user == "admin" && pass == "secret"));
```

### BearerAuth
Sync implementation of HTTP Bearer Auth.

```rust
use feu_middleware::BearerAuth;
app.use_mw(BearerAuth::new(|token| token == "valid-token"));
```

## Performance & Caching

### Compress (Feature: `compress`)
Compresses responses using Gzip (Brotli/Deflate supported by feature flags).

```rust
use feu_middleware::Compress;
app.use_mw(Compress::new());
```

### Etag
Generates weak ETags for response bodies and handles `304 Not Modified`.

```rust
use feu_middleware::Etag;
app.use_mw(Etag::new());
```

### BodyLimit
Reject requests with payloads larger than the limit. Checks `Content-Length` and counts bytes.

```rust
use feu_middleware::BodyLimit;
app.use_mw(BodyLimit::new(1024 * 1024)); // 1MB
```

## Compatibility

### MethodOverride
Allows overriding HTTP method via `X-HTTP-Method-Override` header.

```rust
use feu_middleware::MethodOverride;
app.use_mw(MethodOverride::new());
```

### TrailingSlash
Redirects requests with trailing slash to non-trailing slash (or vice versa). Default: Strip & Redirect (Permanent).

```rust
use feu_middleware::TrailingSlash;
app.use_mw(TrailingSlash::new());
```

## Development / DX

### PrettyJson (Feature: `json`)
Pretty-prints JSON responses if query param `?pretty` is present.

```rust
use feu_middleware::PrettyJson;
app.use_mw(PrettyJson::new());
```
