use crate::result::Result;
use crate::use_cases::config::Config;
use crate::use_cases::persistence::Persistence;
use crate::use_cases::repository::{RepoRead, SearchResult};
use crate::use_cases::user::User;

use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::serde::Deserialize;
use rocket::{get, post, State};
use std::fs::File;
use tracing::instrument;

#[instrument(skip(repo))]
#[get("/search?<q>")]
pub fn search(user: User, q: String, repo: &State<RepoRead>) -> Result<Json<SearchResult>> {
    Ok(Json(repo.search(user, q)?))
}

#[instrument(skip(persistence))]
#[get("/thumbnail/<filename>")]
pub fn thumbnail(
    user: User,
    filename: String,
    cfg: &State<Config>,
    persistence: &State<Persistence>,
) -> Result<Option<File>> {
    persistence.load(cfg.thumbnails_dir.join(relative_path(&user, filename)))
}

#[instrument(skip(repo))]
#[get("/thumbnails/all")]
pub fn all_thumbnails(user: User, repo: &State<RepoRead>) -> Result<Json<SearchResult>> {
    Ok(Json(repo.all_documents(user)?))
}

#[instrument(skip(persistence))]
#[allow(clippy::needless_pass_by_value)] // rocket requires pass by value here
#[get("/document/<filename>")]
pub fn document(
    user: User,
    filename: String,
    cfg: &State<Config>,
    persistence: &State<Persistence>,
) -> Result<Option<File>> {
    persistence.load(cfg.watched_dir.join(relative_path(&user, filename)))
}

fn relative_path<S: Into<String>>(user: &User, filename: S) -> String {
    format!("{}/{}", base64::encode(&user.email), filename.into())
}

#[instrument(skip(doc, persistence))]
#[allow(clippy::needless_pass_by_value)] // rocket requires pass by value here
#[post("/document/upload", data = "<doc>")]
pub fn receive_document(
    user: User,
    doc: Json<Document>,
    cfg: &State<Config>,
    persistence: &State<Persistence>,
) -> Result<Status> {
    persistence.save(
        cfg.watched_dir.join(relative_path(&user, &doc.filename)),
        &base64::decode(&doc.body)?,
    )?;
    Ok(Status::Created)
}

#[derive(Debug, Deserialize)]
pub struct Document {
    filename: String,
    body: String,
}

#[cfg(test)]
mod test {
    use crate::launch;

    use anyhow::Result;
    use rocket::{http::Status, local::blocking::Client};
    use serial_test::serial;
    use std::thread;
    use std::time::Duration;
    use testutils::{cp_docs, create_test_env, to_base64, LocalResponseExt};

    #[test]
    #[serial]
    #[ignore] // TODO: take care of authorization
    fn test_search_endpoint_with_empty_index() -> Result<()> {
        // given
        let _env = create_test_env()?;
        let client = Client::tracked(launch())?;

        // when
        let mut resp = client.get("/search?q=not-important").dispatch();
        let body = resp.read_body::<14>()?;

        // then
        assert_eq!(resp.status(), Status::Ok);
        assert_eq!(body, r#"{"entries":[]}"#);

        Ok(())
    }

    // TODO: when there are multiple active tests which require indexing,
    // then at least one test is failing - but only in cargo make, in cargo test
    // everything is working correctly
    #[test]
    #[serial]
    #[ignore] // TODO: take care of authorization
    fn test_search_endpoint_with_indexed_docs() -> Result<()> {
        // given
        let (config, _config_dir) = create_test_env()?;
        let client = Client::tracked(launch())?;
        thread::sleep(Duration::from_secs(5));
        cp_docs(config.watched_dir_path())?;

        // when
        let mut resp = client.get("/search?q=Parlamentarny").dispatch();
        let body = resp.read_body::<60>()?;

        // then
        assert_eq!(resp.status(), Status::Ok);
        assert_eq!(
            body,
            r#"{"entries":[{"filename":"doc1.png","thumbnail":"doc1.png"}]}"#
        );

        Ok(())
    }

    #[test]
    #[serial]
    #[ignore] // TODO: take care of authorization
    fn test_search_endpoint_with_wrong_query() -> Result<()> {
        // given
        let (config, _config_dir) = create_test_env()?;
        let client = Client::tracked(launch())?;
        thread::sleep(Duration::from_secs(5));
        cp_docs(config.watched_dir_path())?;

        // when
        let mut resp = client.get("/search?q=not-existing-query").dispatch();
        let body = resp.read_body::<14>()?;

        // then
        assert_eq!(resp.status(), Status::Ok);
        assert_eq!(body, r#"{"entries":[]}"#);

        Ok(())
    }

    #[test]
    #[serial]
    #[ignore] // TODO: take care of authorization
    fn test_all_thumbnails_endpoint_with_empty_index() -> Result<()> {
        // given
        let _env = create_test_env()?;
        let client = Client::tracked(launch())?;

        // when
        let mut resp = client.get("/thumbnails/all").dispatch();
        let body = resp.read_body::<14>()?;

        // then
        assert_eq!(resp.status(), Status::Ok);
        assert_eq!(body, r#"{"entries":[]}"#);

        Ok(())
    }

    #[test]
    #[serial]
    #[ignore] // TODO: take care of authorization
    fn test_all_thumbnails_endpoint_with_indexed_docs() -> Result<()> {
        // given
        let (config, _config_dir) = create_test_env()?;
        let client = Client::tracked(launch())?;
        thread::sleep(Duration::from_secs(5));
        cp_docs(config.watched_dir_path())?;

        // when
        let mut resp = client.get("/thumbnails/all").dispatch();
        let body = resp.read_body::<60>()?;

        // then
        assert_eq!(resp.status(), Status::Ok);
        assert_eq!(
            body,
            r#"{"entries":[{"filename":"doc1.png","thumbnail":"doc1.png"}]}"#
        );

        Ok(())
    }

    #[test]
    #[serial]
    #[ignore] // TODO: take care of authorization
    fn test_receive_document_endpoint() -> Result<()> {
        // given
        let _env = create_test_env()?;
        let client = Client::tracked(launch())?;

        let mut resp = client.get("/search?q=Parlamentarny").dispatch();
        let body = resp.read_body::<14>()?;
        assert_eq!(resp.status(), Status::Ok);
        assert_eq!(body, r#"{"entries":[]}"#);

        // when
        let resp = client
            .post("/document/upload")
            .body(format!(
                r#"{{ "filename": "doc1.png", "body": "{}" }}"#,
                to_base64("res/doc1.png")?
            ))
            .dispatch();
        assert_eq!(resp.status(), Status::Created);

        thread::sleep(Duration::from_secs(15)); // allow to index docs

        let mut resp = client.get("/search?q=Parlamentarny").dispatch();
        let body = resp.read_body::<60>()?;

        // then
        assert_eq!(resp.status(), Status::Ok);
        assert_eq!(
            body,
            r#"{"entries":[{"filename":"doc1.png","thumbnail":"doc1.png"}]}"#
        );

        Ok(())
    }
}
