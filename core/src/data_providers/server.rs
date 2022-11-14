use crate::entities::extension::supported_extensions;
use crate::entities::user::User;
use crate::helpers::PathRefExt;
use crate::result::{DocumentReadErr, DocumentSaveErr, SearchErr, ThumbnailReadErr};
use crate::use_cases::cipher::CipherRead;
use crate::use_cases::config::Config;
use crate::use_cases::fs::Fs as Filesystem;
use crate::use_cases::repository::{RepoRead, SearchResult};

use anyhow::Context;
use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::serde::Deserialize;
use rocket::{get, post, State};
use std::path::Path;
use tracing::instrument;

type Cfg = State<Config>;
type Fs = State<Filesystem>;
type Cipher = State<CipherRead>;
type Repo = State<RepoRead>;
type Doc = Json<Document>;

type SearchRes = Result<Json<SearchResult>, SearchErr>;
type GetThumbRes = Result<Option<Vec<u8>>, ThumbnailReadErr>;
type GetAllThumbsRes = Result<Json<SearchResult>, ThumbnailReadErr>;
type GetDocRes = Result<(Status, Option<Vec<u8>>), DocumentReadErr>;
type PostDocRes = Result<(Status, String), DocumentSaveErr>;

#[instrument(skip(repo))]
#[get("/search?<q>")]
pub fn search(user: User, q: String, repo: &Repo) -> SearchRes {
    Ok(Json(repo.search(user, q).context("Searching failed.")?))
}

#[instrument(skip(fs, cipher))]
#[get("/thumbnail/<name>")]
pub fn thumbnail(user: User, name: String, cfg: &Cfg, fs: &Fs, cipher: &Cipher) -> GetThumbRes {
    let thumbnail_path = cfg.thumbnails_dir.join(relative_path(&user, name));
    let Some(buf) = fs.load(thumbnail_path)? else {
        return Ok(None);
    };
    Ok(Some(cipher.decrypt(&buf).context("Image decrypt failed.")?))
}

#[instrument(skip(repo))]
#[get("/thumbnails/all")]
pub fn all_thumbnails(user: User, repo: &Repo) -> GetAllThumbsRes {
    Ok(Json(repo.all_docs(user).context("Failed to read docs.")?))
}

#[instrument(skip(fs, cipher))]
#[allow(clippy::needless_pass_by_value)] // rocket requires pass by value here
#[get("/document/<filename>")]
pub fn document(user: User, filename: String, cfg: &Cfg, fs: &Fs, cipher: &Cipher) -> GetDocRes {
    let document_path = cfg.watched_dir.join(relative_path(&user, filename));
    if !fs.exists(&document_path) {
        return Ok((Status::NotFound, None));
    }
    let Some(buf) = fs.load(document_path).context("Failed to read document.")? else {
        return Ok((Status::InternalServerError, None));
    };
    Ok((
        Status::Ok,
        Some(cipher.decrypt(&buf).context("Doc decrypt failed.")?),
    ))
}

fn relative_path<S: Into<String>>(user: &User, filename: S) -> String {
    format!("{}/{}", base64::encode(&user.email), filename.into())
}

#[instrument(skip(doc, fs))]
#[allow(clippy::needless_pass_by_value)] // rocket requires pass by value here
#[post("/document/upload", data = "<doc>")]
pub fn receive_document(user: User, doc: Doc, cfg: &Cfg, fs: &Fs) -> PostDocRes {
    let filename = Path::new(&doc.filename);
    if !filename.has_supported_extension() {
        return Ok((Status::UnsupportedMediaType, wrong_extension_msg(filename)));
    }
    let to = cfg.watched_dir.join(relative_path(&user, &doc.filename));
    let doc = base64::decode(&doc.body).context("Failed to decode body.")?;
    fs.save(to, &doc).context("Failed to save document.")?;
    Ok((Status::Created, String::new()))
}

fn wrong_extension_msg<P: AsRef<Path>>(filename: P) -> String {
    let filename = filename.as_ref();
    format!(
        "File '{}' has unsupported extension. Those are supported: {:?}.",
        filename.display(),
        supported_extensions()
    )
}

#[derive(Debug, Deserialize)]
pub struct Document {
    filename: String,
    body: String,
}

#[cfg(test)]
mod test {
    use crate::configuration::telemetry::init_tracing;
    use crate::testingtools::integration::{doc, start_test_app, test_app};

    use anyhow::Result;
    use fake::{Fake, Faker};
    use rocket::http::Status;

    #[test]
    fn empty_index_returns_200_and_empty_json_entries() -> Result<()> {
        // given
        init_tracing();
        let app = start_test_app()?;

        // when
        let res = app.search(Faker.fake::<String>())?;

        // then
        assert_eq!(res.status, Status::Ok);
        assert_eq!(res.body, r#"{"entries":[]}"#);

        Ok(())
    }

    #[test]
    fn uploading_pdf_document_triggers_indexing() -> Result<()> {
        // given
        init_tracing();
        let mut app = test_app().with_tracked_repo()?.start()?;
        let search_term = "zdjęcie";

        let res = app.search(search_term)?;
        assert_eq!(res.status, Status::Ok);
        assert_eq!(res.body, r#"{"entries":[]}"#);

        // when
        app.upload_doc(doc("doc1.pdf"))?;
        app.wait_til_indexed();

        // TODO: for some reason, only one word search is working - fix it
        let res = app.search(search_term)?;

        // then
        assert_eq!(res.status, Status::Ok);
        assert_eq!(
            res.body,
            r#"{"entries":[{"filename":"doc1.pdf","thumbnail":"doc1.png"}]}"#
        );

        Ok(())
    }

    #[test]
    fn uploading_png_document_triggers_indexing() -> Result<()> {
        // given
        init_tracing();
        let mut app = test_app().with_tracked_repo()?.start()?;
        let search_term = "Parlamentarny";

        let res = app.search(search_term)?;
        assert_eq!(res.status, Status::Ok);
        assert_eq!(res.body, r#"{"entries":[]}"#);

        // when
        app.upload_doc(doc("doc1.png"))?;
        app.wait_til_indexed();

        // TODO: for some reason, only one word search is working - fix it
        let res = app.search(search_term)?;

        // then
        assert_eq!(res.status, Status::Ok);
        assert_eq!(
            res.body,
            r#"{"entries":[{"filename":"doc1.png","thumbnail":"doc1.png"}]}"#
        );

        Ok(())
    }

    #[test]
    fn uploading_document_without_extension_results_in_415_status_code() -> Result<()> {
        // given
        init_tracing();
        let app = test_app().with_tracked_repo()?.start()?;
        let wrong_doc = "no-extension-doc";

        // when
        let res = app.upload_doc(doc(wrong_doc))?;

        // then
        assert_eq!(res.status, Status::from_code(415).unwrap());
        assert_eq!(res.body, wrong_extension_msg(wrong_doc));

        Ok(())
    }

    fn wrong_extension_msg<S: Into<String>>(filename: S) -> String {
        let filename = filename.into();
        format!(
            "File '{}' has unsupported extension. Those are supported: [Png, Jpg, Webp, Pdf].",
            filename
        )
    }

    #[test]
    fn uploading_document_with_unsupported_extension_results_in_415_status_code() -> Result<()> {
        // given
        init_tracing();
        let app = test_app().with_tracked_repo()?.start()?;
        let wrong_doc = "unsupported-extension-doc.abc";

        // when
        let res = app.upload_doc(doc(wrong_doc))?;

        // then
        assert_eq!(res.status, Status::from_code(415).unwrap());
        assert_eq!(res.body, wrong_extension_msg(wrong_doc));

        Ok(())
    }

    #[test]
    fn fetching_not_existing_document_returns_404() -> Result<()> {
        // given
        init_tracing();
        let app = start_test_app()?;

        // when
        let res = app.get_doc("not-existing-doc")?;

        // then
        assert_eq!(res.status, Status::NotFound);

        Ok(())
    }

    #[test]
    fn when_fs_fails_to_load_document() -> Result<()> {
        // given
        init_tracing();
        let app = test_app().with_failing_load_fs()?.start()?;

        // when
        let res = app.get_doc("not-existing-doc")?;

        // then
        assert_eq!(res.status, Status::InternalServerError);

        Ok(())
    }
}
