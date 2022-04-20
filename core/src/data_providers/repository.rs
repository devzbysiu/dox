use crate::configuration::cfg::Config;
use crate::entities::document::DocDetails;
use crate::result::{DoxErr, Result};
use crate::use_cases::repository::{Repository, SearchEntry, SearchResult};

use core::fmt;
use std::fmt::Debug;
use std::fs::create_dir_all;
use tantivy::collector::TopDocs;
use tantivy::directory::MmapDirectory;
use tantivy::query::{AllQuery, FuzzyTermQuery, Query};
use tantivy::schema::{Field, Schema, Value, STORED, TEXT};
use tantivy::{doc, DocAddress, Index, LeasedItem, ReloadPolicy, Term};
use tracing::{debug, instrument};

#[derive(Debug, Clone)]
pub struct TantivyRepository {
    index: Index,
    schema: Schema,
}

impl TantivyRepository {
    pub fn new(cfg: &Config) -> Result<Self> {
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
        schema_builder.add_text_field(&Fields::Thumbnail.to_string(), TEXT | STORED);
        let schema = schema_builder.build();
        let dir = MmapDirectory::open(&cfg.index_dir)?;
        let index = Index::open_or_create(dir, schema.clone())?;
        Ok(Self { index, schema })
    }

    fn create_searcher(&self) -> Result<Searcher> {
        Ok(self
            .index
            .reader_builder()
            .reload_policy(ReloadPolicy::OnCommit)
            .try_into()?
            .searcher())
    }

    fn make_query<S: Into<String>>(&self, term: S) -> Box<dyn Query> {
        let term = Term::from_field_text(self.field(&Fields::Body), &term.into());
        Box::new(FuzzyTermQuery::new(term, 2, true))
    }

    fn field(&self, field: &Fields) -> Field {
        // can unwrap because this field comes from an
        // enum and I'm using this enum to get the field
        self.schema.get_field(&field.to_string()).unwrap()
    }

    fn to_search_result(&self, searcher: &Searcher, docs: TantivyDocs) -> Result<SearchResult> {
        let mut results = Vec::new();
        for (_score, doc_address) in docs {
            let retrieved_doc = searcher.doc(doc_address)?;
            let filenames = retrieved_doc.get_all(self.field(&Fields::Filename));
            let thumbnails = retrieved_doc.get_all(self.field(&Fields::Thumbnail));
            results.extend(to_search_entries(filenames, thumbnails));
        }
        Ok(SearchResult::new(results))
    }
}

impl Repository for TantivyRepository {
    #[instrument(skip(self))]
    fn search(&self, term: String) -> Result<SearchResult> {
        let searcher = self.create_searcher()?;
        let top_docs = searcher.search(&self.make_query(term), &TopDocs::with_limit(100))?;
        self.to_search_result(&searcher, top_docs)
    }

    #[instrument(skip(self, docs_details))]
    fn index(&self, docs_details: Vec<DocDetails>) -> Result<()> {
        let index = &self.index;
        let schema = &self.schema;
        // NOTE: IndexWriter is already multithreaded and
        // cannot be shared between external threads
        let mut index_writer = index.writer(50_000_000)?;
        let filename = schema.get_field(&Fields::Filename.to_string()).unwrap();
        let body = schema.get_field(&Fields::Body.to_string()).unwrap();
        let thumbnail = schema.get_field(&Fields::Thumbnail.to_string()).unwrap();
        for doc_detail in docs_details {
            debug!("indexing {}", doc_detail.filename);
            index_writer.add_document(doc!(
                    filename => doc_detail.filename.clone(),
                    body => doc_detail.body.clone(),
                    thumbnail => doc_detail.thumbnail.clone(),
            ))?;
            index_writer.commit()?;
        }
        Ok(())
    }

    #[instrument(skip(self))]
    fn all_documents(&self) -> Result<SearchResult> {
        let searcher = self.create_searcher()?;
        let top_docs = searcher.search(&AllQuery, &TopDocs::with_limit(100))?;
        self.to_search_result(&searcher, top_docs)
    }
}

type Searcher = LeasedItem<tantivy::Searcher>;
type TantivyDocs = Vec<(f32, DocAddress)>;

enum Fields {
    Filename,
    Body,
    Thumbnail,
}

impl fmt::Display for Fields {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Fields::Filename => write!(f, "filename"),
            Fields::Body => write!(f, "body"),
            Fields::Thumbnail => write!(f, "thumbnail"),
        }
    }
}

pub fn to_search_entries<'a, V: Iterator<Item = &'a Value>>(
    filenames: V,
    thumbnails: V,
) -> Vec<SearchEntry> {
    filenames
        .zip(thumbnails)
        .map(to_text)
        .map(SearchEntry::new)
        .collect::<Vec<SearchEntry>>()
}

fn to_text((filename, thumbnail): (&Value, &Value)) -> (String, String) {
    (filename.text(), thumbnail.text())
}

trait ValueExt {
    fn text(&self) -> String;
}

impl ValueExt for Value {
    fn text(&self) -> String {
        self.as_text()
            .unwrap_or_else(|| panic!("failed to extract text"))
            .to_string()
    }
}
