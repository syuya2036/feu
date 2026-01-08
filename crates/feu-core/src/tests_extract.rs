#[cfg(test)]
mod tests {
    use crate::app::App;
    use crate::ctx::Ctx;
    use crate::extract::{Path, Query};
    use crate::types::FeuBody;
    use bytes::Bytes;
    use http::Request;
    use serde::Deserialize;

    #[tokio::test]
    async fn test_extract_bytes() {
        let mut app = App::new();
        app.post("/bytes", |mut c: Ctx| async move {
            let b: Bytes = c.extract().await.unwrap();
            Ok(c.body(b)) // echo
        });

        let req = Request::builder()
            .method("POST")
            .uri("/bytes")
            .body(FeuBody::Bytes(bytes::Bytes::from("hello")))
            .unwrap();
        let res = app.handle(req, ()).await.unwrap();
        if let FeuBody::Bytes(b) = res.0.body() {
            assert_eq!(b, "hello");
        } else {
            panic!("Expected bytes");
        }
    }

    #[derive(Deserialize)]
    struct MyQuery {
        q: String,
        page: i32,
    }

    #[tokio::test]
    async fn test_extract_query() {
        let mut app = App::new();
        app.get("/search", |mut c: Ctx| async move {
            let Query(q) = c.extract::<Query<MyQuery>>().await.unwrap();
            Ok(c.text(format!("{}:{}", q.q, q.page)))
        });

        let req = Request::builder()
            .uri("/search?q=foo&page=1")
            .body(FeuBody::Empty)
            .unwrap();
        let res = app.handle(req, ()).await.unwrap();
        if let FeuBody::Text(s) = res.0.body() {
            assert_eq!(s, "foo:1");
        } else {
            panic!("Expected text");
        }
    }

    #[tokio::test]
    async fn test_extract_path() {
        let mut app = App::new();
        app.get("/users/:id/posts/:post_id", |mut c: Ctx| async move {
            #[derive(Deserialize)]
            struct Params {
                id: String,
                post_id: i32,
            }
            let Path(p) = c.extract::<Path<Params>>().await.unwrap();
            Ok(c.text(format!("user {} post {}", p.id, p.post_id)))
        });

        let req = Request::builder()
            .uri("/users/alice/posts/42")
            .body(FeuBody::Empty)
            .unwrap();
        let res = app.handle(req, ()).await.unwrap();
        if let FeuBody::Text(s) = res.0.body() {
            assert_eq!(s, "user alice post 42");
        } else {
            panic!("Expected text");
        }
    }

    #[cfg(feature = "json")]
    #[tokio::test]
    async fn test_extract_json() {
        use crate::extract::Json;
        use serde::{Deserialize, Serialize};

        #[derive(Serialize, Deserialize, Debug, PartialEq)]
        struct Payload {
            name: String,
        }

        let mut app = App::new();
        app.post("/json", |mut c: Ctx| async move {
            let Json(p) = c.extract::<Json<Payload>>().await.unwrap();
            // Echo back using c.json
            c.json(p)
        });

        let req = Request::builder()
            .method("POST")
            .uri("/json")
            .header("content-type", "application/json")
            .body(FeuBody::Bytes(bytes::Bytes::from(r#"{"name":"feu"}"#)))
            .unwrap();

        let res = app.handle(req, ()).await.unwrap();
        assert_eq!(
            res.0.headers().get("content-type").unwrap(),
            "application/json"
        );

        if let FeuBody::Text(s) = res.0.body() {
            assert_eq!(s, r#"{"name":"feu"}"#);
        } else {
            panic!("Expected text (json string)");
        }
    }
}

#[cfg(feature = "cookie")]
#[cfg(test)]
mod tests_cookie {
    use crate::app::App;
    use crate::ctx::Ctx;
    use crate::types::FeuBody;
    use bytes::Bytes;
    use cookie::Cookie;
    use http::Request;

    #[tokio::test]
    async fn test_cookie() {
        let mut app = App::new();
        app.get("/cookie", |mut c: Ctx| async move {
            c.set_cookie(Cookie::new("foo", "bar"));
            // Removal: set with max-age 0
            let removal = Cookie::build(("baz", "qux"))
                .max_age(cookie::time::Duration::ZERO)
                .build();
            c.set_cookie(removal);
            Ok(c.text("cookies"))
        });

        let req = Request::builder()
            .uri("/cookie")
            .body(FeuBody::Empty)
            .unwrap();
        let res = app.handle(req, ()).await.unwrap();
        let cookies: Vec<_> = res.0.headers().get_all("set-cookie").iter().collect();
        let found_foo = cookies
            .iter()
            .any(|v| v.to_str().unwrap().contains("foo=bar"));
        let found_baz_remove = cookies.iter().any(|v| {
            v.to_str().unwrap().contains("baz=qux") && v.to_str().unwrap().contains("Max-Age=0")
        });

        if !found_baz_remove {
            panic!("Cookie removal (baz) not found. Headers: {:?}", cookies);
        }
        assert!(found_foo, "Cookie foo=bar not found"); // Move this after panic to ensure we see error
    }

    #[tokio::test]
    async fn test_extract_headers() {
        let mut app = App::new();
        app.get("/headers", |mut c: Ctx| async move {
            let headers: http::HeaderMap = c.extract().await.unwrap();
            let val = headers.get("x-foo").unwrap().to_str().unwrap();
            Ok(c.text(val.to_string()))
        });

        let req = Request::builder()
            .uri("/headers")
            .header("x-foo", "bar")
            .body(FeuBody::Empty)
            .unwrap();
        let res = app.handle(req, ()).await.unwrap();
        let b = res.0.into_body();
        // Simple body check (assuming Text extraction works or manual check)
        // We need to extract body from FeuBody to check content.
        let body_bytes = match b {
            FeuBody::Text(s) => Bytes::from(s.into_bytes()),
            _ => panic!("Expected text body"),
        };
        assert_eq!(body_bytes, Bytes::from("bar"));
    }
}
// Intentional blank to separate
