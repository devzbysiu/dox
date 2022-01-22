use anyhow::Result;
use cooldown_buffer::cooldown_buffer;
use core::fmt;
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

struct IndexTuple {
    filename: String,
    body: String,
}

enum Fields {
    Filename,
    Body,
}

impl fmt::Display for Fields {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Fields::Filename => write!(f, "filename"),
            Fields::Body => write!(f, "body"),
        }
    }
}

impl IndexTuple {
    fn new<S: Into<String>, A: AsRef<Path>>(path: A, body: S) -> Self {
        let filename = path
            .as_ref()
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();
        let body = body.into();
        Self { filename, body }
    }
}

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

    let (index, schema) = mk_idx_and_schema("dox")?;

    let (thread_idx, thread_schema) = (index.clone(), schema.clone());
    thread::spawn(move || -> Result<()> {
        loop {
            let paths = doc_rx.recv()?;
            debug!("new docs: {:?}", paths);
            let tuples = extract_text(&paths);
            index_docs(&tuples, &thread_idx, &thread_schema)?;
        }
    });
    thread::sleep(Duration::from_secs(10));

    debug!("searching...");
    let reader = index
        .reader_builder()
        .reload_policy(ReloadPolicy::OnCommit)
        .try_into()?;
    let searcher = reader.searcher();
    let filename = schema.get_field(&Fields::Filename.to_string()).unwrap();
    let body = schema.get_field(&Fields::Body.to_string()).unwrap();
    let query_parser = QueryParser::for_index(&index, vec![filename, body]);
    let query = query_parser.parse_query("komentarz dotyczący przebiegu akcji")?;
    let top_docs = searcher.search(&query, &TopDocs::with_limit(10))?;

    debug!("results:");
    for (_score, doc_address) in top_docs {
        let retrieved_doc = searcher.doc(doc_address)?;
        debug!("\t{}", schema.to_json(&retrieved_doc));
    }
    Ok(())
}

fn mk_idx_and_schema<A: AsRef<Path>>(relative_path: A) -> Result<(Index, Schema)> {
    let index_path = dirs::data_dir().unwrap().join(relative_path);
    let mut schema_builder = Schema::builder();
    schema_builder.add_text_field(&Fields::Filename.to_string(), TEXT | STORED);
    schema_builder.add_text_field(&Fields::Body.to_string(), TEXT);
    let schema = schema_builder.build();
    let index = Index::create_in_dir(&index_path, schema.clone())?;
    Ok((index, schema))
}

fn extract_text(paths: &[PathBuf]) -> Vec<IndexTuple> {
    debug!("extracting text...");
    paths
        .par_iter()
        .map(make_tuple)
        .filter_map(Result::ok)
        .collect::<Vec<IndexTuple>>()
}

fn make_tuple<P: AsRef<Path>>(path: P) -> Result<IndexTuple> {
    // it's actually more efficient to create LepTess
    // each time than sharing it between threads
    let mut lt = LepTess::new(None, "pol")?;
    lt.set_image(path.as_ref())?;
    Ok(IndexTuple::new(path, lt.get_utf8_text()?))
}

fn index_docs(tuples: &[IndexTuple], index: &Index, schema: &Schema) -> tantivy::Result<()> {
    debug!("indexing...");
    let mut index_writer = index.writer(50_000_000)?;
    let filename = schema.get_field(&Fields::Filename.to_string()).unwrap();
    let body = schema.get_field(&Fields::Body.to_string()).unwrap();
    tuples.iter().for_each(|tuple| {
        debug!("indexing {}", tuple.filename);
        index_writer.add_document(doc!(
            filename => tuple.filename.clone(),
            body => tuple.body.clone()
        ));
        index_writer.commit().unwrap();
    });
    Ok(())
}

#[allow(dead_code)] // TODO: for testing purposes only
fn test_files() -> Vec<PathBuf> {
    vec![
        Path::new("/home/zbychu/tests/scanned-docs/doc1.png").to_path_buf(),
        Path::new("/home/zbychu/tests/scanned-docs/doc2.jpg").to_path_buf(),
        Path::new("/home/zbychu/tests/scanned-docs/doc3.jpg").to_path_buf(),
        Path::new("/home/zbychu/tests/scanned-docs/doc4.webp").to_path_buf(),
        Path::new("/home/zbychu/tests/scanned-docs/doc5.png").to_path_buf(),
        Path::new("/home/zbychu/tests/scanned-docs/doc6.jpg").to_path_buf(),
        Path::new("/home/zbychu/tests/scanned-docs/doc7.jpg").to_path_buf(),
        Path::new("/home/zbychu/tests/scanned-docs/doc8.webp").to_path_buf(),
    ]
}
