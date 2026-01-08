use feu::middleware::{cors, logger};
use feu::prelude::*;
use http::header::HeaderName; // Used for with_header

#[cfg(feature = "hyper")]
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut app = App::new();

    // Verify simple handler (no mut c needed for text() if it consumes self)
    // Note: If c.text() consumes self, we can use it on non-mut c.
    // However, if we need to call c.header() first (A-plan), we need mut c.
    app.get("/", |c: Ctx| async move { Ok(c.text("Hello, feu!")) });

    app.get("/users/:id", |c: Ctx| async move {
        // param returns Option<&str> currently.
        // We verify that we can access it.
        let val = c.param("id").unwrap_or("default").to_string();
        Ok(c.text(format!("id: {}", val)))
        // Note: README says `let id: String = c.param("id")?`.
        // This discrepancy exists. We accept it for now or update README later.
    });

    // Validating A-plan (requires mut c)
    app.post("/posts", |mut c: Ctx| async move {
        c.status(StatusCode::CREATED);
        c.header("x-created", "1");
        // c.json needs `json` feature and implementation (Phase 4)
        // Ok(c.json(serde_json::json!({ "ok": true })))
        Ok(c.text("created (json pending)"))
    });

    // Builder pattern usage (by ref)
    app.use_mw(logger::default());
    app.use_mw(cors::Cors::permissive());

    app.get("/healthz", |c: Ctx| async move { Ok(c.text("ok")) });

    // Hono-style middleware
    app.use_fn(|mut c: Ctx, next: feu::middleware::Next| async move {
        c.header("x-before", "1");
        let mut res = next.run(c).await?;
        // with_header requires HeaderName
        res = res.with_header(
            "x-after".parse::<HeaderName>().unwrap(),
            "1".parse().unwrap(),
        );
        Ok(res)
    });

    // Hyper serve
    feu::adapters::hyper::serve(app, "0.0.0.0:3000").await?;

    Ok(())
}

#[cfg(not(feature = "hyper"))]
fn main() {
    println!("This example requires the 'hyper' feature.");
}
