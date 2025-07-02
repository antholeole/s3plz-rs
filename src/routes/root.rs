use crate::util::fallback;
use crate::{error::S3Error, shared_state::SharedState};
use axum::extract::{Query, State};
use axum::response::{IntoResponse, Response};
use chrono::Utc;
use hyper::{StatusCode, Uri};
use serde::{Deserialize, Serialize};
use serde_xml_rs::to_string;
use std::sync::Arc;
use std::vec::Vec;
use tracing::debug;

#[derive(Deserialize)]
pub struct RootQuery {
    #[serde(rename = "list-type")]
    list_type: Option<usize>,

    location: Option<String>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct Content {
    key: String,
    size: usize,
    last_modified: chrono::DateTime<Utc>,
    #[serde(rename = "ETag")]
    etag: String,
    storage_class: String,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ListBucketResult {
    #[serde(rename = "Name")]
    bucket_name: String,

    is_truncated: bool,
    max_keys: usize,
    key_count: usize,

    // even if we know that we are only passing in 'static, just make it a
    // heap string so we can deseralize for tests.
    encoding_type: String,

    #[serde(rename = "Contents")]
    contents: Vec<Content>,
}

impl ListBucketResult {
    pub fn new(state: &SharedState) -> ListBucketResult {
        ListBucketResult {
            contents: state
                .etags_and_sizes
                .iter()
                .map(|(key, (etag, size))| Content {
                    key: key.clone(),
                    etag: etag.clone(),
                    size: *size,
                    last_modified: state.last_modified,
                    storage_class: "STANDARD".to_string(),
                })
                .collect(),
            bucket_name: state.bucket_name.clone(),
            is_truncated: false,
            key_count: state.etags_and_sizes.len(),
            max_keys: 1000,
            encoding_type: "url".to_string(),
        }
    }
}

impl IntoResponse for ListBucketResult {
    fn into_response(self) -> Response {
        to_string(&self).unwrap().into_response()
    }
}

pub async fn head_root() -> impl IntoResponse {
    StatusCode::OK
}

pub async fn get_root(
    Query(query): Query<RootQuery>,
    State(state): State<Arc<SharedState>>,
    uri: Uri,
) -> Result<Response, S3Error> {
    if query.location.is_some() {
        debug!("recieved location query. Responding with mock XML.");
        return Ok((
            StatusCode::OK,
            [("content-type", "application/xml")],
            r#"<?xml version="1.0" encoding="UTF-8"?>
<LocationConstraint xmlns="http://s3.amazonaws.com/doc/2006-03-01/">Europe</LocationConstraint>     
"#,
        )
            .into_response());
    }

    if query.list_type.is_some() {
        debug!("handling listv2 query.");
        return Ok((
            StatusCode::OK,
            [("content-type", "application/xml")],
            to_string(&ListBucketResult::new(&state)).unwrap(),
        )
            .into_response());
    }

    Err(fallback(uri).await)
}

#[cfg(test)]
mod tests {
    use serde_xml_rs::from_str;

    use super::ListBucketResult;

    #[test]
    fn list_bucket_spec() {
        let list_bucket_example = r#"<?xml version="1.0" encoding="UTF-8"?>
<ListBucketResult xmlns="http://s3.amazonaws.com/doc/2006-03-01/">
  <Name>s3plz-test-bucket</Name>
  <Contents>
    <Key>deployment.yaml</Key>
    <Size>364</Size>
    <LastModified>2025-07-03T04:21:58.888Z</LastModified>
    <ETag>&quot;d79e6507072e6cf55e90ff4cda2a45b6&quot;</ETag>
    <StorageClass>STANDARD</StorageClass>
  </Contents>
  <Contents>
    <Key>fixtures%2Fdeployment.yaml</Key>
    <Size>364</Size>
    <LastModified>2025-07-03T17:36:14.244Z</LastModified>
    <ETag>&quot;d79e6507072e6cf55e90ff4cda2a45b6&quot;</ETag>
    <StorageClass>STANDARD</StorageClass>
  </Contents>
    <IsTruncated>false</IsTruncated>
  <MaxKeys>1000</MaxKeys>
  <KeyCount>4</KeyCount>
  <EncodingType>url</EncodingType>
</ListBucketResult>'
        "#;

        let deserialized = from_str::<ListBucketResult>(list_bucket_example).unwrap();

        assert_eq!(
            deserialized.bucket_name, "s3plz-test-bucket",
            "bucket name mismatch"
        );
        assert_eq!(deserialized.is_truncated, false, "is_truncated mismatch");
        assert_eq!(deserialized.max_keys, 1000, "max_keys mismatch");
        assert_eq!(deserialized.key_count, 4, "key_count mismatch");
        assert_eq!(deserialized.encoding_type, "url", "encoding_type mismatch");
        assert_eq!(deserialized.contents.len(), 2, "contents length mismatch");
        assert_eq!(
            deserialized.contents[0].key, "deployment.yaml",
            "first content key mismatch"
        );
        assert_eq!(
            deserialized.contents[1].key, "fixtures%2Fdeployment.yaml",
            "second content key mismatch"
        );
    }
}
