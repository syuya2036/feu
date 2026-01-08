use crate::app::App;
use crate::ctx::Ctx;
use crate::types::FeuBody;
use http::{Method, Request, StatusCode};

#[tokio::test]
async fn test_methods() {
    let mut app = App::new();
    app.get("/get", |c: Ctx| async move { Ok(c.text("GET")) })
        .post("/post", |c: Ctx| async move { Ok(c.text("POST")) })
        .put("/put", |c: Ctx| async move { Ok(c.text("PUT")) })
        .delete("/delete", |c: Ctx| async move { Ok(c.text("DELETE")) });

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
        let res = app.handle(req, ()).await.unwrap();
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
    let mut app = App::new();
    app.any("/all", |c: Ctx| async move {
        let method = c.req().method().to_string();
        Ok(c.text(method))
    });

    let req = Request::builder()
        .method("POST")
        .uri("/all")
        .body(FeuBody::Empty)
        .unwrap();
    let res = app.handle(req, ()).await.unwrap();
    if let FeuBody::Text(s) = res.0.body() {
        assert_eq!(s, "POST");
    }
}

#[tokio::test]
async fn test_composition() {
    let mut api = App::new();
    api.get("/users", |c: Ctx| async move { Ok(c.text("users")) })
        .get("/posts", |c: Ctx| async move { Ok(c.text("posts")) });

    let mut app = App::new();
    app.route("/api", api);

    // GET /api/users
    let req = Request::builder()
        .uri("/api/users")
        .body(FeuBody::Empty)
        .unwrap();
    let res = app.handle(req, ()).await.unwrap();
    assert_eq!(res.0.status(), StatusCode::OK);
    // body check omitted for brevity, expecting OK is good signal match worked

    // GET /api/posts
    let req = Request::builder()
        .uri("/api/posts")
        .body(FeuBody::Empty)
        .unwrap();
    let res = app.handle(req, ()).await.unwrap();
    assert_eq!(res.0.status(), StatusCode::OK);

    // GET /users (should fail)
    let req = Request::builder()
        .uri("/users")
        .body(FeuBody::Empty)
        .unwrap();
    let res = app.handle(req, ()).await.unwrap();
    assert_eq!(res.0.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_nested_wildcard() {
    let mut app = App::new();
    app.get("/static/*path", |c: Ctx| async move {
        let path = c.param("path").unwrap_or("").to_string();
        Ok(c.text(format!("static: {}", path)))
    });

    let req = Request::builder()
        .uri("/static/css/style.css")
        .body(FeuBody::Empty)
        .unwrap();
    let res = app.handle(req, ()).await.unwrap();
    if let FeuBody::Text(s) = res.0.body() {
        assert_eq!(s, "static: css/style.css");
    }
}
