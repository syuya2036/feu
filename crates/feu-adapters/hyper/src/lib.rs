use feu_core::app::App;
use feu_core::types::{FeuBody, FeuRequest};
use http_body_util::BodyExt;
use hyper::body::Incoming;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper_util::rt::TokioIo;
use std::convert::Infallible;
use std::sync::Arc;
use tokio::net::{TcpListener, ToSocketAddrs};

pub async fn serve<E, A>(app: App<E>, env: E, addr: A) -> feu_core::error::Result<()>
where
    E: Clone + Send + Sync + 'static,
    A: ToSocketAddrs,
{
    let listener = TcpListener::bind(addr)
        .await
        .map_err(|e| feu_core::error::Error::msg(e.to_string()))?;

    let app = Arc::new(app);

    loop {
        let (stream, _) = match listener.accept().await {
            Ok(s) => s,
            Err(e) => {
                eprintln!("Accept error: {}", e);
                continue;
            }
        };

        let io = TokioIo::new(stream);
        let app = app.clone();
        let env = env.clone();

        tokio::task::spawn(async move {
            if let Err(err) = http1::Builder::new()
                .serve_connection(
                    io,
                    service_fn(move |req: hyper::Request<Incoming>| {
                        let app = app.clone();
                        let env = env.clone();
                        async move {
                            let (parts, body) = req.into_parts();

                            let bytes = match body.collect().await {
                                Ok(collected) => collected.to_bytes(),
                                Err(e) => {
                                    return Ok::<_, Infallible>(
                                        http::Response::builder()
                                            .status(http::StatusCode::INTERNAL_SERVER_ERROR)
                                            .body(http_body_util::Full::new(bytes::Bytes::from(
                                                format!("Body error: {}", e),
                                            )))
                                            .unwrap(),
                                    )
                                }
                            };

                            let feu_req = FeuRequest::from_parts(parts, FeuBody::from(bytes));

                            match app.handle(feu_req, env).await {
                                Ok(res) => {
                                    let (parts, body) = res.into_inner().into_parts();
                                    let body_bytes = match body {
                                        FeuBody::Empty => bytes::Bytes::new(),
                                        FeuBody::Bytes(b) => b,
                                        FeuBody::Text(s) => bytes::Bytes::from(s),
                                        #[allow(unreachable_patterns)]
                                        _ => bytes::Bytes::from(
                                            "Unsupported body type in default adapter (stream?)",
                                        ),
                                    };

                                    Ok::<_, Infallible>(http::Response::from_parts(
                                        parts,
                                        http_body_util::Full::new(body_bytes),
                                    ))
                                }
                                Err(e) => Ok(http::Response::builder()
                                    .status(http::StatusCode::INTERNAL_SERVER_ERROR)
                                    .body(http_body_util::Full::new(bytes::Bytes::from(format!(
                                        "Internal Error: {:?}",
                                        e
                                    ))))
                                    .unwrap()),
                            }
                        }
                    }),
                )
                .await
            {
                eprintln!("Error serving connection: {:?}", err);
            }
        });
    }
}
