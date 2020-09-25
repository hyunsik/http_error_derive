# http_error_derive

http_error_derive crate provides a convenient derive macro for HTTP errors.

## Usage
The following enum is an example of http_error_derive.
```
#[derive(HttpError)]
pub enum ApiError {
    #[detail(
        status = 400,
        message = "bad request. Use correct values of the request and HTTP headers"
    )]
    BadRequest,
    #[detail(status = 400, message = "target_ir field is invalid")]
    InvalidTargetIr,
    #[detail(status = 400, message = "source field is missing")]
    MissingSource,
    #[detail(status = 400, message = "target_npu_spec field is missing")]
    MissingNpuSpec,
    #[detail(status = 400, message = "target_npu_spec format is invalid")]
    InvalidNpuSpec,
    #[detail(
        status = 409,
        message = "The same API key name already exists. Please use another API key name."
    )]
    DuplicateApiKeyName,
    DssInvalidDynamicRanges,
    #[detail(
        status = 501,
        message = "Currently, Nux Quantizer is not able to handle this model"
    )]
}
```

With the http_error_derive, we can readily implement extensible HTTP error responses
in actix-web and other REST/GRPC API frameworks.

The following is an example to show
how we can easily implement http error responses including error code, error message, http status code with actix-web.

```
use actix_web::http::StatusCode;
use actix_web::{HttpResponse, ResponseError};
use http_error_derive::HttpError;
use serde::Serialize;

#[derive(Serialize)]
pub struct ApiResponse {
    pub error_code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trace_id: Option<String>,
}

impl<'a> From<&'a ApiError> for ApiResponse {
    fn from(e: &'a ApiError) -> Self {
        ApiResponse {
            error_code: e.error_code().to_string(),
            message: e.message().to_string(),
            trace_id: None,
        }
    }
}

impl<'a> From<&'a ApiError> for HttpResponse {
    fn from(e: &'a ApiError) -> Self {
        let http_status = e.http_status();
        let error: ApiResponse = e.into();

        HttpResponse::Ok()
            .status(StatusCode::from_u16(http_status).unwrap())
            .json(error)
    }
}
```
