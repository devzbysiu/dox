// TODO: Take care of those tests
// use std::thread;
// use std::time::Duration;

// use anyhow::Result;
// use testutils::{config_path, cp_docs, create_test_app, ls, make_search, spawn_dox, SearchEntry};

// #[test]
// fn it_allows_to_search_through_api() -> Result<()> {
//     // given
//     let (config, config_dir) = create_test_app()?;

//     let user_dir_name = base64::encode("bartosz.zbytniewski@gmail.com");
//     let user_dir = config.watched_dir_path().join(&user_dir_name);

//     // NOTE: Drop trait causes this `_dox_process` to be killed on drop (even when the test fails)
//     let _dox_process = spawn_dox(config_path(&config_dir))?;

//     let search = make_search("ale")?;

//     assert!(search.entries.is_empty()); // initial search returns no results

//     // when
//     // TODO: test should add documents via API
//     cp_docs(user_dir)?; // then we copy documents and indexing starts
//     thread::sleep(Duration::from_secs(30));

//     // then
//     let top_dir_contents = ls(config.thumbnails_dir_path())?;
//     assert_eq!(top_dir_contents, vec![user_dir_name.clone()]);
//     let user_dir_contents = ls(config.thumbnails_dir_path().join(user_dir_name))?;
//     assert_eq!(user_dir_contents, vec!["doc1.png"]);

//     let results = make_search("ale")?;

//     let mut entries = results.entries;
//     assert_eq!(entries.len(), 1); // then we have two results
//     entries.sort_by(|a, b| a.filename.cmp(&b.filename));
//     assert_eq!(
//         entries,
//         vec![SearchEntry {
//             filename: "doc1.png".to_string()
//         }]
//     );

//     Ok(())
// }
//

// #![allow(clippy::missing_errors_doc)]

// use anyhow::{bail, Result};
// use rand::Rng;
// use rocket::serde::Deserialize;
// use std::env;
// use std::fs::{self, create_dir_all, File};
// use std::io::Read;
// use std::net::SocketAddrV4;
// use std::path::Path;
// use std::process::{Child, Command};
// use std::thread;
// use std::time::Duration;
// use tracing::debug;

// #[derive(Debug, Deserialize, Default)]
// pub struct SearchResults {
//     pub entries: Vec<SearchEntry>,
// }

// #[derive(Debug, Deserialize, Default, PartialEq, Eq)]
// pub struct SearchEntry {
//     pub filename: String,
// }

// pub struct DoxProcess(Child);

// impl Drop for DoxProcess {
//     fn drop(&mut self) {
//         self.0.kill().expect("failed to kill dox process");
//     }
// }

// pub fn spawn_dox<P: AsRef<Path>>(config_path: P) -> Result<DoxProcess> {
//     debug!("spawning 'dox {} &'", config_path.as_ref().display());
//     let child = Command::new("./target/debug/dox")
//         .arg(format!("{}", config_path.as_ref().display()))
//         .arg("&")
//         .spawn()?;
//     thread::sleep(Duration::from_secs(2));
//     Ok(DoxProcess(child))
// }

// pub fn make_search<S: Into<String>>(query: S) -> Result<SearchResults> {
//     let url = format!("http://localhost:8000/search?q={}", query.into());
//     let res = ureq::get(&url)
//         .set("authorization", &id_token()?)
//         .call()?
//         .into_json()?;
//     debug!("search results: {:?}", res);
//     Ok(res)
// }

// fn id_token() -> Result<String> {
//     let res: IdToken = ureq::post("https://www.googleapis.com/oauth2/v4/token")
//         .send_json(ureq::json!({
//                 "grant_type": "refresh_token",
//                 "client_id": env!("DOX_CLIENT_ID"),
//                 "client_secret": env!("DOX_CLIENT_SECRET"),
//                 "refresh_token": env!("DOX_REFRESH_TOKEN"),

//         }))?
//         .into_json()?;
//     Ok(res.id_token)
// }

// #[derive(Default, Deserialize)]
// struct IdToken {
//     id_token: String,
// }

// pub fn cp_docs<P: AsRef<Path>>(parent_dir: P) -> Result<()> {
//     debug!("copying docs to watched dir...");
//     let parent_dir = parent_dir.as_ref();
//     let from = "./res/doc1.png";
//     let to = parent_dir.join("doc1.png");
//     create_dir_all(to.parent().expect("failed to get parent"))?;
//     thread::sleep(Duration::from_secs(1)); // allow to start listening for events on this new dir
//     debug!("\tfrom {} to {}", from, to.display());
//     fs::copy(from, to)?; // TODO: it should be just one file
//     debug!("done");
//     Ok(())
// }

// pub fn ls<P: AsRef<Path>>(dir: P) -> Result<Vec<String>> {
//     let dir = dir.as_ref();
//     if !dir.is_dir() {
//         bail!("I can list only directories");
//     }
//     let mut result = Vec::new();
//     for path in dir.read_dir()? {
//         let path = path?;
//         result.push(path.file_name().to_str().unwrap().to_string());
//     }
//     result.sort();
//     Ok(result)
// }

// pub fn random_addr() -> SocketAddrV4 {
//     let mut rng = rand::thread_rng();
//     let port = rng.gen_range(8000..9000);
//     format!("0.0.0.0:{}", port).parse().unwrap()
// }

// pub fn to_base64<P: AsRef<Path>>(path: P) -> Result<String> {
//     let mut file = File::open(path)?;
//     let mut buff = Vec::new();
//     file.read_to_end(&mut buff)?;
//     Ok(base64::encode(&buff))
// }
