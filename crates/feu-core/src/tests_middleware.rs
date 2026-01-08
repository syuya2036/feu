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
            let headers = res.0.headers_mut();
            headers.insert("x-logger-after", "1".parse().unwrap());
            Ok(res)
        })
    }
}

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
    let h = res.0.headers();
    assert!(h.contains_key("x-logger-before"));
    assert!(h.contains_key("x-mw2-before"));
    assert!(h.contains_key("x-logger-after"));
    assert!(h.contains_key("x-mw2-after"));
}

#[tokio::test]
async fn test_middleware_short_circuit() {
    let app = App::new()
        .use_mw(|_c: Ctx, _next: Next| async move { Ok(FeuResponse::text("Short Circuit")) })
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

    let req = Request::builder()
        .uri("/api/v1/users")
        .body(FeuBody::Empty)
        .unwrap();
    let res = app.handle(req).await.unwrap();
    assert_eq!(res.0.status(), StatusCode::OK);

    let req = Request::builder()
        .uri("/users")
        .body(FeuBody::Empty)
        .unwrap();
    let res = app.handle(req).await.unwrap();
    assert_eq!(res.0.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_not_found_handler() {
    let app = App::new().not_found(|mut c: Ctx| async move {
        Ok(c.text("Custom 404").with_status(StatusCode::NOT_FOUND))
    });

    let req = Request::builder()
        .uri("/nothing")
        .body(FeuBody::Empty)
        .unwrap();
    let res = app.handle(req).await.unwrap();

    assert_eq!(res.0.status(), StatusCode::NOT_FOUND);
    if let FeuBody::Text(s) = res.0.body() {
        assert_eq!(s, "Custom 404");
    } else {
        panic!("Expected text");
    }
}

#[tokio::test]
async fn test_use_at() {
    let app = App::new()
        .use_at("/admin", Logger)
        .get(
            "/admin/dash",
            |mut c: Ctx| async move { Ok(c.text("dash")) },
        )
        .get(
            "/public/home",
            |mut c: Ctx| async move { Ok(c.text("home")) },
        );

    let req = Request::builder()
        .uri("/admin/dash")
        .body(FeuBody::Empty)
        .unwrap();
    let res = app.handle(req).await.unwrap();
    let h = res.0.headers();
    assert!(h.contains_key("x-logger-before"));

    let req = Request::builder()
        .uri("/public/home")
        .body(FeuBody::Empty)
        .unwrap();
    let res = app.handle(req).await.unwrap();
    let h = res.0.headers();
    assert!(!h.contains_key("x-logger-before"));
}
