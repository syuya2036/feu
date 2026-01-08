use feu_core::app::App;
use feu_core::ctx::Ctx;
use feu_core::types::FeuResponse;
use feu_middleware::BodyLimit;
use http::StatusCode;

#[tokio::test]
async fn test_body_limit() {
    let mut app = App::new();
    app.use_mw(BodyLimit::new(10)); // 10 bytes limit
    app.post("/", |mut c: Ctx| async move {
        // Must consume body to trigger limit
        let bytes = c.extract::<bytes::Bytes>().await.unwrap();
        Ok(c.text(format!("size: {}", bytes.len())))
    });

    // 1. Within limit
    let req = http::Request::builder()
        .method("POST")
        .uri("/")
        .body("123456789".into())
        .unwrap();
    let res = app.handle(req, ()).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    // 2. Over limit (Content-Length known)
    let req = http::Request::builder()
        .method("POST")
        .uri("/")
        .body("12345678901".into())
        .unwrap();
    let res = app.handle(req, ()).await.unwrap();
    assert_eq!(res.status(), StatusCode::PAYLOAD_TOO_LARGE);
}

#[cfg(feature = "compress")]
#[tokio::test]
async fn test_compress() {
    use feu_middleware::Compress;
    let mut app = App::new();
    app.use_mw(Compress::new());
    app.get(
        "/",
        |c: Ctx| async move { Ok(c.text("hello world".repeat(10))) },
    );

    // Request with accept-encoding
    let req = http::Request::builder()
        .uri("/")
        .header("accept-encoding", "gzip")
        .body(Default::default())
        .unwrap();
    let res = app.handle(req, ()).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    assert_eq!(
        res.headers()
            .get("content-encoding")
            .unwrap()
            .to_str()
            .unwrap(),
        "gzip"
    );
    // Should verify body is compressed (not plain text)
    // Decompress via client logic or just check length/header
}

#[cfg(feature = "json")]
#[tokio::test]
async fn test_pretty_json() {
    use feu_middleware::PrettyJson;
    use serde_json::json;

    let mut app = App::new();
    app.use_mw(PrettyJson::new());
    app.get("/", |c: Ctx| async move {
        c.json(json!({"foo": "bar", "baz": 1}))
    });

    // Normal
    let req = http::Request::builder()
        .uri("/")
        .body(Default::default())
        .unwrap();
    let res = app.handle(req, ()).await.unwrap();
    match res.body() {
        feu_core::types::FeuBody::Text(s) => assert!(!s.contains("\n")),
        _ => panic!("Expected text"),
    }

    // Pretty
    let req = http::Request::builder()
        .uri("/?pretty")
        .body(Default::default())
        .unwrap();
    let res = app.handle(req, ()).await.unwrap();
    match res.body() {
        feu_core::types::FeuBody::Text(s) => {
            assert!(s.contains("\n"));
            assert!(s.contains("  \"foo\": \"bar\""));
        }
        _ => panic!("Expected text"),
    }
}
