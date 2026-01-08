use crate::app::App;
use crate::ctx::Ctx;
use crate::middleware::{BoxFuture, Middleware, Next};
use crate::types::{FeuBody, FeuResponse};
use http::Request;
use http::StatusCode;

struct Logger;

impl Middleware for Logger {
    fn handle(&self, mut ctx: Ctx, next: Next) -> BoxFuture<crate::error::Result<FeuResponse>> {
        Box::pin(async move {
            ctx.header("x-logger-before", "1");
            let mut res = next.run(ctx).await?;
            // We can't access ctx here readily as it was moved.
            // But we can modify response.
            let headers = res.0.headers_mut();
            headers.insert("x-logger-after", "1".parse().unwrap());
            Ok(res)
        })
    }
}

/*
// Closure middleware wrapper helper if needed, but we implemented Middleware for Fn
// Fn(Ctx, Next) -> Future
*/

#[tokio::test]
async fn test_middleware_execution_order() {
    let app = App::new()
        .use_mw(Logger)
        .use_mw(|mut c: Ctx, next: Next| async move {
            c.header("x-mw2-before", "1");
            let mut res = next.run(c).await?;
            res.0
                .headers_mut()
                .insert("x-mw2-after", "1".parse().unwrap());
            Ok(res)
        })
        .get("/test", |mut c: Ctx| async move { Ok(c.text("hello")) });

    let req = Request::builder()
        .uri("/test")
        .body(FeuBody::Empty)
        .unwrap();
    let res = app.handle(req).await.unwrap();

    assert_eq!(res.0.status(), StatusCode::OK);

    // Check headers
    // Note: Ctx headers set during "A-plan" (ctx.header(...)) are applied to response *inside* `c.text()`.
    // The middleware calls `next.run(ctx)`. `ctx` carries the pending headers.
    // The handler calls `c.text()`, which calls `c.apply_pending()`, flushing "x-logger-before" and "x-mw2-before" to response.
    // Then middleware returns `res` and adds "after" headers.

    let h = res.0.headers();
    assert!(h.contains_key("x-logger-before"));
    assert!(h.contains_key("x-mw2-before"));
    assert!(h.contains_key("x-logger-after"));
    assert!(h.contains_key("x-mw2-after"));
}

#[tokio::test]
async fn test_middleware_short_circuit() {
    let app = App::new()
        .use_mw(|_c: Ctx, _next: Next| async move {
            // Don't call next, return immediately
            Ok(FeuResponse::text("Short Circuit"))
        })
        .get("/test", |_| async {
            Ok(FeuResponse::text("Should not be reached"))
        });

    let req = Request::builder()
        .uri("/test")
        .body(FeuBody::Empty)
        .unwrap();
    let res = app.handle(req).await.unwrap();

    if let FeuBody::Text(s) = res.0.body() {
        assert_eq!(s, "Short Circuit");
    } else {
        panic!("Expected text");
    }
}

#[tokio::test]
async fn test_base_path() {
    let app = App::new()
        .base_path("/api/v1")
        .get("/users", |mut c: Ctx| async move { Ok(c.text("users")) });

    // GET /api/v1/users
    let req = Request::builder()
        .uri("/api/v1/users")
        .body(FeuBody::Empty)
        .unwrap();
    let res = app.handle(req).await.unwrap();
    assert_eq!(res.0.status(), StatusCode::OK);

    // GET /users -> 404
    let req = Request::builder()
        .uri("/users")
        .body(FeuBody::Empty)
        .unwrap();
    let res = app.handle(req).await.unwrap();
    assert_eq!(res.0.status(), StatusCode::NOT_FOUND);
}
