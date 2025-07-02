use base64::{Engine as _, engine::general_purpose};
use chrono::Utc;
use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tracing::debug;

pub struct SharedState {
    pub bucket_name: String,
    pub root_path: PathBuf,
    pub last_modified: chrono::DateTime<Utc>,
    pub etags_and_sizes: HashMap<String, (String, usize)>,
}

impl SharedState {
    fn process_directory(
        dir: &Path,
        metadata: &mut HashMap<String, (String, usize)>,
        // the path we put in is not === to the dir; the s3 bucket does not
        // care that the dir is in /etc/, for instance.
        current_path: &str,
    ) -> io::Result<()> {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;

            let key = match current_path {
                "" => entry.file_name().to_string_lossy().to_string(),
                other => {
                    (other.to_string() + "/" + &entry.file_name().to_string_lossy()).to_string()
                }
            };

            let path = entry.path();

            if path.is_dir() {
                Self::process_directory(&path, metadata, &key)?;
            } else {
                let contents = fs::read(&path)?;
                let etag = general_purpose::STANDARD.encode(*md5::compute(&contents));

                metadata.insert(key, (etag, contents.len()));
            }
        }

        Ok(())
    }

    pub fn new(
        from_path: PathBuf,
        bucket_name: String,
        last_modified: chrono::DateTime<Utc>,
    ) -> io::Result<Arc<SharedState>> {
        let mut etags: HashMap<String, (String, usize)> = HashMap::new();

        Self::process_directory(from_path.as_path(), &mut etags, "")?;

        for (k, v) in &etags {
            debug!("{} -> etag: {} size: {}", k, v.0, v.1);
        }

        Ok(Arc::new(SharedState {
            bucket_name,
            etags_and_sizes: etags,
            last_modified,
            root_path: from_path,
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    static FIXTURE_BUCKET_NAME: &'static str = "s3plz-test-bucket";

    #[test]
    fn test_shared_state_built() {
        let state = SharedState::new(
            FIXTURE_BUCKET_NAME.into(),
            FIXTURE_BUCKET_NAME.into(),
            chrono::Utc::now(),
        )
        .unwrap();

        assert_eq!(state.etags_and_sizes.len(), 4);

        for (k, v) in &state.etags_and_sizes {
            println!("{} -> etag: {} size: {}", k, v.0, v.1);
        }

        vec![
            "deployment.yaml",
            "service.yaml",
            "things/deployment.yaml",
            "things/service.yaml",
        ]
        .into_iter()
        .for_each(|file| {
            assert!(
                &state.etags_and_sizes.contains_key(file),
                "expected {} to be in state but it was not!",
                file,
            )
        })
    }
}
