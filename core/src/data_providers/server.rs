use crate::entities::user::User;
use crate::result::{DocumentReadErr, DocumentSaveErr, SearchErr, ThumbnailReadErr};
use crate::use_cases::cipher::CipherRead;
use crate::use_cases::config::Config;
use crate::use_cases::persistence::Persistence;
use crate::use_cases::repository::{RepoRead, SearchResult};

use anyhow::Context;
use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::serde::Deserialize;
use rocket::{get, post, State};
use tracing::instrument;

#[instrument(skip(repo_read))]
#[get("/search?<q>")]
pub fn search(
    user: User,
    q: String,
    repo_read: &State<RepoRead>,
) -> Result<Json<SearchResult>, SearchErr> {
    Ok(Json(
        repo_read
            .search(user, q)
            .context("Failed to search for query.")?,
    ))
}

#[instrument(skip(persistence, cipher_read))]
#[get("/thumbnail/<filename>")]
pub fn thumbnail(
    user: User,
    filename: String,
    cfg: &State<Config>,
    persistence: &State<Persistence>,
    cipher_read: &State<CipherRead>,
) -> Result<Option<Vec<u8>>, ThumbnailReadErr> {
    let thumbnail_path = cfg.thumbnails_dir.join(relative_path(&user, filename));
    let buf = persistence.load(thumbnail_path)?;
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

#[instrument(skip(persistence, cipher_read))]
#[allow(clippy::needless_pass_by_value)] // rocket requires pass by value here
#[get("/document/<filename>")]
pub fn document(
    user: User,
    filename: String,
    cfg: &State<Config>,
    persistence: &State<Persistence>,
    cipher_read: &State<CipherRead>,
) -> Result<Option<Vec<u8>>, DocumentReadErr> {
    let document_path = cfg.watched_dir.join(relative_path(&user, filename));
    let buf = persistence
        .load(document_path)
        .context("Failed to read document.")?;
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

#[instrument(skip(doc, persistence))]
#[allow(clippy::needless_pass_by_value)] // rocket requires pass by value here
#[post("/document/upload", data = "<doc>")]
pub fn receive_document(
    user: User,
    doc: Json<Document>,
    cfg: &State<Config>,
    persistence: &State<Persistence>,
) -> Result<Status, DocumentSaveErr> {
    let target_path = cfg.watched_dir.join(relative_path(&user, &doc.filename));
    let decoded_body = base64::decode(&doc.body).context("Failed to decode body.")?;
    persistence
        .save(target_path, &decoded_body)
        .context("Failed to save document")?;
    Ok(Status::Created)
}

#[derive(Debug, Deserialize)]
pub struct Document {
    filename: String,
    body: String,
}

// #[cfg(test)]
// mod test {
//     use crate::testingtools::create_test_app;

//     use anyhow::Result;
//     use rocket::http::Status;

//     #[test]
//     fn empty_index_returns_200_and_empty_json_entries() -> Result<()> {
//         // given
//         let app = create_test_app()?;

//         // when
//         let res = app.search("not-important")?;

//         // then
//         assert_eq!(res.status, Status::Ok);
//         assert_eq!(res.body, r#"{"entries":[]}"#);

//         Ok(())
//     }

//     #[test]
//     fn test_search_endpoint_with_indexed_docs() -> Result<()> {
//         // given
//         let (config, config_dir) = create_test_env()?;
//         let client = Client::tracked(rocket(Some(config_dir_string(&config_dir))))?;
//         thread::sleep(Duration::from_secs(5));
//         let user_dir_name = base64::encode(FAKE_USER_EMAIL); // TODO: it's repetition, think about this
//         cp_docs(config.watched_dir_path().join(user_dir_name))?;

//         // when
//         let (body, status) = client.read_entries("/search?q=Parlamentarny")?;

//         // then
//         assert_eq!(status, Status::Ok);
//         assert_eq!(
//             body,
//             r#"{"entries":[{"filename":"doc1.png","thumbnail":"doc1.png"}]}"#
//         );

//         Ok(())
//     }

//     #[test]
//     fn test_search_endpoint_with_wrong_query() -> Result<()> {
//         // given
//         let (config, config_dir) = create_test_env()?;
//         let client = Client::tracked(rocket(Some(config_dir_string(&config_dir))))?;
//         thread::sleep(Duration::from_secs(5));
//         let user_dir_name = base64::encode(FAKE_USER_EMAIL);
//         cp_docs(config.watched_dir_path().join(user_dir_name))?;

//         // when
//         let mut resp = client.get("/search?q=not-existing-query").dispatch();
//         let body = resp.read_body()?;

//         // then
//         assert_eq!(resp.status(), Status::Ok);
//         assert_eq!(body, r#"{"entries":[]}"#);

//         Ok(())
//     }

//     #[test]
//     fn test_all_thumbnails_endpoint_with_empty_index() -> Result<()> {
//         // given
//         let (_, config_dir) = create_test_env()?;
//         let client = Client::tracked(rocket(Some(config_dir_string(&config_dir))))?;

//         // when
//         let mut resp = client.get("/thumbnails/all").dispatch();
//         let body = resp.read_body()?;

//         // then
//         assert_eq!(resp.status(), Status::Ok);
//         assert_eq!(body, r#"{"entries":[]}"#);

//         Ok(())
//     }

//     #[test]
//     fn test_all_thumbnails_endpoint_with_indexed_docs() -> Result<()> {
//         // given
//         let (config, config_dir) = create_test_env()?;
//         let client = Client::tracked(rocket(Some(config_dir_string(&config_dir))))?;
//         thread::sleep(Duration::from_secs(5));
//         let user_dir_name = base64::encode(FAKE_USER_EMAIL);
//         cp_docs(config.watched_dir_path().join(user_dir_name))?;

//         // when
//         let (body, status) = client.read_entries("/thumbnails/all")?;

//         // then
//         assert_eq!(status, Status::Ok);
//         assert_eq!(
//             body,
//             r#"{"entries":[{"filename":"doc1.png","thumbnail":"doc1.png"}]}"#
//         );

//         Ok(())
//     }

//     #[test]
//     fn test_receive_document_endpoint() -> Result<()> {
//         // given
//         let (_, config_dir) = create_test_env()?;
//         let client = Client::tracked(rocket(Some(config_dir_string(&config_dir))))?;

//         let mut resp = client.get("/search?q=Parlamentarny").dispatch();
//         let body = resp.read_body()?;
//         assert_eq!(resp.status(), Status::Ok);
//         assert_eq!(body, r#"{"entries":[]}"#);

//         // when
//         let resp = client
//             .post("/document/upload")
//             .body(format!(
//                 r#"{{ "filename": "doc1.png", "body": "{}" }}"#,
//                 to_base64("res/doc1.png")?
//             ))
//             .dispatch();
//         assert_eq!(resp.status(), Status::Created);

//         thread::sleep(Duration::from_secs(15)); // allow to index docs

//         let (body, status) = client.read_entries("/search?q=Parlamentarny")?;

//         // then
//         assert_eq!(status, Status::Ok);
//         assert_eq!(
//             body,
//             r#"{"entries":[{"filename":"doc1.png","thumbnail":"doc1.png"}]}"#
//         );

//         Ok(())
//     }
// }
