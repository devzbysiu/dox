use crate::extractor::FilenameToBody;

use crate::result::Result;
use core::fmt;
use log::debug;
use rocket::serde::Serialize;
use std::path::Path;
use tantivy::collector::TopDocs;
use tantivy::query::{Query, QueryParser};
use tantivy::schema::{Field, Schema, Value, STORED, TEXT};
use tantivy::{doc, Index, LeasedItem, ReloadPolicy, Searcher};

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
    pub fn new(index: Index, schema: Schema) -> Self {
        Self { index, schema }
    }

    pub fn search<S: Into<String>>(&self, term: S) -> Result<SearchResults> {
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

pub fn mk_idx_and_schema<P: AsRef<Path>>(index_path: P) -> Result<(Index, Schema)> {
    let mut schema_builder = Schema::builder();
    schema_builder.add_text_field(&Fields::Filename.to_string(), TEXT | STORED);
    schema_builder.add_text_field(&Fields::Body.to_string(), TEXT);
    let schema = schema_builder.build();
    // FIXME: take care of a case when the index already exists
    let index = Index::create_in_dir(index_path, schema.clone())?;
    Ok((index, schema))
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
