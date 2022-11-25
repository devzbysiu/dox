use crate::configuration::factories::{repository, Context};
use crate::helpers::PathRefExt;
use crate::startup::rocket;
use crate::testingtools::integration::api::ApiResponse;
use crate::testingtools::integration::config::TestConfig;
use crate::testingtools::integration::services::{FailingLoadFs, RepoSpies, TrackedRepo};
use crate::use_cases::fs::Fs;
use crate::use_cases::repository::Repo;

use anyhow::Result;
use rocket::local::blocking::Client;
use rocket::serde::json::json;
use std::convert::TryInto;
use std::fs;
use std::path::Path;
use urlencoding::encode;

pub fn start_test_app() -> Result<App> {
    let config = TestConfig::new()?;
    let client = Client::tracked(rocket(Context::new(&config)?))?;
    Ok(App {
        client,
        config,
        repo_spies: None,
    })
}

pub fn test_app() -> Result<AppBuilder> {
    let config = TestConfig::new()?;
    Ok(AppBuilder {
        ctx: Some(Context::new(&config)?),
        config: Some(config),
        repo_spies: None,
    })
}

pub struct App {
    client: Client,
    #[allow(unused)]
    config: TestConfig,
    repo_spies: Option<RepoSpies>,
}

impl App {
    pub fn wait_til_indexed(&mut self) {
        self.repo_spies().write().method_called();
    }

    fn repo_spies(&self) -> &RepoSpies {
        self.repo_spies
            .as_ref()
            .unwrap_or_else(|| panic!("uninitialized tracked repo spy"))
    }

    pub fn search<S: Into<String>>(&self, q: S) -> Result<ApiResponse> {
        let q = q.into();
        self.get(format!("/search?q={}", encode(&q)))
    }

    fn get<S: Into<String>>(&self, url: S) -> Result<ApiResponse> {
        self.client.get(url.into()).dispatch().try_into()
    }

    pub fn upload_doc<P: AsRef<Path>>(&self, path: P) -> Result<ApiResponse> {
        let body = base64::encode(fs::read(&path)?);
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
}

pub struct AppBuilder {
    config: Option<TestConfig>,
    ctx: Option<Context>,
    repo_spies: Option<RepoSpies>,
}

impl AppBuilder {
    pub fn with_tracked_repo(mut self) -> Result<Self> {
        let cfg = self.config.as_ref().unwrap();
        let (repo_spies, tracked_repo) = TrackedRepo::wrap(&repository(cfg)?);
        let cfg = self.ctx.as_mut().unwrap();
        cfg.with_repo(tracked_repo);
        self.repo_spies = Some(repo_spies);
        Ok(self)
    }

    pub fn with_failing_load_fs(mut self) -> Self {
        let cfg = self.ctx.as_mut().unwrap();
        cfg.with_fs(FailingLoadFs::new());
        self
    }

    pub fn start(mut self) -> Result<App> {
        let client = Client::tracked(rocket(self.context()))?;
        let repo_spies = self.repo_spies();
        let config = self.config();
        Ok(App {
            client,
            config,
            repo_spies,
        })
    }

    fn context(&mut self) -> Context {
        self.ctx
            .take()
            .unwrap_or_else(|| panic!("uninitialized context"))
    }

    fn repo_spies(&mut self) -> Option<RepoSpies> {
        self.repo_spies.take()
    }

    fn config(&mut self) -> TestConfig {
        self.config
            .take()
            .unwrap_or_else(|| panic!("uninitialized config"))
    }
}

impl Context {
    pub fn with_repo(&mut self, repo: Repo) -> &Self {
        self.repo = repo;
        self
    }

    pub fn with_fs(&mut self, fs: Fs) -> &Self {
        self.fs = fs;
        self
    }
}
