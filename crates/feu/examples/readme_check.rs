#[cfg(feature = "hyper")]
use feu::middleware::{cors, logger, Next};
#[cfg(feature = "hyper")]
use feu::prelude::*;
#[cfg(feature = "hyper")]
use http::header::HeaderName;
#[cfg(feature = "hyper")]
use http::StatusCode;

#[cfg(feature = "hyper")]
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut app = App::new();

    app.get("/", |c: Ctx| async move { Ok(c.text("Hello, feu!")) });

    app.get("/users/:id", |c: Ctx| async move {
        let val = c.param("id").unwrap_or("default").to_string();
        Ok(c.text(format!("id: {}", val)))
    });

    app.post("/posts", |mut c: Ctx| async move {
        c.status(StatusCode::CREATED);
        c.header("x-created", "1");
        Ok(c.text("created (json pending)"))
    });

    // Builder pattern usage (by ref)
    app.use_mw(logger::Logger::new());
    app.use_mw(cors::Cors::permissive());

    app.get("/healthz", |c: Ctx| async move { Ok(c.text("ok")) });

    // Hono-style middleware
    app.use_fn(|mut c: Ctx, next: Next| async move {
        c.header("x-before", "1");
        let mut res = next.run(c).await?;
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
