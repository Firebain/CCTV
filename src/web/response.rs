use std::error;
use std::fmt;

use serde::Serialize;
use futures::future::{ok, err, Ready};
use actix_web::{Responder, HttpRequest, HttpResponse, http::StatusCode, Error};

#[derive(Serialize, Debug)]
#[serde(untagged)]
pub enum ResultData<T: Serialize> {
    Ok(T),
    Err(String)
}

#[derive(Serialize, Debug)]
pub struct Response<T: Serialize> {
    ok: bool,
    #[serde(skip_serializing)]
    status_code: StatusCode,
    result: ResultData<T>
}

impl<T: Serialize> Response<T> {
    pub fn ok(result: T) -> Response<T> {
        Response {
            ok: true,
            status_code: StatusCode::OK,
            result: ResultData::Ok(result)
        }
    }

    pub fn err(status_code: StatusCode, message: String) -> Response<T> {
        Response {
            ok: false,
            status_code,
            result: ResultData::Err(message)
        }
    }

    pub fn decide<E>(data: Result<T, E>) -> Response<T> 
    where
        E: error::Error + fmt::Display  
    {
        match data {
            Ok(result) => Self::ok(result),
            Err(err) => Self::err(StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
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