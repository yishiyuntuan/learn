use bon::Builder;
use serde::{Deserialize, Serialize};
use springboot_web::axum::response::{IntoResponse, Response as AxumResponse};
use springboot_web::extractor::Json;
pub type Resp = Result<AxumResponse, AppError>;

#[derive(Builder, Deserialize, Serialize)]
pub struct Response<T>
where
    T: Serialize,
{
    code: i32,
    message: String,
    data: Option<T>,
    success: bool,
}

impl<T> Response<T>
where
    T: Serialize,
{
    fn success(data: T) -> Self {
        Response::builder()
            .code(1000)
            .message("ok".to_owned())
            .data(data)
            .success(true)
            .build()
    }

    pub fn to_json_result(self) -> Resp {
        Ok(Json(self).into_response())
    }

    pub fn ok(data: T) -> Resp {
        Response::success(data).to_json_result()
    }
}

impl Response<()> {
    pub fn err(error: i32, message: &str) -> Self {
        Response {
            code: error,
            message: message.to_owned(),
            data: None,
            success: false,
        }
    }

    #[allow(dead_code)]
    pub fn error(err_code: i32, msg: &str) -> Resp {
        Response::err(err_code, msg).to_json_result()
    }
}

// Make our own error that wraps `anyhow::Error`.
pub struct AppError(anyhow::Error);

// Tell axum how to convert `AppError` into a response.
impl IntoResponse for AppError {
    fn into_response(self) -> AxumResponse {
        Json(Response::err(1000, self.0.to_string().as_str())).into_response()
    }
}

// This enables using `?` on functions that return `Result<_, anyhow::Error>` to turn them into
// `Result<_, AppError>`. That way you don't need to do that manually.
impl<E> From<E> for AppError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        Self(err.into())
    }
}
