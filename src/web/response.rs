use std::fmt;

use actix_web::{http::StatusCode, Error, HttpRequest, HttpResponse, Responder, ResponseError};
use futures::future::{err, ok, Ready};
use serde::Serialize;

#[derive(Serialize, Debug)]
#[serde(untagged)]
pub enum ResultData<T: Serialize> {
    Ok(T),
    Err(String),
}

#[derive(Serialize, Debug)]
pub struct Response<T: Serialize> {
    ok: bool,
    #[serde(skip_serializing)]
    status_code: StatusCode,
    result: ResultData<T>,
}

impl<T: Serialize> Response<T> {
    pub fn ok(result: T) -> Response<T> {
        Response {
            ok: true,
            status_code: StatusCode::OK,
            result: ResultData::Ok(result),
        }
    }

    pub fn err(status_code: StatusCode, message: String) -> Response<T> {
        Response {
            ok: false,
            status_code,
            result: ResultData::Err(message),
        }
    }
}

impl<T: Serialize> Responder for Response<T> {
    type Error = Error;
    type Future = Ready<Result<HttpResponse, Error>>;

    fn respond_to(self, _: &HttpRequest) -> Self::Future {
        let body = match serde_json::to_string(&self) {
            Ok(body) => body,
            Err(e) => return err(e.into()),
        };

        ok(HttpResponse::build(self.status_code)
            .content_type("application/json")
            .body(body))
    }
}

impl<T: Serialize> fmt::Display for Response<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match serde_json::to_string(&self) {
            Ok(data) => write!(f, "{}", data),
            Err(err) => write!(
                f,
                "{}",
                Response::<T>::err(StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
            ),
        }
    }
}

impl ResponseError for Response<String> {
    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code).json(self)
    }

    fn status_code(&self) -> StatusCode {
        self.status_code
    }
}
