//! This is concrete implementation of [`crate::use_cases::repository`] abstractions.
//!
//! It uses [`tantivy`] as full text search library.
use crate::data_providers::server::User;
use crate::entities::document::DocDetails;
use crate::result::{DoxErr, Result};
use crate::use_cases::config::Config;
use crate::use_cases::repository::{
    RepoRead, RepoWrite, RepositoryRead, RepositoryWrite, SearchEntry, SearchResult,
};

use core::fmt;
use std::collections::HashMap;
use std::fmt::Debug;
use std::fs::create_dir_all;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use tantivy::collector::TopDocs;
use tantivy::directory::MmapDirectory;
use tantivy::query::{AllQuery, FuzzyTermQuery, Query};
use tantivy::schema::{Field, Schema, Value, STORED, TEXT};
use tantivy::{doc, DocAddress, Index, LeasedItem, ReloadPolicy, Term};
use tracing::{debug, instrument};

#[derive(Debug, Clone)]
pub struct TantivyRepository;

impl TantivyRepository {
    pub fn create(cfg: &Config) -> Result<(RepoRead, RepoWrite)> {
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
        let indexes = Arc::new(RwLock::new(HashMap::new()));
        Ok((
            Box::new(TantivyRead::new(indexes.clone(), schema.clone())),
            Box::new(TantivyWrite::new(cfg.index_dir.clone(), indexes, schema)),
        ))
    }
}

#[derive(Debug, Clone)]
struct TantivyRead {
    indexes: Arc<RwLock<HashMap<User, Index>>>,
    schema: Schema,
}

impl TantivyRead {
    fn new(indexes: Arc<RwLock<HashMap<User, Index>>>, schema: Schema) -> Self {
        Self { indexes, schema }
    }

    fn create_searcher(&self, user: User) -> Result<Searcher> {
        Ok(self
            .indexes
            .read()
            .expect("poisoned lock")
            .get(&user)
            .ok_or(DoxErr::MissingIndex(user.email))?
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
        Ok(SearchResult::from_vec(results))
    }
}

impl RepositoryRead for TantivyRead {
    #[instrument(skip(self))]
    fn search(&self, user: User, term: String) -> Result<SearchResult> {
        let searcher = self.create_searcher(user);
        if let Err(DoxErr::MissingIndex(email)) = searcher {
            debug!("No index for user: '{}'", email);
            return Ok(SearchResult::default());
        }
        let searcher = searcher.unwrap(); // can unwrap because it's checked above
        let top_docs = searcher.search(&self.make_query(term), &TopDocs::with_limit(100))?;
        Ok(self.to_search_result(&searcher, top_docs)?)
    }

    #[instrument(skip(self))]
    fn all_documents(&self, user: User) -> Result<SearchResult> {
        // TODO: get rid of this duplication
        let searcher = self.create_searcher(user);
        if let Err(DoxErr::MissingIndex(email)) = searcher {
            debug!("No index for user: '{}'", email);
            return Ok(SearchResult::default());
        }
        let searcher = searcher.unwrap(); // can unwrap beause it's checked above
        let top_docs = searcher.search(&AllQuery, &TopDocs::with_limit(100))?;
        Ok(self.to_search_result(&searcher, top_docs)?)
    }
}

#[derive(Debug, Clone)]
struct TantivyWrite {
    indexes: Arc<RwLock<HashMap<User, Index>>>,
    idx_root: PathBuf,
    schema: Schema,
}

impl TantivyWrite {
    fn new(idx_root: PathBuf, indexes: Arc<RwLock<HashMap<User, Index>>>, schema: Schema) -> Self {
        Self {
            idx_root,
            indexes,
            schema,
        }
    }
}

impl RepositoryWrite for TantivyWrite {
    #[instrument(skip(self, docs_details))]
    fn index(&self, user: User, docs_details: &[DocDetails]) -> Result<()> {
        {
            let mut indexes = self.indexes.write().expect("poisoned rw lock");
            if !indexes.contains_key(&user) {
                let idx_dir = self.idx_root.join(base64::encode(&user.email));
                debug!(
                    "creating new index directory for '{}' under path '{}'",
                    user.email,
                    idx_dir.display()
                );
                create_dir_all(&idx_dir)?;
                let dir = MmapDirectory::open(&idx_dir)?;
                let index = Index::open_or_create(dir, self.schema.clone())?;
                debug!("adding newly created index to indexes map");
                indexes.insert(user.clone(), index);
            }
        }
        let indexes = self.indexes.read().expect("poisoned rw lock");
        let index = indexes.get(&user).unwrap(); // can unwrap because it's added above
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

#[cfg(test)]
mod test {
    use super::*;
    use anyhow::Result;
    use std::fs::File;
    use std::time::Duration;
    use testutils::{index_dir_path, thumbnails_dir_path, watched_dir_path};

    #[test]
    fn test_mk_index_and_schema_when_index_dir_is_taken_by_file() -> Result<()> {
        // given
        let config = create_config()?;
        File::create(&config.index_dir)?;

        // when
        let result = TantivyRepository::create(&config);

        // then
        assert_eq!(
            result.err().unwrap().to_string(),
            format!(
                "Invalid index path: 'It needs to be a directory: '{}''",
                config.index_dir.display()
            )
        );
        Ok(())
    }

    fn create_config() -> Result<Config> {
        // NOTE: TempDir is removed on the end of this fn call,
        // but paths are randomized so it's still useful
        let index_dir = index_dir_path()?;
        let watched_dir = watched_dir_path()?;
        let thumbnails_dir = thumbnails_dir_path()?;
        Ok(Config {
            watched_dir: watched_dir.path().to_path_buf(),
            thumbnails_dir: thumbnails_dir.path().to_path_buf(),
            index_dir: index_dir.path().to_path_buf(),
            cooldown_time: Duration::from_secs(1),
            notifications_addr: "0.0.0.0:8001".parse()?,
            websocket_cleanup_time: Duration::from_secs(10),
        })
    }

    #[test]
    fn test_index_docs() -> Result<()> {
        // given
        let config = create_config()?;
        let (repo_read, repo_write) = TantivyRepository::create(&config)?;
        let tuples_to_index = vec![
            DocDetails::new("filename1", "body1", "thumbnail1"),
            DocDetails::new("filename2", "body2", "thumbnail2"),
            DocDetails::new("filename3", "body3", "thumbnail3"),
            DocDetails::new("filename4", "body4", "thumbnail4"),
            DocDetails::new("filename5", "body5", "thumbnail5"),
        ];

        // when
        repo_write.index(&tuples_to_index)?;
        // TODO: this test should check only indexing but it's also
        // searching via all_documents
        let all_docs = repo_read.all_documents()?;

        // then
        assert_eq!(
            all_docs,
            SearchResult::from_vec(vec![
                SearchEntry::new(("filename1".into(), "thumbnail1".into())),
                SearchEntry::new(("filename2".into(), "thumbnail2".into())),
                SearchEntry::new(("filename3".into(), "thumbnail3".into())),
                SearchEntry::new(("filename4".into(), "thumbnail4".into())),
                SearchEntry::new(("filename5".into(), "thumbnail5".into())),
            ])
        );

        Ok(())
    }

    #[test]
    fn test_search() -> Result<()> {
        // given
        let config = create_config()?;
        let (repo_read, repo_write) = TantivyRepository::create(&config)?;
        let tuples_to_index = vec![
            DocDetails::new("filename1", "some document body", "thumbnail1"),
            DocDetails::new("filename2", "another text here", "thumbnail2"),
            DocDetails::new("filename3", "important information", "thumbnail3"),
            DocDetails::new("filename4", "this is not so important", "thumbnail4"),
            DocDetails::new("filename5", "and this is last line", "thumbnail5"),
        ];

        // when
        repo_write.index(&tuples_to_index)?;
        let results = repo_read.search("line".into())?;

        // then
        assert_eq!(
            results,
            SearchResult::from_vec(vec![SearchEntry::new((
                "filename5".into(),
                "thumbnail5".into()
            )),])
        );

        Ok(())
    }

    #[test]
    fn test_search_with_fuzziness() -> Result<()> {
        // given
        let config = create_config()?;
        let (repo_read, repo_write) = TantivyRepository::create(&config)?;
        let tuples_to_index = vec![
            DocDetails::new("filename1", "some document body", "thumbnail1"),
            DocDetails::new("filename2", "another text here", "thumbnail2"),
            DocDetails::new("filename3", "this is unique word: 9fZX", "thumbnail3"),
        ];

        // when
        repo_write.index(&tuples_to_index)?;
        // NOTE: it's not the same word as above, two letters of fuzziness is fine
        let first_results = repo_read.search("9fAB".into())?;
        // NOTE: three letters is too much
        let second_results = repo_read.search("9ABC".into())?;

        // then
        assert_eq!(
            first_results,
            SearchResult::from_vec(vec![SearchEntry::new((
                "filename3".into(),
                "thumbnail3".into()
            )),])
        );
        assert_eq!(second_results, SearchResult::from_vec(vec![]));

        Ok(())
    }
}
