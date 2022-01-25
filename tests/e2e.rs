use anyhow::Result;
use log::debug;
use rocket::serde::Deserialize;
use std::fs;
use std::process::Command;
use std::thread;
use std::time::Duration;

#[derive(Debug, Deserialize, Default)]
struct SearchResults {
    results: Vec<SearchEntry>,
}

#[derive(Debug, Deserialize, Default)]
struct SearchEntry {
    filename: String,
}

#[test]
fn it_works() -> Result<()> {
    pretty_env_logger::init();
    // given
    debug!("recreating index directory");
    let index_dir = dirs::data_dir().unwrap().join("dox");
    fs::remove_dir_all(&index_dir)?;
    fs::create_dir_all(&index_dir)?;

    debug!("recreating watched directory");
    let watched_dir = dirs::home_dir().unwrap().join("tests/notify");
    fs::remove_dir_all(&watched_dir)?;
    fs::create_dir_all(&watched_dir)?;

    debug!("spawning dox");
    let mut child = Command::new("./target/debug/dox").arg("&").spawn()?;
    thread::sleep(Duration::from_secs(2));

    let search: SearchResults = ureq::get("http://localhost:8000/search?q=ale")
        .call()?
        .into_json()?;

    debug!("initial results: {:?}", search);
    assert!(search.results.is_empty());

    debug!("copying docs to watched dir");
    let docs_dir = dirs::home_dir().unwrap().join("tests/scanned-docs");
    for file in fs::read_dir(docs_dir)? {
        let file = file?;
        debug!(
            "\tcopying {} to {}",
            file.path().display(),
            watched_dir.display()
        );
        fs::copy(file.path(), &watched_dir.join(file.file_name()))?;
    }
    thread::sleep(Duration::from_secs(10));

    let search: SearchResults = ureq::get("http://localhost:8000/search?q=ale")
        .call()?
        .into_json()?;
    debug!("results after indexing: {:?}", search);

    child.kill()?;
    assert_eq!(search.results.len(), 2);

    Ok(())
}
