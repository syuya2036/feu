use feu_core::app::App;
use feu_core::ctx::Ctx;
use feu_core::types::{FeuBody, FeuResponse};
use feu_middleware::{Cors, Etag, Logger, RequestId, SecureHeaders, Timeout};
use http::StatusCode;
use std::time::Duration;

#[tokio::test]
async fn test_request_id() {
    let mut app = App::new();
    app.use_mw(RequestId::new());
    app.get("/", |c: Ctx| async move { Ok(c.text("ok")) });

    // 1. Missing header -> Generated
    let req = http::Request::builder()
        .uri("/")
        .body(FeuBody::Empty)
        .unwrap();
    let res = app.handle(req, ()).await.unwrap();
    assert!(res.0.headers().contains_key("x-request-id"));

    // 2. Existing header -> Preserved
    let req = http::Request::builder()
        .uri("/")
        .header("x-request-id", "my-id-123")
        .body(FeuBody::Empty)
        .unwrap();
    let res = app.handle(req, ()).await.unwrap();
    let val = res.0.headers().get("x-request-id").unwrap();
    assert_eq!(val, "my-id-123");
}

#[tokio::test]
async fn test_cors() {
    let mut app = App::new();
    app.use_mw(Cors::permissive());
    app.get("/", |c: Ctx| async move { Ok(c.text("ok")) });

    // 1. OPTIONS preflight
    let req = http::Request::builder()
        .method("OPTIONS")
        .uri("/")
        .body(FeuBody::Empty)
        .unwrap();
    let res = app.handle(req, ()).await.unwrap();
    assert_eq!(res.status(), StatusCode::NO_CONTENT);
    assert!(
        res.headers()
            .get("access-control-allow-origin")
            .unwrap()
            .to_str()
            .unwrap()
            == "*"
    );

    // 2. GET request
    let req = http::Request::builder()
        .uri("/")
        .body(FeuBody::Empty)
        .unwrap();
    let res = app.handle(req, ()).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    assert!(
        res.headers()
            .get("access-control-allow-origin")
            .unwrap()
            .to_str()
            .unwrap()
            == "*"
    );
}

#[tokio::test]
async fn test_secure_headers() {
    let mut app = App::new();
    app.use_mw(SecureHeaders::new());
    app.get("/", |c: Ctx| async move { Ok(c.text("ok")) });

    let req = http::Request::builder()
        .uri("/")
        .body(FeuBody::Empty)
        .unwrap();
    let res = app.handle(req, ()).await.unwrap();
    assert!(res.headers().contains_key("x-frame-options"));
    assert!(res.headers().contains_key("strict-transport-security"));
}

#[tokio::test]
async fn test_timeout() {
    let mut app = App::new();
    // Short timeout
    app.use_mw(Timeout::new(Duration::from_millis(50)));

    // Slow handler
    app.get("/slow", |_c: Ctx| async move {
        tokio::time::sleep(Duration::from_millis(100)).await;
        Ok(FeuResponse::text("too slow"))
    });

    // Fast handler
    app.get("/fast", |c: Ctx| async move { Ok(c.text("fast")) });

    // 1. Timeout
    let req = http::Request::builder()
        .uri("/slow")
        .body(FeuBody::Empty)
        .unwrap();
    let res = app.handle(req, ()).await.unwrap();
    assert_eq!(res.status(), StatusCode::GATEWAY_TIMEOUT);

    // 2. Success
    let req = http::Request::builder()
        .uri("/fast")
        .body(FeuBody::Empty)
        .unwrap();
    let res = app.handle(req, ()).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_etag() {
    let mut app = App::new();
    app.use_mw(Etag::new());
    app.get("/", |c: Ctx| async move { Ok(c.text("content")) });

    // 1. Get ETag
    let req = http::Request::builder()
        .uri("/")
        .body(FeuBody::Empty)
        .unwrap();
    let res = app.handle(req, ()).await.unwrap();
    let etag = res
        .headers()
        .get("etag")
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();
    assert!(!etag.is_empty());

    // 2. If-None-Match matching
    let req = http::Request::builder()
        .uri("/")
        .header("if-none-match", &etag)
        .body(FeuBody::Empty)
        .unwrap();
    let res = app.handle(req, ()).await.unwrap();
    assert_eq!(res.status(), StatusCode::NOT_MODIFIED);
    match res.body() {
        FeuBody::Empty => {}
        _ => panic!("Body should be empty for 304"),
    }

    // 3. If-None-Match non-matching
    let req = http::Request::builder()
        .uri("/")
        .header("if-none-match", "\"dummy\"")
        .body(FeuBody::Empty)
        .unwrap();
    let res = app.handle(req, ()).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_logger() {
    // Logger just shouldn't crash
    let mut app = App::new();
    app.use_mw(Logger::new());
    app.get("/", |c: Ctx| async move { Ok(c.text("ok")) });

    let req = http::Request::builder()
        .uri("/")
        .body(FeuBody::Empty)
        .unwrap();
    let res = app.handle(req, ()).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
}
