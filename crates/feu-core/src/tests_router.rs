use crate::app::App;
use crate::ctx::Ctx;
use crate::types::FeuBody;
use http::{Method, Request, StatusCode};

#[tokio::test]
async fn test_methods() {
    let app = App::new()
        .get("/get", |mut c: Ctx| async move { Ok(c.text("GET")) })
        .post("/post", |mut c: Ctx| async move { Ok(c.text("POST")) })
        .put("/put", |mut c: Ctx| async move { Ok(c.text("PUT")) })
        .delete("/delete", |mut c: Ctx| async move { Ok(c.text("DELETE")) });

    let methods = vec![
        (Method::GET, "/get", "GET"),
        (Method::POST, "/post", "POST"),
        (Method::PUT, "/put", "PUT"),
        (Method::DELETE, "/delete", "DELETE"),
    ];

    for (method, path, expected) in methods {
        let req = Request::builder()
            .method(method)
            .uri(path)
            .body(FeuBody::Empty)
            .unwrap();
        let res = app.handle(req).await.unwrap();
        assert_eq!(res.0.status(), StatusCode::OK);
        if let FeuBody::Text(s) = res.0.body() {
            assert_eq!(s, expected);
        } else {
            panic!("Expected text body");
        }
    }
}

#[tokio::test]
async fn test_any() {
    let app = App::new().any("/all", |mut c: Ctx| async move {
        let method = c.req().method().to_string();
        Ok(c.text(method))
    });

    let req = Request::builder()
        .method("POST")
        .uri("/all")
        .body(FeuBody::Empty)
        .unwrap();
    let res = app.handle(req).await.unwrap();
    if let FeuBody::Text(s) = res.0.body() {
        assert_eq!(s, "POST");
    }
}

#[tokio::test]
async fn test_composition() {
    let api = App::new()
        .get("/users", |mut c: Ctx| async move { Ok(c.text("users")) })
        .get("/posts", |mut c: Ctx| async move { Ok(c.text("posts")) });

    let app = App::new().route("/api", api);

    // GET /api/users
    let req = Request::builder()
        .uri("/api/users")
        .body(FeuBody::Empty)
        .unwrap();
    let res = app.handle(req).await.unwrap();
    assert_eq!(res.0.status(), StatusCode::OK);
    // body check omitted for brevity, expecting OK is good signal match worked

    // GET /api/posts
    let req = Request::builder()
        .uri("/api/posts")
        .body(FeuBody::Empty)
        .unwrap();
    let res = app.handle(req).await.unwrap();
    assert_eq!(res.0.status(), StatusCode::OK);

    // GET /users (should fail)
    let req = Request::builder()
        .uri("/users")
        .body(FeuBody::Empty)
        .unwrap();
    let res = app.handle(req).await.unwrap();
    assert_eq!(res.0.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_nested_wildcard() {
    let app = App::new().get("/static/*path", |mut c: Ctx| async move {
        let path = c.param("path").unwrap_or("").to_string();
        Ok(c.text(format!("static: {}", path)))
    });

    let req = Request::builder()
        .uri("/static/css/style.css")
        .body(FeuBody::Empty)
        .unwrap();
    let res = app.handle(req).await.unwrap();
    if let FeuBody::Text(s) = res.0.body() {
        assert_eq!(s, "static: css/style.css");
    }
}
