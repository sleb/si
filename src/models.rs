use std::{
    fs::{self, File},
    path::{Path, PathBuf},
    sync::OnceLock,
};

use anyhow::{Context, Result};
use directories::ProjectDirs;
use hf_hub::api::tokio::Api;
use log::debug;
use serde::{Deserialize, Serialize};

static PROJECT_DIR: OnceLock<Option<ProjectDirs>> = OnceLock::new();
const MODELS_DIR: &str = "models";
const MODEL_INDEX_FILENAME: &str = "model_index.json";

fn get_project_dir() -> Option<&'static ProjectDirs> {
    let dir = PROJECT_DIR.get_or_init(|| ProjectDirs::from("", "", "si"));
    dir.as_ref()
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ModelInfo {
    pub model_id: String,
    pub files: Vec<ModelFile>,
    // pub description: Option<String>,
    // pub tags: Vec<String>,
    // pub downloaded_at: Option<DateTime<Utc>>,
    // pub size_bytes: u64,
}

impl ModelInfo {
    pub fn new<T: Into<String>>(model_id: T, files: Vec<ModelFile>) -> Self {
        Self {
            model_id: model_id.into(),
            files,
        }
    }
}

impl TryFrom<&Path> for ModelInfo {
    type Error = anyhow::Error;

    fn try_from(path: &Path) -> Result<Self> {
        debug!("ModelInfo path: {path:?}");
        let file =
            File::open(path).with_context(|| format!("Failed to open {}", path.display()))?;
        let info: ModelInfo = serde_json::from_reader(file)
            .with_context(|| format!("Failed to parse model info from {}", path.display()))?;

        Ok(info)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ModelFile {
    pub size: u64,
    pub path: PathBuf,
}

#[derive(Debug)]
pub struct HuggingFaceRepoInfo {}

#[derive(Debug)]
pub struct HuggingFaceFile {}

#[derive(Debug, Serialize, Deserialize)]
pub struct ModelIndex {
    pub models: Vec<ModelInfo>,
}

impl TryFrom<&Path> for ModelIndex {
    type Error = anyhow::Error;

    fn try_from(path: &Path) -> Result<Self> {
        debug!("ModelIndex path: {path:?}");
        let file =
            File::open(path).with_context(|| format!("Failed to open {}", path.display()))?;
        let index: ModelIndex = serde_json::from_reader(file)
            .with_context(|| format!("Failed to parse model index from {}", path.display()))?;

        Ok(index)
    }
}

pub struct ModelManagerBuilder {
    models_dir: Option<PathBuf>,
    hf_api: Option<Api>,
}

impl ModelManagerBuilder {
    pub fn new() -> Self {
        Self {
            models_dir: None,
            hf_api: None,
        }
    }

    pub fn with_models_dir(mut self, models_dir: PathBuf) -> Self {
        self.models_dir = Some(models_dir);
        self
    }

    pub fn with_hf_api(mut self, hf_api: Api) -> Self {
        self.hf_api = Some(hf_api);
        self
    }

    pub fn build(self) -> Result<ModelManager> {
        let models_dir = self
            .models_dir
            .or_else(|| get_project_dir().map(|d| d.data_dir().join(MODELS_DIR)))
            .context("Models directory is not set")?;
        let hf_api = self
            .hf_api
            .or(Api::new().ok())
            .context("Failed to create HuggingFace API client")?;
        Ok(ModelManager { models_dir, hf_api })
    }
}

#[derive(Debug)]
pub struct ModelManager {
    models_dir: PathBuf,
    hf_api: Api,
}

impl ModelManager {
    pub fn new() -> Result<Self> {
        let model_manager = ModelManagerBuilder::new().build()?;
        fs::create_dir_all(&model_manager.models_dir)
            .with_context(|| "Failed to create models dir")?;
        Ok(model_manager)
    }

    pub fn list_models(&self) -> Result<Vec<ModelInfo>> {
        let model_index: ModelIndex = serde_json::from_reader(
            File::open(&self.models_dir.join(MODEL_INDEX_FILENAME))
                .context("Couldn't open model index")?,
        )
        .with_context(|| "Couldn't de-serialize model index")?;
        Ok(model_index.models)
    }

    pub async fn download_model(&self, model_id: &str) -> Result<ModelInfo> {
        debug!("download_model: {model_id}");
        let mut model_info = ModelInfo::new(model_id, vec![]);
        let model = self.hf_api.model(model_id.to_string());
        let info = model
            .info()
            .await
            .with_context(|| format!("Failed to get info for `{model_id}`"))?;
        debug!("  info: {info:?}");
        for sibling in &info.siblings {
            debug!("    downloading file: {}", sibling.rfilename);
            let local_path = model
                .download(&sibling.rfilename)
                .await
                .with_context(|| format!("{} download faild", &sibling.rfilename))?;
            model_info.files.push(ModelFile {
                size: fs::metadata(local_path.as_path())
                    .with_context(|| {
                        format!("Couldn't get file size for `{}`", local_path.display())
                    })?
                    .len(),
                path: local_path,
            });
        }

        Ok(model_info)
    }
}
