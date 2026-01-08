#[cfg(test)]
mod unit_tests {
    use crate::app::App;
    use crate::ctx::Ctx;
    use crate::rt::NoOpRuntimeCtx;
    use crate::types::FeuBody;
    use http::{Request, StatusCode};
    use std::sync::Arc;

    // --- Ctx Tests ---

    #[test]
    fn test_ctx_status_header_text_consumption() {
        // Setup
        let req = Request::new(FeuBody::Empty);
        let mut ctx = Ctx::new(req, (), Arc::new(NoOpRuntimeCtx), vec![]);

        // A-plan: Configure pending state
        ctx.status(StatusCode::CREATED).header("x-foo", "bar");

        // Action: Consume via text()
        let res = ctx.text("hello");

        // Asset: Status is CREATED
        assert_eq!(res.0.status(), StatusCode::CREATED);

        // Assert: Header is present
        assert_eq!(res.0.headers().get("x-foo").unwrap(), "bar");

        // Assert: Content-Type is text/plain
        assert_eq!(
            res.0.headers().get("content-type").unwrap(),
            "text/plain; charset=utf-8"
        );

        // Assert: Body is "hello"
        if let FeuBody::Text(s) = res.0.body() {
            assert_eq!(s, "hello");
        } else {
            panic!("Expected Text body");
        }
    }

    #[test]
    fn test_ctx_redirect() {
        // Default redirect
        let req = Request::new(FeuBody::Empty);
        let ctx = Ctx::new(req, (), Arc::new(NoOpRuntimeCtx), vec![]);
        let res = ctx.redirect("/home");
        assert_eq!(res.0.status(), StatusCode::FOUND); // 302
        assert_eq!(res.0.headers().get("location").unwrap(), "/home");

        // Custom status redirect
        let req = Request::new(FeuBody::Empty);
        let mut ctx = Ctx::new(req, (), Arc::new(NoOpRuntimeCtx), vec![]);
        ctx.status(StatusCode::MOVED_PERMANENTLY);
        let res = ctx.redirect("/gone");

        assert_eq!(res.0.status(), StatusCode::MOVED_PERMANENTLY);
        assert_eq!(res.0.headers().get("location").unwrap(), "/gone");
    }

    // --- App Tests ---

    #[tokio::test]
    async fn test_app_routing() {
        let mut app = App::new();
        app.get("/", |c: Ctx| async move { Ok(c.text("root")) })
            .get("/foo", |mut c: Ctx| async move {
                c.status(StatusCode::ACCEPTED);
                Ok(c.text("foo"))
            });

        // Test Match "/"
        let req = Request::builder().uri("/").body(FeuBody::Empty).unwrap();
        let res = app.handle(req, ()).await.unwrap();
        assert_eq!(res.0.status(), StatusCode::OK);
        if let FeuBody::Text(s) = res.0.body() {
            assert_eq!(s, "root");
        }

        // Test Match "/foo"
        let req = Request::builder().uri("/foo").body(FeuBody::Empty).unwrap();
        let res = app.handle(req, ()).await.unwrap();
        assert_eq!(res.0.status(), StatusCode::ACCEPTED);

        // Test 404
        let req = Request::builder().uri("/bar").body(FeuBody::Empty).unwrap();
        let res = app.handle(req, ()).await.unwrap();
        assert_eq!(res.0.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_valid_params() {
        let mut app = App::new();
        app.get("/user/:id", |c: Ctx| async move {
            let id = c.param("id").unwrap().to_string();
            Ok(c.text(format!("user {}", id)))
        });

        let req = Request::builder()
            .uri("/user/123")
            .body(FeuBody::Empty)
            .unwrap();
        let res = app.handle(req, ()).await.unwrap();
        assert_eq!(res.0.status(), StatusCode::OK);
        if let FeuBody::Text(s) = res.0.body() {
            assert_eq!(s, "user 123");
        } else {
            panic!("Expected text");
        }
    }
}
