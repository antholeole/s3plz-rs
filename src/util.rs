use hyper::{StatusCode, Uri};
use tracing::debug;

use crate::error::S3Error;

pub async fn fallback(uri: Uri) -> S3Error {
    debug!(
        "recived unknown request for {}. Maybe you can implement it?",
        uri
    );

    S3Error {
        code: "NotImplemented",
        message: "Root query was unable to decypher; perhaps unimplemented?".to_string(),
        status: StatusCode::NOT_IMPLEMENTED,
    }
}
