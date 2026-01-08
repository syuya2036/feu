use bytes::Bytes;
use http::StatusCode;

#[derive(Debug, Default)]
pub enum FeuBody {
    #[default]
    Empty,
    Bytes(Bytes),
    Text(String),
    // Stream will be added later with "streaming" feature
}

impl From<()> for FeuBody {
    fn from(_: ()) -> Self {
        FeuBody::Empty
    }
}

impl From<Bytes> for FeuBody {
    fn from(b: Bytes) -> Self {
        FeuBody::Bytes(b)
    }
}

impl From<Vec<u8>> for FeuBody {
    fn from(v: Vec<u8>) -> Self {
        FeuBody::Bytes(Bytes::from(v))
    }
}

impl From<String> for FeuBody {
    fn from(s: String) -> Self {
        FeuBody::Text(s)
    }
}

impl From<&'static str> for FeuBody {
    fn from(s: &'static str) -> Self {
        FeuBody::Text(s.to_string())
    }
}

pub type FeuRequest = http::Request<FeuBody>;

#[derive(Debug)]
pub struct FeuResponse(pub http::Response<FeuBody>);

impl FeuResponse {
    pub fn empty(status: StatusCode) -> Self {
        let mut res = http::Response::new(FeuBody::Empty);
        *res.status_mut() = status;
        FeuResponse(res)
    }

    pub fn text(body: impl Into<String>) -> Self {
        let mut res = http::Response::new(FeuBody::Text(body.into()));
        res.headers_mut().insert(
            http::header::CONTENT_TYPE,
            http::HeaderValue::from_static("text/plain; charset=utf-8"),
        );
        FeuResponse(res)
    }

    pub fn html(body: impl Into<String>) -> Self {
        let mut res = http::Response::new(FeuBody::Text(body.into()));
        res.headers_mut().insert(
            http::header::CONTENT_TYPE,
            http::HeaderValue::from_static("text/html; charset=utf-8"),
        );
        FeuResponse(res)
    }

    pub fn redirect(location: impl Into<String>, status: StatusCode) -> Self {
        let mut res = http::Response::new(FeuBody::Empty);
        *res.status_mut() = status;

        if let Ok(val) = http::HeaderValue::from_str(&location.into()) {
            res.headers_mut().insert(http::header::LOCATION, val);
        }

        FeuResponse(res)
    }

    // Fluent builders (non-mutating style for composition)
    pub fn with_status(mut self, status: StatusCode) -> Self {
        *self.0.status_mut() = status;
        self
    }

    pub fn with_header(
        mut self,
        key: http::header::HeaderName,
        value: http::header::HeaderValue,
    ) -> Self {
        self.0.headers_mut().insert(key, value);
        self
    }

    pub fn into_inner(self) -> http::Response<FeuBody> {
        self.0
    }
}

impl From<http::Response<FeuBody>> for FeuResponse {
    fn from(inner: http::Response<FeuBody>) -> Self {
        FeuResponse(inner)
    }
}
