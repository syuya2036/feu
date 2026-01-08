use crate::error::Error;
use crate::types::{FeuBody, FeuResponse};
use http::StatusCode;

pub trait IntoResponse {
    fn into_response(self) -> FeuResponse;
}

impl IntoResponse for FeuResponse {
    fn into_response(self) -> FeuResponse {
        self
    }
}

impl IntoResponse for http::Response<FeuBody> {
    fn into_response(self) -> FeuResponse {
        FeuResponse(self)
    }
}

impl<T> IntoResponse for Result<T, Error>
where
    T: IntoResponse,
{
    fn into_response(self) -> FeuResponse {
        match self {
            Ok(val) => val.into_response(),
            Err(e) => {
                // Should probably have a way to map error to response.
                // For now, simple 500 or 400 based on kind.
                let status = match e.kind() {
                    crate::error::ErrorKind::NotFound => StatusCode::NOT_FOUND,
                    crate::error::ErrorKind::BadRequest => StatusCode::BAD_REQUEST,
                    crate::error::ErrorKind::Internal => StatusCode::INTERNAL_SERVER_ERROR,
                    crate::error::ErrorKind::Message(_) => StatusCode::BAD_REQUEST,
                };
                FeuResponse::text(e.to_string()).with_status(status)
            }
        }
    }
}

impl<T> IntoResponse for (StatusCode, T)
where
    T: IntoResponse,
{
    fn into_response(self) -> FeuResponse {
        let (status, body) = self;
        body.into_response().with_status(status)
    }
}

// Implement for string types
impl IntoResponse for String {
    fn into_response(self) -> FeuResponse {
        FeuResponse::text(self)
    }
}

impl IntoResponse for &'static str {
    fn into_response(self) -> FeuResponse {
        FeuResponse::text(self)
    }
}

impl IntoResponse for () {
    fn into_response(self) -> FeuResponse {
        FeuResponse::empty(StatusCode::OK)
    }
}
