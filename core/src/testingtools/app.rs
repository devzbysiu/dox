use crate::configuration::factories::{fs, repository, Context};
use crate::entities::location::SafePathBuf;
use crate::startup::rocket;
use crate::testingtools::api::ApiResponse;
use crate::testingtools::services::encrypter::{
    failing as failing_cipher, tracked as tracked_cipher, CipherSpies,
};
use crate::testingtools::services::fs::{failing as failing_fs, tracked as tracked_fs, FsSpies};
use crate::testingtools::services::repo::{tracked, StateSpies};
use crate::testingtools::TestConfig;
use crate::use_cases::cipher::Cipher;
use crate::use_cases::fs::Fs;
use crate::use_cases::repository::State;

use anyhow::Result;
use base64::engine::general_purpose::STANDARD as b64;
use base64::Engine;
use rocket::local::blocking::Client;
use rocket::serde::json::json;
use std::convert::TryInto;
use std::fs;
use tracing::debug;
use urlencoding::encode;

pub fn start_test_app() -> Result<App> {
    let config = TestConfig::new()?;
    let client = Client::tracked(rocket(Context::new(&config)?))?;
    Ok(App {
        client,
        config,
        repo_spies: None,
        cipher_spies: None,
        fs_spies: None,
    })
}

pub fn test_app() -> Result<AppBuilder> {
    let config = TestConfig::new()?;
    Ok(AppBuilder {
        ctx: Some(Context::new(&config)?),
        config: Some(config),
        repo_spies: None,
        cipher_spies: None,
        fs_spies: None,
    })
}

pub struct App {
    client: Client,
    #[allow(unused)]
    config: TestConfig,
    repo_spies: Option<StateSpies>,
    #[allow(unused)]
    cipher_spies: Option<CipherSpies>,
    fs_spies: Option<FsSpies>,
}

impl App {
    pub fn wait_til_indexed(&mut self) {
        self.repo_spies().index_called();
    }

    #[allow(unused)]
    pub fn wait_til_encrypted(&mut self) {
        self.cipher_spies().encrypt_called();
    }

    pub fn wait_til_file_removed(&mut self) {
        self.fs_spies().rm_file_called();
    }

    fn repo_spies(&self) -> &StateSpies {
        self.repo_spies
            .as_ref()
            .unwrap_or_else(|| panic!("uninitialized tracked repo spies"))
    }

    fn cipher_spies(&self) -> &CipherSpies {
        self.cipher_spies
            .as_ref()
            .unwrap_or_else(|| panic!("uninitialized cipher spies"))
    }

    fn fs_spies(&self) -> &FsSpies {
        self.fs_spies
            .as_ref()
            .unwrap_or_else(|| panic!("uninitialized fs spy"))
    }

    pub fn search<S: Into<String>>(&self, q: S) -> Result<ApiResponse> {
        let q = q.into();
        self.get(format!("/search?q={}", encode(&q)))
    }

    fn get<S: Into<String>>(&self, url: S) -> Result<ApiResponse> {
        self.client.get(url.into()).dispatch().try_into()
    }

    pub fn upload_doc(&self, path: &SafePathBuf) -> Result<ApiResponse> {
        let body = b64.encode(fs::read(path)?);
        let filename = path.filename();
        self.client
            .post("/document/upload")
            .body(
                json!({
                    "filename": filename,
                    "body": body
                })
                .to_string(),
            )
            .dispatch()
            .try_into()
    }

    pub fn get_doc<S: Into<String>>(&self, name: S) -> Result<ApiResponse> {
        self.get(format!("/document/{}", name.into()))
    }

    pub fn get_thumbnail<S: Into<String>>(&self, name: S) -> Result<ApiResponse> {
        self.get(format!("/thumbnail/{}", name.into()))
    }

    pub fn thumbnail_exists<S: Into<String>>(&self, name: S) -> bool {
        let name = name.into();
        debug!("checking if thumbnail '{}' exists", name);
        self.config.thumbnail_path(name).exists()
    }

    pub fn document_exists<S: Into<String>>(&self, name: S) -> bool {
        let name = name.into();
        debug!("checking if document '{}' exists", name);
        self.config.doc_path(name).exists()
    }
}

pub struct AppBuilder {
    config: Option<TestConfig>,
    ctx: Option<Context>,
    repo_spies: Option<StateSpies>,
    cipher_spies: Option<CipherSpies>,
    fs_spies: Option<FsSpies>,
}

impl AppBuilder {
    pub fn with_tracked_repo(mut self) -> Result<Self> {
        let cfg = self.config.as_ref().unwrap();
        let (repo_spies, tracked_repo) = tracked(&repository(cfg)?);
        let ctx = self.ctx.as_mut().unwrap();
        ctx.with_repo(tracked_repo);
        self.repo_spies = Some(repo_spies);
        Ok(self)
    }

    pub fn with_failing_load_fs(mut self) -> Self {
        let ctx = self.ctx.as_mut().unwrap();
        ctx.with_fs(failing_fs());
        self
    }

    pub fn with_tracked_failing_cipher(mut self) -> Self {
        let (cipher_spies, tracked_cipher) = tracked_cipher(&failing_cipher());
        let ctx = self.ctx.as_mut().unwrap();
        ctx.with_cipher(tracked_cipher);
        self.cipher_spies = Some(cipher_spies);
        self
    }

    pub fn with_tracked_fs(mut self) -> Self {
        let (fs_spies, tracked_fs) = tracked_fs(fs());
        let ctx = self.ctx.as_mut().unwrap();
        ctx.with_fs(tracked_fs);
        self.fs_spies = Some(fs_spies);
        self
    }

    pub fn start(mut self) -> Result<App> {
        let client = Client::tracked(rocket(self.context()))?;
        let repo_spies = self.repo_spies();
        let config = self.config();
        let cipher_spies = self.cipher_spies();
        let fs_spies = self.fs_spies();
        Ok(App {
            client,
            config,
            repo_spies,
            cipher_spies,
            fs_spies,
        })
    }

    fn context(&mut self) -> Context {
        self.ctx
            .take()
            .unwrap_or_else(|| panic!("uninitialized context"))
    }

    fn repo_spies(&mut self) -> Option<StateSpies> {
        self.repo_spies.take()
    }

    fn cipher_spies(&mut self) -> Option<CipherSpies> {
        self.cipher_spies.take()
    }

    fn fs_spies(&mut self) -> Option<FsSpies> {
        self.fs_spies.take()
    }

    fn config(&mut self) -> TestConfig {
        self.config
            .take()
            .unwrap_or_else(|| panic!("uninitialized config"))
    }
}

impl Context {
    pub fn with_repo(&mut self, repo: State) -> &Self {
        self.repo = repo;
        self
    }

    pub fn with_fs(&mut self, fs: Fs) -> &Self {
        self.fs = fs;
        self
    }

    pub fn with_cipher(&mut self, cipher: Cipher) -> &Self {
        self.cipher = cipher;
        self
    }
}
