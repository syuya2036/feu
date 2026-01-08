use feu_core::ctx::Ctx;
use feu_core::middleware::{BoxFuture, Middleware, Next};
use feu_core::types::{FeuBody, FeuResponse};
use futures_util::stream::{self, StreamExt, TryStreamExt};
use http::header::{ACCEPT_ENCODING, CONTENT_ENCODING, CONTENT_LENGTH};
use std::io;
use tokio_util::io::{ReaderStream, StreamReader};

#[cfg(feature = "compress")]
#[derive(Clone, Copy, Debug)]
pub struct Compress;

#[cfg(feature = "compress")]
impl Compress {
    pub fn new() -> Self {
        Compress
    }
}

#[cfg(feature = "compress")]
impl Default for Compress {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "compress")]
impl Middleware for Compress {
    fn handle(&self, ctx: Ctx, next: Next) -> BoxFuture<feu_core::error::Result<FeuResponse>> {
        // Check Accept-Encoding
        let accept_encoding = ctx
            .req
            .headers()
            .get(ACCEPT_ENCODING)
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string())
            .unwrap_or_default();

        Box::pin(async move {
            let mut res = next.run(ctx).await?;

            // Simple negotiation: gzip > deflate (br optional but skipping for now to keep deps small? no I have brotli feature)
            // Let's implement gzip for now.
            if accept_encoding.contains("gzip") {
                // If Content-Encoding is already set, skip
                if res.headers().contains_key(CONTENT_ENCODING) {
                    return Ok(res);
                }

                // Compress
                let body = std::mem::replace(&mut *res.body_mut(), FeuBody::Empty);

                let stream = match body {
                    FeuBody::Empty => return Ok(res), // Don't compress empty
                    FeuBody::Bytes(b) => {
                        let s = stream::once(async move { Ok(b) });
                        Box::pin(s)
                            as futures_util::stream::BoxStream<
                                'static,
                                Result<bytes::Bytes, io::Error>,
                            >
                    }
                    FeuBody::Text(s) => {
                        let b = bytes::Bytes::from(s);
                        let s = stream::once(async move { Ok(b) });
                        Box::pin(s)
                            as futures_util::stream::BoxStream<
                                'static,
                                Result<bytes::Bytes, io::Error>,
                            >
                    }
                    #[cfg(feature = "streaming")]
                    FeuBody::Stream(s) => {
                        // Map error to io::Error
                        let s = s.map_err(|e| io::Error::new(io::ErrorKind::Other, e));
                        Box::pin(s)
                            as futures_util::stream::BoxStream<
                                'static,
                                Result<bytes::Bytes, io::Error>,
                            >
                    }
                    #[cfg(not(feature = "streaming"))]
                    _ => return Ok(res), // Should not happen if exhaustively matched
                };

                // Convert Stream to AsyncRead
                let reader = StreamReader::new(stream);
                // Encode
                let encoder = async_compression::tokio::bufread::GzipEncoder::new(reader);
                // Convert AsyncRead to Stream
                let new_stream = ReaderStream::new(encoder)
                    // Map back to feu_core error
                    .map_err(|e| feu_core::error::Error::msg(e.to_string()));

                // Set body
                *res.body_mut() = FeuBody::Stream(Box::pin(new_stream));

                // Update headers
                res.headers_mut().remove(CONTENT_LENGTH);
                res.headers_mut()
                    .insert(CONTENT_ENCODING, "gzip".parse().unwrap());
            }

            Ok(res)
        })
    }
}
