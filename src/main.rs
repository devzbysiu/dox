#![allow(clippy::no_effect_underscore_binding)] // needed because of how rocket macros work

use anyhow::{Error, Result};
use cooldown_buffer::cooldown_buffer;
use core::fmt;
use leptess::LepTess;
use log::{debug, error};
use notify::{watcher, DebouncedEvent, RecursiveMode, Watcher};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use rocket::response::Debug;
use rocket::serde::json::Json;
use rocket::serde::Serialize;
use rocket::{get, launch, routes, Build, Rocket, State};
use std::path::{Path, PathBuf};
use std::sync::mpsc::channel;
use std::thread;
use std::time::Duration;
use tantivy::collector::TopDocs;
use tantivy::query::{Query, QueryParser};
use tantivy::schema::{Field, Schema, Value, STORED, TEXT};
use tantivy::{doc, Index, LeasedItem, ReloadPolicy, Searcher};

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

struct Repo {
    index: Index,
    schema: Schema,
}

impl Repo {
    fn new(index: Index, schema: Schema) -> Self {
        Self { index, schema }
    }

    fn search<S: Into<String>>(&self, term: S) -> Result<SearchResults> {
        let term = term.into();
        debug!("searching '{}'...", term);
        let searcher = self.create_searcher()?;
        let top_docs = searcher.search(&self.make_query(term)?, &TopDocs::with_limit(10))?;
        let mut results = Vec::new();
        for (_score, doc_address) in top_docs {
            let retrieved_doc = searcher.doc(doc_address)?;
            let filenames = retrieved_doc.get_all(self.field(&Fields::Filename));
            results.extend(to_search_entries(filenames));
        }
        debug!("results: {:?}", results);
        Ok(SearchResults::new(results))
    }

    fn create_searcher(&self) -> Result<LeasedItem<Searcher>> {
        Ok(self
            .index
            .reader_builder()
            .reload_policy(ReloadPolicy::OnCommit)
            .try_into()?
            .searcher())
    }

    fn make_query<S: Into<String>>(&self, term: S) -> Result<Box<dyn Query>> {
        let parser = QueryParser::for_index(
            &self.index,
            vec![self.field(&Fields::Filename), self.field(&Fields::Body)],
        );
        Ok(parser.parse_query(&term.into())?)
    }

    fn field(&self, field: &Fields) -> Field {
        // can unwrap because this field comes from an
        // enum and I'm using this enym to get the field
        self.schema.get_field(&field.to_string()).unwrap()
    }
}

fn to_search_entries<'a, I: Iterator<Item = &'a Value>>(filenames: I) -> Vec<SearchEntry> {
    filenames
        .filter_map(Value::text)
        .map(ToString::to_string)
        .map(SearchEntry::new)
        .collect::<Vec<SearchEntry>>()
}

#[derive(Debug, Serialize, Default)]
struct SearchResults {
    entries: Vec<SearchEntry>,
}

impl SearchResults {
    fn new(entries: Vec<SearchEntry>) -> Self {
        Self { entries }
    }
}

#[derive(Debug, Serialize, Default)]
struct SearchEntry {
    filename: String,
}

impl SearchEntry {
    fn new(filename: String) -> Self {
        Self { filename }
    }
}

#[launch]
fn launch() -> Rocket<Build> {
    pretty_env_logger::init();

    let repo = setup().expect("failed to setup indexer");
    debug!("starting server...");
    rocket::build().mount("/", routes![search]).manage(repo)
}

fn setup() -> Result<Repo> {
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
    Ok(Repo::new(index, schema))
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
        .map(do_ocr)
        .filter_map(Result::ok)
        .collect::<Vec<IndexTuple>>()
}

fn do_ocr<P: AsRef<Path>>(path: P) -> Result<IndexTuple> {
    debug!("executing OCR on {}", path.as_ref().display());
    // NOTE: it's actually more efficient to create LepTess
    // each time than sharing it between threads
    let mut lt = LepTess::new(None, "pol")?;
    lt.set_image(path.as_ref())?;
    Ok(IndexTuple::new(path, lt.get_utf8_text()?))
}

fn index_docs(tuples: &[IndexTuple], index: &Index, schema: &Schema) -> tantivy::Result<()> {
    debug!("indexing...");
    // NOTE: IndexWriter is already multithreaded and
    // cannot be shared between external threads
    let mut index_writer = index.writer(50_000_000)?;
    let filename = schema.get_field(&Fields::Filename.to_string()).unwrap();
    let body = schema.get_field(&Fields::Body.to_string()).unwrap();
    for t in tuples {
        debug!("indexing {}", t.filename);
        index_writer.add_document(doc!(filename => t.filename.clone(), body => t.body.clone()));
        index_writer.commit().unwrap();
    }
    Ok(())
}

#[get("/search?<q>")]
fn search(q: String, repo: &State<Repo>) -> Result<Json<SearchResults>, Debug<Error>> {
    Ok(Json(repo.search(q)?))
}
