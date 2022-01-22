use anyhow::Result;
use cooldown_buffer::cooldown_buffer;
use leptess::LepTess;
use log::{debug, error};
use notify::{watcher, DebouncedEvent, RecursiveMode, Watcher};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use std::path::{Path, PathBuf};
use std::sync::mpsc::channel;
use std::thread;
use std::time::Duration;

use tantivy::collector::TopDocs;
use tantivy::query::QueryParser;
use tantivy::schema::*;
use tantivy::{doc, Index, ReloadPolicy};

fn main() -> Result<()> {
    pretty_env_logger::init();

    let (doc_tx, doc_rx) = cooldown_buffer(Duration::from_secs(1));
    thread::spawn(move || -> Result<()> {
        let (tx, rx) = channel();

        let mut watcher = watcher(tx, Duration::from_millis(100))?;
        watcher.watch("/home/zbychu/tests/notify", RecursiveMode::Recursive)?;

        loop {
            match rx.recv() {
                Ok(DebouncedEvent::Create(path)) => doc_tx.send(path)?,
                Ok(_) => { /* not supported */ }
                Err(e) => error!("watch error: {:?}", e),
            }
        }
    });

    let index_path = dirs::data_dir().unwrap().join("dox");
    let mut schema_builder = Schema::builder();
    schema_builder.add_text_field("path", TEXT | STORED);
    schema_builder.add_text_field("body", TEXT);
    let schema = schema_builder.build();
    let index = Index::create_in_dir(&index_path, schema.clone())?;

    // loop {
    let paths = doc_rx.recv()?;
    let _paths = paths
        .par_iter()
        .map(extract_text)
        .filter_map(Result::ok)
        .map(|tuple| index_docs(tuple, index.clone(), schema.clone()))
        .filter_map(Result::ok)
        .collect::<Vec<&PathBuf>>();
    // }

    debug!("searching...");
    let reader = index
        .reader_builder()
        .reload_policy(ReloadPolicy::OnCommit)
        .try_into()?;
    let searcher = reader.searcher();
    let path = schema.get_field("path").unwrap();
    let body = schema.get_field("body").unwrap();
    let query_parser = QueryParser::for_index(&index, vec![path, body]);
    let query = query_parser.parse_query("komentarz dotyczący przebiegu akcji")?;
    let top_docs = searcher.search(&query, &TopDocs::with_limit(10))?;

    debug!("results:");
    for (_score, doc_address) in top_docs {
        let retrieved_doc = searcher.doc(doc_address)?;
        debug!("\t{}", schema.to_json(&retrieved_doc));
    }
    Ok(())
}

fn extract_text<P: AsRef<Path>>(p: P) -> Result<(P, String)> {
    let mut lt = LepTess::new(None, "pol")?;
    lt.set_image(p.as_ref())?;
    Ok((p, lt.get_utf8_text()?))
}

fn index_docs<P: AsRef<Path>>(
    (p, s): (P, String),
    index: Index,
    schema: Schema,
) -> tantivy::Result<P> {
    debug!("indexing {}", p.as_ref().display());
    let mut index_writer = index.writer(50_000_000)?;
    let path = schema.get_field("path").unwrap();
    let body = schema.get_field("body").unwrap();
    index_writer.add_document(doc!(path => p.as_ref().display().to_string(), body => s));
    index_writer.commit()?;
    Ok(p)
}
