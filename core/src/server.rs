use crate::cfg::Config;
use crate::indexer::{Repo, SearchResults};
use crate::result::Result;

use log::debug;
use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::serde::Deserialize;
use rocket::{get, post, State};
use std::fs::File;
use std::io::prelude::*;

#[get("/search?<q>")]
pub fn search(q: String, repo: &State<Repo>) -> Result<Json<SearchResults>> {
    Ok(Json(repo.search(q)?))
}

#[get("/thumbnails/all")]
pub fn all_thumbnails(repo: &State<Repo>) -> Result<Json<SearchResults>> {
    Ok(Json(repo.all_documents()?))
}

#[allow(clippy::needless_pass_by_value)] // rocket requires pass by value here
#[post("/document/upload", data = "<doc>")]
pub fn receive_document(doc: Json<Document>, cfg: &State<Config>) -> Result<Status> {
    debug!("receiving document: {}", doc.filename);
    let mut document = File::create(cfg.watched_dir.join(&doc.filename))?;
    document.write_all(&base64::decode(&doc.body)?)?;
    Ok(Status::Created)
}

#[derive(Debug, Deserialize)]
pub struct Document {
    filename: String,
    body: String,
}

#[cfg(test)]
mod test {
    use rocket::{
        http::Status,
        local::blocking::{Client, LocalResponse},
    };
    use serial_test::serial;
    use std::{io::Read, time::Duration};

    use testutils::{
        cp_docs, create_cfg_file, index_dir_path, override_config_path, thumbnails_dir_path,
        watched_dir_path, TestConfig,
    };

    use crate::launch;
    use anyhow::Result;

    #[test]
    #[serial]
    fn test_all_thumbnails_endpoint_with_empty_index() -> Result<()> {
        // given
        let index_dir = index_dir_path()?;
        let watched_dir = watched_dir_path()?;
        let thumbnails_dir = thumbnails_dir_path()?;
        let config = create_cfg_file(&TestConfig {
            watched_dir: watched_dir.path().to_path_buf(),
            thumbnails_dir: thumbnails_dir.path().to_path_buf(),
            index_dir: index_dir.path().to_path_buf(),
            cooldown_time: Duration::from_secs(1),
        })?;
        override_config_path(&config.path().join("dox.toml"));
        let client = Client::tracked(launch())?;

        // when
        let mut resp: LocalResponse = client.get("/thumbnails/all").dispatch();
        let mut buffer = [0; 14];
        resp.read(&mut buffer)?;
        let body = String::from_utf8(buffer.to_vec())?;

        // then
        assert_eq!(resp.status(), Status::Ok);
        assert_eq!(body, r#"{"entries":[]}"#);

        Ok(())
    }

    #[test]
    #[serial]
    fn test_all_thumbnails_endpoint_indexed_docs() -> Result<()> {
        // given
        let index_dir = index_dir_path()?;
        let watched_dir = watched_dir_path()?;
        let thumbnails_dir = thumbnails_dir_path()?;
        let config = create_cfg_file(&TestConfig {
            watched_dir: watched_dir.path().to_path_buf(),
            thumbnails_dir: thumbnails_dir.path().to_path_buf(),
            index_dir: index_dir.path().to_path_buf(),
            cooldown_time: Duration::from_secs(1),
        })?;
        override_config_path(&config.path().join("dox.toml"));
        let client = Client::tracked(launch())?;
        cp_docs(watched_dir.path())?;

        // when
        let mut resp: LocalResponse = client.get("/thumbnails/all").dispatch();
        let mut buffer = [0; 60];
        resp.read(&mut buffer)?;
        let body = String::from_utf8(buffer.to_vec())?;

        // then
        assert_eq!(resp.status(), Status::Ok);
        assert_eq!(
            body,
            r#"{"entries":[{"filename":"doc1.png","thumbnail":"doc1.png"}]}"#
        );

        Ok(())
    }
}
