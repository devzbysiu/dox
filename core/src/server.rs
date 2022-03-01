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

#[derive(Deserialize)]
pub struct Document {
    filename: String,
    body: String,
}
