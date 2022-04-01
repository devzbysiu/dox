use crate::cfg::Config;
use crate::extractor::DocDetails;
use crate::result::{DoxErr, Result};

use core::fmt;
use log::debug;
use rocket::serde::Serialize;
use std::fs::create_dir_all;
use tantivy::collector::TopDocs;
use tantivy::directory::MmapDirectory;
use tantivy::query::{AllQuery, FuzzyTermQuery, Query};
use tantivy::schema::{Field, Schema, Value, STORED, TEXT};
use tantivy::{doc, DocAddress, Index, LeasedItem, ReloadPolicy, Term};

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

#[derive(Debug)]
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
        let top_docs = searcher.search(&self.make_query(term), &TopDocs::with_limit(100))?;
        self.to_search_results(&searcher, top_docs)
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

    fn to_search_results(&self, searcher: &Searcher, docs: TantivyDocs) -> Result<SearchResults> {
        let mut results = Vec::new();
        for (_score, doc_address) in docs {
            let retrieved_doc = searcher.doc(doc_address)?;
            let filenames = retrieved_doc.get_all(self.field(&Fields::Filename));
            let thumbnails = retrieved_doc.get_all(self.field(&Fields::Thumbnail));
            results.extend(to_search_entries(filenames, thumbnails));
        }
        Ok(SearchResults::new(results))
    }

    pub fn all_documents(&self) -> Result<SearchResults> {
        debug!("fetching all documents...");
        let searcher = self.create_searcher()?;
        let top_docs = searcher.search(&AllQuery, &TopDocs::with_limit(100))?;
        self.to_search_results(&searcher, top_docs)
    }
}

fn to_search_entries<'a, V: Iterator<Item = &'a Value>>(
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
    (as_text(filename), as_text(thumbnail))
}

fn as_text(val: &Value) -> String {
    val.as_text()
        .unwrap_or_else(|| panic!("failed to extract text"))
        .to_string()
}

#[derive(Debug, Serialize, Default, PartialEq, Eq)]
pub struct SearchResults {
    entries: Vec<SearchEntry>,
}

impl SearchResults {
    pub fn new(entries: Vec<SearchEntry>) -> Self {
        Self { entries }
    }
}

#[derive(Debug, Serialize, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct SearchEntry {
    filename: String,
    thumbnail: String,
}

impl SearchEntry {
    pub fn new((filename, thumbnail): (String, String)) -> Self {
        Self {
            filename,
            thumbnail,
        }
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
    schema_builder.add_text_field(&Fields::Thumbnail.to_string(), TEXT | STORED);
    let schema = schema_builder.build();
    let dir = MmapDirectory::open(&cfg.index_dir)?;
    let index = Index::open_or_create(dir, schema.clone())?;
    Ok(RepoTools { index, schema })
}

#[derive(Debug, Clone)]
pub struct RepoTools {
    pub index: Index,
    pub schema: Schema,
}

#[allow(clippy::module_name_repetitions)]
pub fn index_docs(tuples: &[DocDetails], tools: &RepoTools) -> Result<()> {
    debug!("indexing...");
    let index = &tools.index;
    let schema = &tools.schema;
    // NOTE: IndexWriter is already multithreaded and
    // cannot be shared between external threads
    let mut index_writer = index.writer(50_000_000)?;
    let filename = schema.get_field(&Fields::Filename.to_string()).unwrap();
    let body = schema.get_field(&Fields::Body.to_string()).unwrap();
    let thumbnail = schema.get_field(&Fields::Thumbnail.to_string()).unwrap();
    for t in tuples {
        debug!("indexing {}", t.filename);
        index_writer.add_document(doc!(
                filename => t.filename.clone(),
                body => t.body.clone(),
                thumbnail => t.thumbnail.clone(),
        ))?;
        index_writer.commit()?;
    }
    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;
    use anyhow::Result;
    use std::fs::File;
    use std::time::Duration;
    use testutils::{create_index_dir, create_thumbnails_dir, create_watched_dir};

    #[test]
    fn test_mk_index_and_schema_when_index_dir_is_taken_by_file() -> Result<()> {
        // given
        let config = setup_dirs_and_config()?;
        File::create(&config.index_dir)?;

        // when
        let result = mk_idx_and_schema(&config);

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

    #[test]
    fn test_index_docs() -> Result<()> {
        // given
        let config = setup_dirs_and_config()?;
        let repo_tools = mk_idx_and_schema(&config)?;
        let tuples_to_index = vec![
            DocDetails::new("filename1", "body1", "thumbnail1"),
            DocDetails::new("filename2", "body2", "thumbnail2"),
            DocDetails::new("filename3", "body3", "thumbnail3"),
            DocDetails::new("filename4", "body4", "thumbnail4"),
            DocDetails::new("filename5", "body5", "thumbnail5"),
        ];

        // when
        index_docs(&tuples_to_index, &repo_tools)?;
        let repo = Repo::new(repo_tools);
        // TODO: this test should check only indexing but it's also
        // searching via all_documents
        let mut all_docs = repo.all_documents()?;
        all_docs.entries.sort();

        // then
        assert_eq!(
            all_docs,
            SearchResults::new(vec![
                SearchEntry::new(("filename1".into(), "thumbnail1".into())),
                SearchEntry::new(("filename2".into(), "thumbnail2".into())),
                SearchEntry::new(("filename3".into(), "thumbnail3".into())),
                SearchEntry::new(("filename4".into(), "thumbnail4".into())),
                SearchEntry::new(("filename5".into(), "thumbnail5".into())),
            ])
        );

        Ok(())
    }

    fn setup_dirs_and_config() -> Result<Config> {
        let index_dir = create_index_dir()?;
        let watched_dir = create_watched_dir()?;
        let thumbnails_dir = create_thumbnails_dir()?;
        Ok(Config {
            watched_dir: watched_dir.path().to_path_buf(),
            thumbnails_dir: thumbnails_dir.path().to_path_buf(),
            index_dir: index_dir.path().to_path_buf(),
            cooldown_time: Duration::from_secs(1),
        })
    }

    #[test]
    fn test_search() -> Result<()> {
        // given
        let config = setup_dirs_and_config()?;
        let repo_tools = mk_idx_and_schema(&config)?;
        let tuples_to_index = vec![
            DocDetails::new("filename1", "some document body", "thumbnail1"),
            DocDetails::new("filename2", "another text here", "thumbnail2"),
            DocDetails::new("filename3", "important information", "thumbnail3"),
            DocDetails::new("filename4", "this is not so important", "thumbnail4"),
            DocDetails::new("filename5", "and this is last line", "thumbnail5"),
        ];

        // when
        index_docs(&tuples_to_index, &repo_tools)?;
        let repo = Repo::new(repo_tools);
        let all_docs = repo.search("line")?;

        // then
        assert_eq!(
            all_docs,
            SearchResults::new(vec![SearchEntry::new((
                "filename5".into(),
                "thumbnail5".into()
            )),])
        );

        Ok(())
    }

    #[test]
    fn test_search_with_fuzziness() -> Result<()> {
        // given
        let config = setup_dirs_and_config()?;
        let repo_tools = mk_idx_and_schema(&config)?;
        let tuples_to_index = vec![
            DocDetails::new("filename1", "some document body", "thumbnail1"),
            DocDetails::new("filename2", "another text here", "thumbnail2"),
            DocDetails::new("filename3", "this is unique word: 9fZX", "thumbnail3"),
        ];

        // when
        index_docs(&tuples_to_index, &repo_tools)?;
        let repo = Repo::new(repo_tools);
        // NOTE: it's not the same word as above, two letters of fuzziness is fine
        let first_results = repo.search("9fAB")?;
        // NOTE: three letters is too much
        let second_results = repo.search("9ABC")?;

        // then
        assert_eq!(
            first_results,
            SearchResults::new(vec![SearchEntry::new((
                "filename3".into(),
                "thumbnail3".into()
            )),])
        );
        assert!(second_results.entries.is_empty(),);

        Ok(())
    }
}
