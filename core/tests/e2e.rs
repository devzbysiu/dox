use anyhow::Result;
use std::time::Duration;
use testutils::{
    config_path, cp_docs, create_cfg_file, create_index_dir, create_thumbnails_dir,
    create_watched_dir, ls, make_search, spawn_dox, TestConfig, SearchEntry,
};

#[test]
fn it_allows_to_search_through_api() -> Result<()> {
    pretty_env_logger::init();
    // given
    let index_dir = create_index_dir()?;
    let watched_dir = create_watched_dir()?;
    let thumbnails_dir = create_thumbnails_dir()?;
    let config_dir = create_cfg_file(&TestConfig {
        watched_dir: watched_dir.path().to_path_buf(),
        thumbnails_dir: thumbnails_dir.path().to_path_buf(),
        index_dir: index_dir.path().to_path_buf(),
        cooldown_time: Duration::from_secs(1),
    })?;

    // NOTE: Drop trait causes this `_dox_process` to be killed on drop (even when the test fails)
    let _dox_process = spawn_dox(config_path(&config_dir))?;

    let search = make_search("ale")?;

    assert!(search.entries.is_empty()); // initial search returns no results

    // when
    // TODO: test should add documents via API
    cp_docs(watched_dir.path())?; // then we copy documents and indexing starts

    // then
    let thumbnails = ls(thumbnails_dir)?;
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
