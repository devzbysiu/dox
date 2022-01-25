use anyhow::Result;
use log::debug;
use rocket::serde::Deserialize;
use std::fs;
use std::process::{Child, Command};
use std::thread;
use std::time::Duration;

#[derive(Debug, Deserialize, Default)]
struct SearchResults {
    results: Vec<SearchEntry>,
}

#[derive(Debug, Deserialize, Default, PartialEq, Eq)]
struct SearchEntry {
    filename: String,
}

#[test]
fn it_works() -> Result<()> {
    pretty_env_logger::init();
    // given
    recreate_index_dir()?;
    recreate_watched_dir()?;

    // when
    let mut child = spawn_dox()?;

    // then
    let search = make_search("ale")?;

    assert!(search.results.is_empty());

    initiate_indexing()?;

    let search = make_search("ale")?;

    let mut results = search.results;
    assert_eq!(results.len(), 2);
    results.sort_by(|a, b| a.filename.cmp(&b.filename));
    assert_eq!(
        results,
        vec![
            SearchEntry {
                filename: "doc1.png".to_string()
            },
            SearchEntry {
                filename: "doc5.png".to_string()
            },
        ]
    );

    child.kill()?;
    Ok(())
}

fn recreate_index_dir() -> Result<()> {
    debug!("recreating index directory");
    let index_dir = dirs::data_dir().unwrap().join("dox");
    fs::remove_dir_all(&index_dir)?;
    fs::create_dir_all(&index_dir)?;
    Ok(())
}

fn recreate_watched_dir() -> Result<()> {
    debug!("recreating watched directory");
    let watched_dir = dirs::home_dir().unwrap().join("tests/notify");
    fs::remove_dir_all(&watched_dir)?;
    fs::create_dir_all(&watched_dir)?;
    Ok(())
}

fn spawn_dox() -> Result<Child> {
    debug!("spawning dox");
    let child = Command::new("./target/debug/dox").arg("&").spawn()?;
    thread::sleep(Duration::from_secs(2));
    Ok(child)
}

fn make_search<S: Into<String>>(query: S) -> Result<SearchResults> {
    let url = format!("http://localhost:8000/search?q={}", query.into());
    let res = ureq::get(&url).call()?.into_json()?;
    debug!("search results: {:?}", res);
    Ok(res)
}

fn initiate_indexing() -> Result<()> {
    debug!("copying docs to watched dir");
    let docs_dir = dirs::home_dir().unwrap().join("tests/scanned-docs");
    let watched_dir = dirs::home_dir().unwrap().join("tests/notify");
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
    Ok(())
}
