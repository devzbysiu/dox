use crate::cfg::Config;
use crate::extractor::FilenameToBody;
use crate::result::{DoxErr, Result};

use core::fmt;
use log::debug;
use rocket::serde::Serialize;
use std::fs::create_dir_all;
use tantivy::collector::TopDocs;
use tantivy::query::{AllQuery, Query, QueryParser};
use tantivy::schema::{Field, Schema, Value, STORED, TEXT};
use tantivy::{doc, DocAddress, Index, LeasedItem, ReloadPolicy};

type Searcher = LeasedItem<tantivy::Searcher>;

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

pub struct Repo {
    index: Index,
    schema: Schema,
}

impl Repo {
    pub fn new(repo_tools: RepoTools) -> Self {
        Self {
            index: repo_tools.index,
            schema: repo_tools.schema,
        }
    }

    pub fn search<S: Into<String>>(&self, term: S) -> Result<SearchResults> {
        let term = term.into();
        debug!("searching '{}'...", term);
        let searcher = self.create_searcher()?;
        let top_docs = searcher.search(&self.make_query(term)?, &TopDocs::with_limit(100))?;
        self.to_search_results(searcher, top_docs)
    }

    fn create_searcher(&self) -> Result<Searcher> {
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

    fn to_search_results(
        &self,
        searcher: Searcher,
        docs: Vec<(f32, DocAddress)>,
    ) -> Result<SearchResults> {
        let mut results = Vec::new();
        for (_score, doc_address) in docs {
            let retrieved_doc = searcher.doc(doc_address)?;
            let filenames = retrieved_doc.get_all(self.field(&Fields::Filename));
            results.extend(to_search_entries(filenames));
        }
        Ok(SearchResults::new(results))
    }

    pub fn all_documents(&self) -> Result<SearchResults> {
        debug!("fetching all documents...");
        let searcher = self.create_searcher()?;
        let top_docs = searcher.search(&AllQuery, &TopDocs::with_limit(100))?;
        self.to_search_results(searcher, top_docs)
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
pub struct SearchResults {
    entries: Vec<SearchEntry>,
}

impl SearchResults {
    pub fn new(entries: Vec<SearchEntry>) -> Self {
        Self { entries }
    }
}

#[derive(Debug, Serialize, Default)]
pub struct SearchEntry {
    filename: String,
}

impl SearchEntry {
    pub fn new(filename: String) -> Self {
        Self { filename }
    }
}

pub fn mk_idx_and_schema(cfg: &Config) -> Result<RepoTools> {
    debug!("creating index under path: {}", cfg.index_dir.display());
    if cfg.index_dir.exists() && cfg.index_dir.is_file() {
        return Err(DoxErr::InvalidIndexPath(format!(
            "It needs to be a directory: '{}'",
            cfg.index_dir.display()
        )));
    }
    create_dir_all(&cfg.index_dir)?;
    let mut schema_builder = Schema::builder();
    schema_builder.add_text_field(&Fields::Filename.to_string(), TEXT | STORED);
    schema_builder.add_text_field(&Fields::Body.to_string(), TEXT);
    let schema = schema_builder.build();
    // FIXME: take care of a case when the index already exists
    let index = Index::create_in_dir(&cfg.index_dir, schema.clone())?;
    Ok(RepoTools { index, schema })
}

#[derive(Debug, Clone)]
pub struct RepoTools {
    pub index: Index,
    pub schema: Schema,
}

#[allow(clippy::module_name_repetitions)]
pub fn index_docs(tuples: &[FilenameToBody], index: &Index, schema: &Schema) -> Result<()> {
    debug!("indexing...");
    // NOTE: IndexWriter is already multithreaded and
    // cannot be shared between external threads
    let mut index_writer = index.writer(50_000_000)?;
    let filename = schema.get_field(&Fields::Filename.to_string()).unwrap();
    let body = schema.get_field(&Fields::Body.to_string()).unwrap();
    for t in tuples {
        debug!("indexing {}", t.filename);
        index_writer.add_document(doc!(filename => t.filename.clone(), body => t.body.clone()));
        index_writer.commit()?;
    }
    Ok(())
}
