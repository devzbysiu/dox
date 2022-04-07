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
    use rocket::{http::Status, local::blocking::Client};
    use std::time::Duration;

    use testutils::{
        create_cfg_file, index_dir_path, override_config_path, thumbnails_dir_path,
        watched_dir_path, TestConfig,
    };

    use crate::launch;
    use anyhow::Result;

    #[test]
    fn test_search_endpoint() -> Result<()> {
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
        let resp = client.get("/thumbnails/all").dispatch();

        // then
        assert_eq!(resp.status(), Status::Ok);

        Ok(())
    }
}
