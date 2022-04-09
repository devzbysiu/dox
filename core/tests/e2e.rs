use anyhow::Result;
use serial_test::serial;
use testutils::{config_path, cp_docs, create_test_env, ls, make_search, spawn_dox, SearchEntry};

#[test]
#[serial]
fn it_allows_to_search_through_api() -> Result<()> {
    pretty_env_logger::init();
    // given
    let (config, config_dir) = create_test_env()?;

    // NOTE: Drop trait causes this `_dox_process` to be killed on drop (even when the test fails)
    let _dox_process = spawn_dox(config_path(&config_dir))?;

    let search = make_search("ale")?;

    assert!(search.entries.is_empty()); // initial search returns no results

    // when
    // TODO: test should add documents via API
    cp_docs(config.watched_dir_path())?; // then we copy documents and indexing starts

    // then
    let thumbnails = ls(config.thumbnails_dir_path())?;
    assert_eq!(thumbnails, vec!["doc1.png"]);

    let results = make_search("ale")?;

    let mut entries = results.entries;
    assert_eq!(entries.len(), 1); // then we have two results
    entries.sort_by(|a, b| a.filename.cmp(&b.filename));
    assert_eq!(
        entries,
        vec![SearchEntry {
            filename: "doc1.png".to_string()
        }]
    );

    Ok(())
}
