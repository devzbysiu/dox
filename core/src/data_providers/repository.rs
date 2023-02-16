//! This is concrete implementation of [`crate::use_cases::repository`] abstractions.
//!
//! It uses [`tantivy`] as full text search library.
use crate::entities::document::DocDetails;
use crate::entities::location::Location;
use crate::entities::user::User;
use crate::result::{IndexerErr, RepositoryErr, SearchErr};
use crate::use_cases::config::Config;
use crate::use_cases::repository::{
    AppState, AppStateReader, AppStateWriter, SearchEntry, SearchResult, State, StateReader,
    StateWriter,
};

use base64::engine::general_purpose::STANDARD as b64;
use base64::Engine;
use core::fmt;
use dashmap::DashMap;
use std::convert::TryInto;
use std::fmt::Debug;
use std::fs::create_dir_all;
use std::path::PathBuf;
use std::sync::Arc;
use tantivy::collector::TopDocs;
use tantivy::directory::MmapDirectory;
use tantivy::query::{AllQuery, FuzzyTermQuery, Query};
use tantivy::schema::{Field, Schema, Value, STORED, TEXT};
use tantivy::{doc, DocAddress, Index, ReloadPolicy, Searcher, Term};
use tracing::{debug, error, instrument, warn};

type TantivyDocs = Vec<(f32, DocAddress)>;

pub struct TantivyState {
    read: StateReader,
    write: StateWriter,
}

impl TantivyState {
    pub fn create(cfg: &Config) -> Result<State, RepositoryErr> {
        if cfg.index_dir.exists() && cfg.index_dir.is_file() {
            return Err(RepositoryErr::InvalidIndexPath(format!(
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
        let indexes = Arc::new(DashMap::new());
        Ok(Box::new(Self {
            read: Arc::new(TantivyStateReader::new(indexes.clone(), schema.clone())),
            write: Arc::new(TantivyStateWriter::new(
                indexes,
                cfg.index_dir.clone(),
                schema,
            )),
        }))
    }
}

impl AppState for TantivyState {
    fn reader(&self) -> StateReader {
        self.read.clone()
    }

    fn writer(&self) -> StateWriter {
        self.write.clone()
    }
}

#[derive(Debug, Clone)]
struct TantivyStateReader {
    indexes: Arc<DashMap<User, Index>>,
    schema: Schema,
}

impl TantivyStateReader {
    fn new(indexes: Arc<DashMap<User, Index>>, schema: Schema) -> Self {
        Self { indexes, schema }
    }

    #[instrument(skip(self))]
    fn create_searcher(&self, user: User) -> Result<Searcher, SearchErr> {
        Ok(self
            .indexes
            .get(&user)
            .ok_or(SearchErr::MissingIndex(user.email))?
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

    #[instrument(skip(self, searcher))]
    fn to_search_result(
        &self,
        searcher: &Searcher,
        docs: TantivyDocs,
    ) -> Result<SearchResult, SearchErr> {
        let mut results = Vec::new();
        for (_score, doc_address) in docs {
            let retrieved_doc = searcher.doc(doc_address)?;
            let filenames = retrieved_doc.get_all(self.field(&Fields::Filename));
            let thumbnails = retrieved_doc.get_all(self.field(&Fields::Thumbnail));
            results.extend(to_search_entries(filenames, thumbnails));
        }
        Ok(results.into())
    }

    #[instrument(skip(self))]
    fn search_for(&self, user: User, query: &impl Query) -> Result<SearchResult, SearchErr> {
        let searcher = self.create_searcher(user);
        if let Err(SearchErr::MissingIndex(email)) = searcher {
            debug!("No index for user: '{}'", email);
            return Ok(SearchResult::default());
        }
        let searcher = searcher.unwrap(); // can unwrap because it's checked above
        let top_docs = searcher.search(query, &TopDocs::with_limit(100))?;
        self.to_search_result(&searcher, top_docs)
    }
}

impl AppStateReader for TantivyStateReader {
    #[instrument(skip(self))]
    fn search(&self, user: User, term: String) -> Result<SearchResult, SearchErr> {
        debug!("search of user: '{}', for: '{}'", user.email, term);
        let res = self.search_for(user, &self.make_query(term))?;
        debug!("found docs: '{:?}'", res);
        Ok(res)
    }

    #[instrument(skip(self))]
    fn all_docs(&self, user: User) -> Result<SearchResult, SearchErr> {
        self.search_for(user, &AllQuery)
    }
}

#[derive(Debug, Clone)]
struct TantivyStateWriter {
    indexes: Arc<DashMap<User, Index>>,
    idx_root: PathBuf,
    schema: Schema,
}

impl TantivyStateWriter {
    fn new(indexes: Arc<DashMap<User, Index>>, idx_root: PathBuf, schema: Schema) -> Self {
        Self {
            indexes,
            idx_root,
            schema,
        }
    }

    fn insert_idx_if_missing(&self, user: &User) -> Result<(), IndexerErr> {
        if !self.indexes.contains_key(user) {
            let idx_dir = self.idx_root.join(b64.encode(&user.email));
            debug!(
                "creating new index directory for '{}' under path '{}'",
                user.email,
                idx_dir.display()
            );
            create_dir_all(&idx_dir)?;
            let dir = MmapDirectory::open(&idx_dir)?;
            let index = Index::open_or_create(dir, self.schema.clone())?;
            debug!("adding newly created index to indexes map");
            self.indexes.insert(user.clone(), index);
        }
        Ok(())
    }

    fn field(&self, field: &Fields) -> Field {
        // can unwrap because this field comes from an
        // enum and I'm using this enum to get the field
        self.schema.get_field(&field.to_string()).unwrap()
    }
}

impl AppStateWriter for TantivyStateWriter {
    #[instrument(skip(self, docs_details))]
    fn index(&self, docs_details: &[DocDetails]) -> Result<(), IndexerErr> {
        for doc_detail in docs_details {
            let user = &doc_detail.user;
            self.insert_idx_if_missing(user)?;
            let index = self.indexes.get(user).unwrap(); // can unwrap because it's added above
            let schema = &self.schema;
            // NOTE: IndexWriter is already multithreaded and
            // cannot be shared between external threads
            let mut index_writer = index.writer(50_000_000)?;
            let filename = schema.get_field(&Fields::Filename.to_string()).unwrap();
            let body = schema.get_field(&Fields::Body.to_string()).unwrap();
            let thumbnail = schema.get_field(&Fields::Thumbnail.to_string()).unwrap();
            debug!("indexing {:?}", doc_detail.filename);
            index_writer.add_document(doc!(
                    filename => doc_detail.filename.clone(),
                    body => doc_detail.body.clone(),
                    thumbnail => doc_detail.thumbnail.clone(),
            ))?;
            debug!("commiting new doc");
            index_writer.commit()?;
        }
        Ok(())
    }

    #[instrument(skip(self))]
    fn delete(&self, loc: &Location) -> Result<(), IndexerErr> {
        let Location::FS(paths) = loc;
        for path in paths {
            let user: User = path.try_into()?;
            let Some(index) = self.indexes.get(&user) else {
                error!("No index for user: '{}'", user);
                return Err(IndexerErr::NoIndex(user));
            };
            let mut writer = index.writer(50_000_000)?;
            let filename = path.filename();

            // NOTE: At this point, we don't know if the `Location` points to thumbnail or
            // document, but it doesn't matter, because we need to delete Tantivy document
            // containing both thumbnail and document name anyway.
            let doc_term = term(self.field(&Fields::Filename), &filename);
            let thumbnail_term = term(self.field(&Fields::Thumbnail), &filename);
            debug!("deleting '{}' as a doc name", filename);
            writer.delete_term(doc_term);
            debug!("deleting '{}' as a doc thumbnail", filename);
            writer.delete_term(thumbnail_term);
            debug!("commiting deletion");
            writer.commit()?;
        }
        Ok(())
    }
}

fn term<S: Into<String>>(field: Field, filename: S) -> Term {
    Term::from_field_text(field, &filename.into())
}

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

    use crate::configuration::telemetry::init_tracing;
    use crate::entities::file::{Filename, Thumbnailname};
    use crate::entities::user::FAKE_USER_EMAIL;
    use crate::testingtools::{
        docs_dir_path, index_dir_path, thumbnails_dir_path, watched_dir_path,
    };

    use anyhow::Result;
    use fake::{Fake, Faker};
    use std::fs::File;

    #[test]
    fn test_mk_index_and_schema_when_index_dir_is_taken_by_file() -> Result<()> {
        // given
        init_tracing();
        let config = create_config()?;
        File::create(&config.index_dir)?;

        // when
        let result = TantivyState::create(&config);

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
        let docs_dir = docs_dir_path()?;
        let watched_dir = watched_dir_path()?;
        let thumbnails_dir = thumbnails_dir_path()?;
        Ok(Config {
            watched_dir: watched_dir.path().to_path_buf(),
            docs_dir: docs_dir.path().to_path_buf(),
            thumbnails_dir: thumbnails_dir.path().to_path_buf(),
            index_dir: index_dir.path().to_path_buf(),
        })
    }

    #[test]
    fn test_index_docs() -> Result<()> {
        // given
        init_tracing();
        let config = create_config()?;
        let repo = TantivyState::create(&config)?;
        let user_email = FAKE_USER_EMAIL;
        let user = User::new(user_email);
        let tuples_to_index = vec![
            DocDetails::new(
                Filename::new("filename1")?,
                "body1",
                Thumbnailname::new("thumbnail1")?,
                user.clone(),
            ),
            DocDetails::new(
                Filename::new("filename2")?,
                "body2",
                Thumbnailname::new("thumbnail2")?,
                user.clone(),
            ),
            DocDetails::new(
                Filename::new("filename3")?,
                "body3",
                Thumbnailname::new("thumbnail3")?,
                user.clone(),
            ),
            DocDetails::new(
                Filename::new("filename4")?,
                "body4",
                Thumbnailname::new("thumbnail4")?,
                user.clone(),
            ),
            DocDetails::new(
                Filename::new("filename5")?,
                "body5",
                Thumbnailname::new("thumbnail5")?,
                user.clone(),
            ),
        ];

        // when
        repo.writer().index(&tuples_to_index)?;
        // TODO: this test should check only indexing but it's also
        // searching via all_documents
        let all_docs = repo.reader().all_docs(user)?;

        // then
        assert_eq!(
            all_docs,
            vec![
                SearchEntry::new(("filename1".into(), "thumbnail1".into())),
                SearchEntry::new(("filename2".into(), "thumbnail2".into())),
                SearchEntry::new(("filename3".into(), "thumbnail3".into())),
                SearchEntry::new(("filename4".into(), "thumbnail4".into())),
                SearchEntry::new(("filename5".into(), "thumbnail5".into())),
            ]
            .into()
        );

        Ok(())
    }

    #[test]
    fn test_search() -> Result<()> {
        // given
        init_tracing();
        let config = create_config()?;
        let repo = TantivyState::create(&config)?;
        let user = User::new(FAKE_USER_EMAIL);
        let tuples_to_index = vec![
            DocDetails::new(
                Filename::new("filename1")?,
                "body",
                Thumbnailname::new("thumbnail1")?,
                user.clone(),
            ),
            DocDetails::new(
                Filename::new("filename2")?,
                "text",
                Thumbnailname::new("thumbnail2")?,
                user.clone(),
            ),
            DocDetails::new(
                Filename::new("filename3")?,
                "information",
                Thumbnailname::new("thumbnail3")?,
                user.clone(),
            ),
            DocDetails::new(
                Filename::new("filename4")?,
                "not important",
                Thumbnailname::new("thumbnail4")?,
                user.clone(),
            ),
            DocDetails::new(
                Filename::new("filename5")?,
                "and last line",
                Thumbnailname::new("thumbnail5")?,
                user.clone(),
            ),
        ];

        // when
        repo.writer().index(&tuples_to_index)?;
        let results = repo.reader().search(user, "line".into())?;

        // then
        assert_eq!(
            results,
            vec![SearchEntry::new(("filename5".into(), "thumbnail5".into())),].into()
        );

        Ok(())
    }

    #[test]
    fn test_search_with_fuzziness() -> Result<()> {
        // given
        init_tracing();
        let config = create_config()?;
        let repo = TantivyState::create(&config)?;
        let user = User::new(FAKE_USER_EMAIL);
        let tuples_to_index = vec![
            DocDetails::new(
                Filename::new("filename1")?,
                "some body",
                Thumbnailname::new("thumbnail1")?,
                user.clone(),
            ),
            DocDetails::new(
                Filename::new("filename2")?,
                "another text here",
                Thumbnailname::new("thumbnail2")?,
                user.clone(),
            ),
            DocDetails::new(
                Filename::new("filename3")?,
                "unique word: 9fZX",
                Thumbnailname::new("thumbnail3")?,
                user.clone(),
            ),
        ];

        // when
        repo.writer().index(&tuples_to_index)?;
        // NOTE: it's not the same word as above, two letters of fuzziness is fine
        let first_results = repo.reader().search(user.clone(), "9fAB".into())?;
        // NOTE: three letters is too much
        let second_results = repo.reader().search(user, "9ABC".into())?;

        // then
        assert_eq!(
            first_results,
            vec![SearchEntry::new(("filename3".into(), "thumbnail3".into())),].into()
        );
        assert_eq!(second_results, Vec::new().into());

        Ok(())
    }

    #[test]
    fn delete_using_doc_path_allows_to_remove_data_of_document() -> Result<()> {
        // given
        init_tracing();
        let config = create_config()?;
        let repo = TantivyState::create(&config)?;
        let user = User::new(FAKE_USER_EMAIL);
        let tuples_to_index = vec![
            DocDetails::new(
                Filename::new("filename1")?,
                "some body",
                Thumbnailname::new("thumbnail1")?,
                user.clone(),
            ),
            DocDetails::new(
                Filename::new("filename2")?,
                "another text here",
                Thumbnailname::new("thumbnail2")?,
                user.clone(),
            ),
            DocDetails::new(
                Filename::new("filename3")?,
                "unique word: 9fZX",
                Thumbnailname::new("thumbnail3")?,
                user.clone(),
            ),
            DocDetails::new(
                Filename::new("filename3")?,
                "unique word: 9fZX",
                Thumbnailname::new("thumbnail3")?,
                user.clone(),
            ),
            DocDetails::new(
                Filename::new("filename3")?,
                "unique word: 9fZX",
                Thumbnailname::new("thumbnail3")?,
                user.clone(),
            ),
        ];
        repo.writer().index(&tuples_to_index)?;
        let res = repo.reader().search(user.clone(), "9fZX".into())?;
        assert_eq!(
            res,
            vec![
                SearchEntry::new(("filename3".into(), "thumbnail3".into())),
                SearchEntry::new(("filename3".into(), "thumbnail3".into())),
                SearchEntry::new(("filename3".into(), "thumbnail3".into()))
            ]
            .into()
        );
        // NOTE: Only name of the file matters
        let loc = Location::FS(vec!["/any/path/filename3".into()]);

        // when
        repo.writer().delete(&loc)?;
        let res = repo.reader().search(user, "9fZX".into())?;

        // then
        assert_eq!(res, Vec::new().into());

        Ok(())
    }

    #[test]
    fn delete_using_thumbnail_path_allows_to_remove_data_of_document() -> Result<()> {
        // given
        init_tracing();
        let config = create_config()?;
        let repo = TantivyState::create(&config)?;
        let user = User::new(FAKE_USER_EMAIL);
        let tuples_to_index = vec![
            DocDetails::new(
                Filename::new("filename1")?,
                "some body",
                Thumbnailname::new("thumbnail1")?,
                user.clone(),
            ),
            DocDetails::new(
                Filename::new("filename2")?,
                "another text here",
                Thumbnailname::new("thumbnail2")?,
                user.clone(),
            ),
            DocDetails::new(
                Filename::new("filename3")?,
                "unique word: 9fZX",
                Thumbnailname::new("thumbnail3")?,
                user.clone(),
            ),
            DocDetails::new(
                Filename::new("filename3")?,
                "unique word: 9fZX",
                Thumbnailname::new("thumbnail3")?,
                user.clone(),
            ),
            DocDetails::new(
                Filename::new("filename3")?,
                "unique word: 9fZX",
                Thumbnailname::new("thumbnail3")?,
                user.clone(),
            ),
        ];
        repo.writer().index(&tuples_to_index)?;
        let res = repo.reader().search(user.clone(), "9fZX".into())?;
        assert_eq!(
            res,
            vec![
                SearchEntry::new(("filename3".into(), "thumbnail3".into())),
                SearchEntry::new(("filename3".into(), "thumbnail3".into())),
                SearchEntry::new(("filename3".into(), "thumbnail3".into()))
            ]
            .into()
        );
        // NOTE: Only name of the file matters
        let loc = Location::FS(vec!["/any/path/thumbnail3".into()]);

        // when
        repo.writer().delete(&loc)?;
        let res = repo.reader().search(user, "9fZX".into())?;

        // then
        assert_eq!(res, Vec::new().into());

        Ok(())
    }

    #[test]
    fn using_delete_with_not_existing_user_returns_no_index_error() -> Result<()> {
        // given
        init_tracing();
        let config = create_config()?;
        let repo = TantivyState::create(&config)?;
        // NOTE: Index data under fake user (inside `tuples_to_index`), then `delete` with test
        // `User` implementation which is different user
        repo.writer().index(&[Faker.fake()])?;
        let loc = Location::FS(vec![Faker.fake()]);

        // when
        let res = repo.writer().delete(&loc);

        // then
        assert!(matches!(res, Err(IndexerErr::NoIndex(_))));

        Ok(())
    }
}
