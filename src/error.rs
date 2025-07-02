use axum::response::{IntoResponse, Response};
use hyper::StatusCode;
use serde::Serialize;
use serde_xml_rs::to_string;

pub struct S3Error {
    pub code: &'static str,
    pub message: String,
    pub status: StatusCode,
}

#[derive(Serialize)]
#[serde(rename = "Error", rename_all = "PascalCase")]
struct S3ErrorInner {
    message: String,
    pub code: &'static str,
}

impl IntoResponse for S3Error {
    fn into_response(self) -> Response {
        (
            self.status,
            [("Content-Type", "application/xml")],
            to_string(&S3ErrorInner {
                message: self.message,
                code: self.code,
            })
            .unwrap(),
        )
            .into_response()
    }
}
