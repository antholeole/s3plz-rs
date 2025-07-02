use axum::{
    extract::{Path, State},
    response::{IntoResponse, Response},
};
use hyper::{StatusCode, Uri};
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use tracing::debug;

use crate::{error::S3Error, shared_state::SharedState};

pub async fn get_path_oneseg(
    State(state): State<Arc<SharedState>>,
    uri: Uri,
    _: Path<String>,
) -> Response {
    get_path(state, uri)
}
pub async fn get_path_twoseg(
    State(state): State<Arc<SharedState>>,
    uri: Uri,
    _: Path<(String, String)>,
) -> Response {
    get_path(state, uri)
}
pub async fn get_path_threeseg(
    State(state): State<Arc<SharedState>>,
    uri: Uri,
    _: Path<(String, String, String)>,
) -> Response {
    get_path(state, uri)
}
pub async fn get_path_fourseg(
    State(state): State<Arc<SharedState>>,
    uri: Uri,
    _: Path<(String, String, String, String)>,
) -> Response {
    get_path(state, uri)
}

fn get_path(state: Arc<SharedState>, uri: Uri) -> Response {
    let path_as_pathbuf = PathBuf::from(uri.path());
    let mut path_as_components = path_as_pathbuf.components();
    path_as_components.next();

    let key = path_as_components.as_path().to_string_lossy().to_string();

    match state.etags_and_sizes.get(&key) {
        Some(metadata) => {
            let mut full_path = state.root_path.clone();
            full_path.push(key.clone());
            let file = fs::read(full_path.clone());

            match file {
                Err(e) => {
                    debug!("recived request for key {}, could not find it.", key);
                    S3Error {
                    code: "CouldNotReadOffDisk",
                    status: StatusCode::INTERNAL_SERVER_ERROR,
                    message: format!(
                        "attempted to read file {:?}, but it doesn't exist. did you delete it (og error: {})?",
                        full_path,
                        e
                    ),
                }}
                .into_response(),
                Ok(content) => {
                    debug!("successfully responded to request for key {}.", key);
                    (
                    StatusCode::OK,
                    [
                        ("Content-Type", "application/json"), // whoops
                        ("Last-Modified", &state.last_modified.to_string()),
                        ("ETag", &metadata.0),
                        ("Content-Length", &metadata.1.to_string()),
                    ],
                    content
                )
                    .into_response()},
            }
        }
        None => StatusCode::NOT_FOUND.into_response(),
    }
}
