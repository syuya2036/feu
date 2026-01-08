use base64::prelude::*;
use feu_core::app::App;
use feu_core::ctx::Ctx;
use feu_middleware::{BasicAuth, BearerAuth, MethodOverride, TrailingSlash};
use http::{Method, StatusCode};

#[tokio::test]
async fn test_method_override() {
    let mut app = App::new();
    app.use_mw(MethodOverride::new());
    app.delete("/item", |c: Ctx| async move { Ok(c.text("deleted")) });

    // 1. Normal POST (Simulating form) with override
    let req = http::Request::builder()
        .method("POST")
        .uri("/item")
        .header("x-http-method-override", "DELETE")
        .body(Default::default())
        .unwrap();
    let res = app.handle(req, ()).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    match res.body() {
        feu_core::types::FeuBody::Text(s) => assert_eq!(s, "deleted"),
        _ => panic!("Expected text"),
    }
}

#[tokio::test]
async fn test_trailing_slash() {
    let mut app = App::new();
    app.use_mw(TrailingSlash::new());
    app.get("/foo", |c: Ctx| async move { Ok(c.text("foo")) });

    // 1. With slash -> Redirect
    let req = http::Request::builder()
        .uri("/foo/")
        .body(Default::default())
        .unwrap();
    let res = app.handle(req, ()).await.unwrap();
    assert_eq!(res.status(), StatusCode::PERMANENT_REDIRECT);
    assert_eq!(res.headers().get("location").unwrap(), "/foo");

    // 2. Without slash -> OK
    let req = http::Request::builder()
        .uri("/foo")
        .body(Default::default())
        .unwrap();
    let res = app.handle(req, ()).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    // 3. Root -> OK (no strip)
    let req = http::Request::builder()
        .uri("/")
        .body(Default::default())
        .unwrap();
    // Handler for root? I didn't register one. But middleware check happens before 404.
    // Wait, middleware runs, then next. If 404, it returns 404.
    // TrailingSlash ignores root.
    let res = app.handle(req, ()).await.unwrap();
    assert_eq!(res.status(), StatusCode::NOT_FOUND); // As expected
}

#[tokio::test]
async fn test_auth_basic() {
    let mut app = App::new();
    app.use_mw(BasicAuth::new(|u, p| u == "admin" && p == "secret"));
    app.get("/protected", |c: Ctx| async move { Ok(c.text("welcome")) });

    // 1. No header -> 401
    let req = http::Request::builder()
        .uri("/protected")
        .body(Default::default())
        .unwrap();
    let res = app.handle(req, ()).await.unwrap();
    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
    assert!(res
        .headers()
        .get("www-authenticate")
        .unwrap()
        .to_str()
        .unwrap()
        .contains("Basic"));

    // 2. Invalid creds -> 401
    let creds = BASE64_STANDARD.encode("user:bad");
    let req = http::Request::builder()
        .uri("/protected")
        .header("authorization", format!("Basic {}", creds))
        .body(Default::default())
        .unwrap();
    let res = app.handle(req, ()).await.unwrap();
    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);

    // 3. Valid creds -> 200
    let creds = BASE64_STANDARD.encode("admin:secret");
    let req = http::Request::builder()
        .uri("/protected")
        .header("authorization", format!("Basic {}", creds))
        .body(Default::default())
        .unwrap();
    let res = app.handle(req, ()).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_auth_bearer() {
    let mut app = App::new();
    app.use_mw(BearerAuth::new(|t| t == "my-token"));
    app.get("/api", |c: Ctx| async move { Ok(c.text("data")) });

    // 1. No header -> 401
    let req = http::Request::builder()
        .uri("/api")
        .body(Default::default())
        .unwrap();
    let res = app.handle(req, ()).await.unwrap();
    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);

    // 2. Valid token -> 200
    let req = http::Request::builder()
        .uri("/api")
        .header("authorization", "Bearer my-token")
        .body(Default::default())
        .unwrap();
    let res = app.handle(req, ()).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
}
