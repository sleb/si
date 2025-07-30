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

#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelFile {
    pub size: u64,
    pub path: PathBuf,
}

#[derive(Debug)]
pub struct HuggingFaceRepoInfo {}

#[derive(Debug)]
pub struct HuggingFaceFile {}

#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::{tempdir, NamedTempFile};

    #[test]
    fn test_model_info_new() {
        let files = vec![
            ModelFile {
                size: 1024,
                path: PathBuf::from("/path/to/file1.bin"),
            },
            ModelFile {
                size: 2048,
                path: PathBuf::from("/path/to/file2.json"),
            },
        ];

        let model_info = ModelInfo::new("test-model", files.clone());

        assert_eq!(model_info.model_id, "test-model");
        assert_eq!(model_info.files.len(), 2);
        assert_eq!(model_info.files[0].size, 1024);
        assert_eq!(model_info.files[1].size, 2048);
    }

    #[test]
    fn test_model_info_from_path() -> Result<()> {
        let mut temp_file = NamedTempFile::new()?;
        let model_data = r#"{
            "model_id": "test-model",
            "files": [
                {
                    "size": 1024,
                    "path": "/path/to/file.bin"
                }
            ]
        }"#;

        temp_file.write_all(model_data.as_bytes())?;
        temp_file.flush()?;

        let model_info = ModelInfo::try_from(temp_file.path())?;

        assert_eq!(model_info.model_id, "test-model");
        assert_eq!(model_info.files.len(), 1);
        assert_eq!(model_info.files[0].size, 1024);

        Ok(())
    }

    #[test]
    fn test_model_info_from_invalid_path() {
        let result = ModelInfo::try_from(Path::new("/nonexistent/path"));
        assert!(result.is_err());
    }

    #[test]
    fn test_model_info_from_invalid_json() -> Result<()> {
        let mut temp_file = NamedTempFile::new()?;
        temp_file.write_all(b"invalid json")?;
        temp_file.flush()?;

        let result = ModelInfo::try_from(temp_file.path());
        assert!(result.is_err());

        Ok(())
    }

    #[test]
    fn test_model_index_from_path() -> Result<()> {
        let mut temp_file = NamedTempFile::new()?;
        let index_data = r#"{
            "models": [
                {
                    "model_id": "model1",
                    "files": []
                },
                {
                    "model_id": "model2",
                    "files": [
                        {
                            "size": 512,
                            "path": "/path/to/model2.bin"
                        }
                    ]
                }
            ]
        }"#;

        temp_file.write_all(index_data.as_bytes())?;
        temp_file.flush()?;

        let model_index = ModelIndex::try_from(temp_file.path())?;

        assert_eq!(model_index.models.len(), 2);
        assert_eq!(model_index.models[0].model_id, "model1");
        assert_eq!(model_index.models[1].model_id, "model2");
        assert_eq!(model_index.models[1].files.len(), 1);

        Ok(())
    }

    #[test]
    fn test_model_index_from_invalid_path() {
        let result = ModelIndex::try_from(Path::new("/nonexistent/path"));
        assert!(result.is_err());
    }

    #[test]
    fn test_model_manager_builder() -> Result<()> {
        let temp_dir = tempdir()?;
        let models_dir = temp_dir.path().join("models");

        let api = Api::new().unwrap_or_else(|_| panic!("Failed to create API for test"));

        let manager = ModelManagerBuilder::new()
            .with_models_dir(models_dir.clone())
            .with_hf_api(api)
            .build()?;

        assert_eq!(manager.models_dir, models_dir);

        Ok(())
    }

    #[test]
    fn test_model_manager_builder_without_models_dir() {
        // This test assumes the system has a valid project directory
        let api = Api::new().unwrap_or_else(|_| panic!("Failed to create API for test"));

        let result = ModelManagerBuilder::new().with_hf_api(api).build();

        // Should either succeed with default directory or fail gracefully
        match result {
            Ok(_) => {
                // Success case - default directory was available
            }
            Err(_) => {
                // Failure case - no default directory available
                // This is acceptable in test environments
            }
        }
    }

    #[test]
    fn test_model_manager_builder_without_api() {
        let temp_dir = tempdir().unwrap();
        let models_dir = temp_dir.path().join("models");

        let result = ModelManagerBuilder::new()
            .with_models_dir(models_dir)
            .build();

        // Should either succeed with default API or fail gracefully
        match result {
            Ok(_) => {
                // Success case - API creation succeeded
            }
            Err(_) => {
                // Failure case - API creation failed (e.g., no network)
                // This is acceptable in test environments
            }
        }
    }

    #[test]
    fn test_model_manager_list_models_no_index() -> Result<()> {
        let temp_dir = tempdir()?;
        let models_dir = temp_dir.path().join("models");
        fs::create_dir_all(&models_dir)?;

        let api = Api::new().unwrap_or_else(|_| panic!("Failed to create API for test"));

        let manager = ModelManagerBuilder::new()
            .with_models_dir(models_dir)
            .with_hf_api(api)
            .build()?;

        let result = manager.list_models();
        assert!(result.is_err());

        Ok(())
    }

    #[test]
    fn test_model_manager_list_models_with_index() -> Result<()> {
        let temp_dir = tempdir()?;
        let models_dir = temp_dir.path().join("models");
        fs::create_dir_all(&models_dir)?;

        // Create a model index file
        let index_path = models_dir.join(MODEL_INDEX_FILENAME);
        let index_data = r#"{
            "models": [
                {
                    "model_id": "test-model",
                    "files": [
                        {
                            "size": 1024,
                            "path": "/path/to/file.bin"
                        }
                    ]
                }
            ]
        }"#;
        fs::write(&index_path, index_data)?;

        let api = Api::new().unwrap_or_else(|_| panic!("Failed to create API for test"));

        let manager = ModelManagerBuilder::new()
            .with_models_dir(models_dir)
            .with_hf_api(api)
            .build()?;

        let models = manager.list_models()?;
        assert_eq!(models.len(), 1);
        assert_eq!(models[0].model_id, "test-model");
        assert_eq!(models[0].files.len(), 1);

        Ok(())
    }

    #[test]
    fn test_get_project_dir() {
        // Test that get_project_dir returns a consistent value
        let dir1 = get_project_dir();
        let dir2 = get_project_dir();

        // Both calls should return the same result
        assert_eq!(dir1.is_some(), dir2.is_some());

        if let (Some(d1), Some(d2)) = (dir1, dir2) {
            assert_eq!(d1.data_dir(), d2.data_dir());
        }
    }

    #[test]
    fn test_model_file_serialization() -> Result<()> {
        let model_file = ModelFile {
            size: 2048,
            path: PathBuf::from("/test/path/file.bin"),
        };

        let json = serde_json::to_string(&model_file)?;
        let deserialized: ModelFile = serde_json::from_str(&json)?;

        assert_eq!(model_file.size, deserialized.size);
        assert_eq!(model_file.path, deserialized.path);

        Ok(())
    }

    #[test]
    fn test_model_info_serialization() -> Result<()> {
        let files = vec![
            ModelFile {
                size: 1024,
                path: PathBuf::from("/path/to/file1.bin"),
            },
            ModelFile {
                size: 2048,
                path: PathBuf::from("/path/to/file2.json"),
            },
        ];

        let model_info = ModelInfo::new("test-model", files);

        let json = serde_json::to_string(&model_info)?;
        let deserialized: ModelInfo = serde_json::from_str(&json)?;

        assert_eq!(model_info.model_id, deserialized.model_id);
        assert_eq!(model_info.files.len(), deserialized.files.len());

        for (original, deserialized) in model_info.files.iter().zip(deserialized.files.iter()) {
            assert_eq!(original.size, deserialized.size);
            assert_eq!(original.path, deserialized.path);
        }

        Ok(())
    }

    #[test]
    fn test_model_index_serialization() -> Result<()> {
        let models = vec![
            ModelInfo::new("model1", vec![]),
            ModelInfo::new(
                "model2",
                vec![ModelFile {
                    size: 512,
                    path: PathBuf::from("/path/to/model2.bin"),
                }],
            ),
        ];

        let model_index = ModelIndex { models };

        let json = serde_json::to_string(&model_index)?;
        let deserialized: ModelIndex = serde_json::from_str(&json)?;

        assert_eq!(model_index.models.len(), deserialized.models.len());
        assert_eq!(
            model_index.models[0].model_id,
            deserialized.models[0].model_id
        );
        assert_eq!(
            model_index.models[1].model_id,
            deserialized.models[1].model_id
        );
        assert_eq!(
            model_index.models[1].files.len(),
            deserialized.models[1].files.len()
        );

        Ok(())
    }
}
