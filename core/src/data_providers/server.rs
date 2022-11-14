use crate::entities::extension::supported_extensions;
use crate::entities::user::User;
use crate::helpers::PathRefExt;
use crate::result::{DocumentReadErr, DocumentSaveErr, SearchErr, ThumbnailReadErr};
use crate::use_cases::cipher::CipherRead;
use crate::use_cases::config::Config;
use crate::use_cases::fs::Fs;
use crate::use_cases::repository::{RepoRead, SearchResult};

use anyhow::Context;
use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::serde::Deserialize;
use rocket::{get, post, State};
use std::path::Path;
use tracing::instrument;

#[instrument(skip(repo))]
#[get("/search?<q>")]
pub fn search(
    user: User,
    q: String,
    repo: &State<RepoRead>,
) -> Result<Json<SearchResult>, SearchErr> {
    Ok(Json(repo.search(user, q).context("Searching failed.")?))
}

#[instrument(skip(fs, cipher_read))]
#[get("/thumbnail/<filename>")]
pub fn thumbnail(
    user: User,
    filename: String,
    cfg: &State<Config>,
    fs: &State<Fs>,
    cipher_read: &State<CipherRead>,
) -> Result<Option<Vec<u8>>, ThumbnailReadErr> {
    let thumbnail_path = cfg.thumbnails_dir.join(relative_path(&user, filename));
    let buf = fs.load(thumbnail_path)?;
    Ok(match buf {
        Some(buf) => Some(
            cipher_read
                .decrypt(&buf)
                .context("Failed to decrypt thumbnail.")?,
        ),
        None => None,
    })
}

#[instrument(skip(repo))]
#[get("/thumbnails/all")]
pub fn all_thumbnails(
    user: User,
    repo: &State<RepoRead>,
) -> Result<Json<SearchResult>, ThumbnailReadErr> {
    let all_docs = repo
        .all_documents(user)
        .context("Failed to retrieve all thumbnails.")?;
    Ok(Json(all_docs))
}

#[instrument(skip(fs, cipher_read))]
#[allow(clippy::needless_pass_by_value)] // rocket requires pass by value here
#[get("/document/<filename>")]
pub fn document(
    user: User,
    filename: String,
    cfg: &State<Config>,
    fs: &State<Fs>,
    cipher_read: &State<CipherRead>,
) -> Result<Option<Vec<u8>>, DocumentReadErr> {
    let document_path = cfg.watched_dir.join(relative_path(&user, filename));
    let buf = fs.load(document_path).context("Failed to read document.")?;
    Ok(match buf {
        Some(buf) => Some(
            cipher_read
                .decrypt(&buf)
                .context("Failed to decrypt document.")?,
        ),
        None => None,
    })
}

fn relative_path<S: Into<String>>(user: &User, filename: S) -> String {
    format!("{}/{}", base64::encode(&user.email), filename.into())
}

#[instrument(skip(doc, fs))]
#[allow(clippy::needless_pass_by_value)] // rocket requires pass by value here
#[post("/document/upload", data = "<doc>")]
pub fn receive_document(
    user: User,
    doc: Json<Document>,
    cfg: &State<Config>,
    fs: &State<Fs>,
) -> Result<(Status, String), DocumentSaveErr> {
    let filename = Path::new(&doc.filename);
    if !filename.has_supported_extension() {
        let msg = format!(
            "File '{}' has unsupported extension. Those are supported: {:?}",
            filename.display(),
            supported_extensions()
        );
        return Ok((Status::UnsupportedMediaType, msg));
    }
    let target_path = cfg.watched_dir.join(relative_path(&user, &doc.filename));
    let doc = base64::decode(&doc.body).context("Failed to decode body.")?;
    fs.save(target_path, &doc)
        .context("Failed to save document")?;
    Ok((Status::Created, String::new()))
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

        // when
        let res = app.upload_doc(doc("no-extension-doc"))?;

        // then
        assert_eq!(res.status, Status::from_code(415).unwrap());
        assert_eq!(
            res.body,
            r#"File 'no-extension-doc' has unsupported extension. Those are supported: [Png, Jpg, Webp, Pdf]"#
        );

        Ok(())
    }

    #[test]
    fn uploading_document_with_unsupported_extension_results_in_415_status_code() -> Result<()> {
        // given
        init_tracing();
        let app = test_app().with_tracked_repo()?.start()?;

        // when
        let res = app.upload_doc(doc("unsupported-extension-doc.abc"))?;

        // then
        assert_eq!(res.status, Status::from_code(415).unwrap());
        assert_eq!(
            res.body,
            r#"File 'unsupported-extension-doc.abc' has unsupported extension. Those are supported: [Png, Jpg, Webp, Pdf]"#
        );

        Ok(())
    }
}
