use crate::helpers::cipher::{self, key, nonce};
use crate::result::Result;
use crate::use_cases::config::Config;
use crate::use_cases::persistence::Persistence;
use crate::use_cases::repository::{RepoRead, SearchResult};
use crate::use_cases::user::User;

use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::serde::Deserialize;
use rocket::{get, post, State};
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
) -> Result<Option<Vec<u8>>> {
    Ok(
        match persistence.load(cfg.thumbnails_dir.join(relative_path(&user, filename)))? {
            Some(buf) => Some(cipher::decrypt(&buf, key(), nonce())?),
            None => None,
        },
    )
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
) -> Result<Option<Vec<u8>>> {
    Ok(
        match persistence.load(cfg.watched_dir.join(relative_path(&user, filename)))? {
            Some(buf) => Some(cipher::decrypt(&buf, key(), nonce())?),
            None => None,
        },
    )
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
    use crate::rocket;

    use anyhow::Result;
    use rocket::local::blocking::LocalResponse;
    use rocket::{http::Status, local::blocking::Client};
    use serial_test::serial;
    use std::io::Read;
    use std::thread;
    use std::time::Duration;
    use tempfile::TempDir;
    use testutils::{cp_docs, create_test_env, to_base64};
    use tracing::{debug, instrument};

    // TODO: this trait WAS also defined here [`testutils::LocalResponseExt`], but for some reason
    // it's not visible by rust compiler. However, when rocket is set to version 0.5.0-rc.1, it's
    // working properly, but for 0.5.0-rc.2 it's not working - no idea why. For now, I'm leaving
    // the definition of LocalResponseExt here
    trait LocalResponseExt {
        fn read_body(&mut self) -> Result<String>;
    }

    impl LocalResponseExt for LocalResponse<'_> {
        #[instrument]
        fn read_body(&mut self) -> Result<String> {
            let mut buffer = Vec::new();
            self.read_to_end(&mut buffer)?;
            let res = String::from_utf8(buffer)?;
            debug!("read the whole buffer: '{}'", res);
            Ok(res)
        }
    }

    #[test]
    #[serial]
    fn test_search_endpoint_with_empty_index() -> Result<()> {
        // given
        let (config, config_dir) = create_test_env()?;
        let client = Client::tracked(rocket(Some(config_dir_string(&config_dir))))?;

        // when
        let mut resp = client.get("/search?q=not-important").dispatch();
        let body = resp.read_body()?;

        // then
        assert_eq!(resp.status(), Status::Ok);
        assert_eq!(body, r#"{"entries":[]}"#);

        Ok(())
    }

    fn config_dir_string(config_dir: &TempDir) -> String {
        config_dir
            .path()
            .join("dox.toml")
            .to_string_lossy()
            .to_string()
    }

    // TODO: when there are multiple active tests which require indexing,
    // then at least one test is failing - but only in cargo make, in cargo test
    // everything is working correctly
    #[test]
    #[serial]
    #[ignore]
    fn test_search_endpoint_with_indexed_docs() -> Result<()> {
        // given
        let (config, config_dir) = create_test_env()?;
        let client = Client::tracked(rocket(Some(config_dir_string(&config_dir))))?;
        thread::sleep(Duration::from_secs(5));
        let user_dir_name = base64::encode("some@email.com"); // TODO: it's repetition, think about this
        cp_docs(config.watched_dir_path().join(user_dir_name))?;

        // when
        let mut resp = client.get("/search?q=Parlamentarny").dispatch();
        let body = resp.read_body()?;

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
    fn test_search_endpoint_with_wrong_query() -> Result<()> {
        // given
        let (config, config_dir) = create_test_env()?;
        let client = Client::tracked(rocket(Some(config_dir_string(&config_dir))))?;
        thread::sleep(Duration::from_secs(5));
        let user_dir_name = base64::encode("some@email.com");
        cp_docs(config.watched_dir_path().join(user_dir_name))?;

        // when
        let mut resp = client.get("/search?q=not-existing-query").dispatch();
        let body = resp.read_body()?;

        // then
        assert_eq!(resp.status(), Status::Ok);
        assert_eq!(body, r#"{"entries":[]}"#);

        Ok(())
    }

    #[test]
    #[serial]
    fn test_all_thumbnails_endpoint_with_empty_index() -> Result<()> {
        // given
        let (config, config_dir) = create_test_env()?;
        let client = Client::tracked(rocket(Some(config_dir_string(&config_dir))))?;

        // when
        let mut resp = client.get("/thumbnails/all").dispatch();
        let body = resp.read_body()?;

        // then
        assert_eq!(resp.status(), Status::Ok);
        assert_eq!(body, r#"{"entries":[]}"#);

        Ok(())
    }

    #[test]
    #[serial]
    #[ignore]
    fn test_all_thumbnails_endpoint_with_indexed_docs() -> Result<()> {
        // given
        let (config, config_dir) = create_test_env()?;
        let client = Client::tracked(rocket(Some(config_dir_string(&config_dir))))?;
        thread::sleep(Duration::from_secs(5));
        let user_dir_name = base64::encode("some@email.com");
        cp_docs(config.watched_dir_path().join(user_dir_name))?;

        // when
        let mut resp = client.get("/thumbnails/all").dispatch();
        let body = resp.read_body()?;

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
    #[ignore]
    fn test_receive_document_endpoint() -> Result<()> {
        // given
        let (config, config_dir) = create_test_env()?;
        let client = Client::tracked(rocket(Some(config_dir_string(&config_dir))))?;

        let mut resp = client.get("/search?q=Parlamentarny").dispatch();
        let body = resp.read_body()?;
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
        let body = resp.read_body()?;

        // then
        assert_eq!(resp.status(), Status::Ok);
        assert_eq!(
            body,
            r#"{"entries":[{"filename":"doc1.png","thumbnail":"doc1.png"}]}"#
        );

        Ok(())
    }
}
