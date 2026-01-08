#[cfg(test)]
mod unit_tests {
    use crate::app::App;
    use crate::ctx::Ctx;
    use crate::types::FeuBody;
    use http::{Request, StatusCode};

    // --- Ctx Tests ---

    #[test]
    fn test_ctx_status_header_text_consumption() {
        // Setup
        let req = Request::new(FeuBody::Empty);
        let mut ctx = Ctx::new(req);

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
        // (For simple test we assume body conversion works, checking body content is slightly harder with FeuBody enum private fields if we don't expose it, but we can match)
        if let FeuBody::Text(s) = res.0.body() {
            assert_eq!(s, "hello");
        } else {
            panic!("Expected Text body");
        }

        // Verify Consumption: Pending state should be empty now
        let (status, headers) = ctx.take_pending();
        assert!(status.is_none());
        assert!(headers.is_empty());
    }

    #[test]
    fn test_ctx_redirect() {
        let req = Request::new(FeuBody::Empty);
        let mut ctx = Ctx::new(req);

        // Default redirect
        let res = ctx.redirect("/home");
        assert_eq!(res.0.status(), StatusCode::FOUND); // 302
        assert_eq!(res.0.headers().get("location").unwrap(), "/home");

        // Custom status redirect
        let mut ctx = Ctx::new(Request::new(FeuBody::Empty));
        ctx.status(StatusCode::MOVED_PERMANENTLY).redirect("/gone"); // 301

        // wait, we need to capture the response. The builder pattern `ctx.status(...)` returns `&mut Ctx`.
        // `redirect` returns `FeuResponse`.

        let mut ctx = Ctx::new(Request::new(FeuBody::Empty));
        ctx.status(StatusCode::MOVED_PERMANENTLY);
        let res = ctx.redirect("/gone");

        assert_eq!(res.0.status(), StatusCode::MOVED_PERMANENTLY);
        assert_eq!(res.0.headers().get("location").unwrap(), "/gone");
    }

    // --- App Tests ---

    #[tokio::test]
    async fn test_app_routing() {
        let app = App::new()
            .get("/", |mut c: Ctx| async move { Ok(c.text("root")) })
            .get("/foo", |mut c: Ctx| async move {
                c.status(StatusCode::ACCEPTED);
                Ok(c.text("foo"))
            });

        // Test Match "/"
        let req = Request::builder().uri("/").body(FeuBody::Empty).unwrap();
        let res = app.handle(req).await.unwrap();
        assert_eq!(res.0.status(), StatusCode::OK);
        if let FeuBody::Text(s) = res.0.body() {
            assert_eq!(s, "root");
        }

        // Test Match "/foo"
        let req = Request::builder().uri("/foo").body(FeuBody::Empty).unwrap();
        let res = app.handle(req).await.unwrap();
        assert_eq!(res.0.status(), StatusCode::ACCEPTED);

        // Test 404
        let req = Request::builder().uri("/bar").body(FeuBody::Empty).unwrap();
        let res = app.handle(req).await.unwrap();
        assert_eq!(res.0.status(), StatusCode::NOT_FOUND);
    }
}
