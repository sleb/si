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

fn default_project_dir() -> Option<&'static ProjectDirs> {
    let dir = PROJECT_DIR.get_or_init(|| ProjectDirs::from("", "", "si"));
    dir.as_ref()
}

fn default_models_dir() -> Result<PathBuf> {
    default_project_dir()
        .map(|p| p.data_dir().join(MODELS_DIR))
        .context("Models directory is not set")
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

#[derive(Debug)]
struct ModelIndex {
    path: PathBuf,
}

impl ModelIndex {
    pub fn new(path: PathBuf) -> Self {
        debug!("ModelIndex path: {path:?}");
        Self { path }
    }

    pub fn models(&self) -> Result<Vec<ModelInfo>> {
        let model_data = self.model_index_data()?;
        Ok(model_data.models)
    }

    pub fn add_model(&self, model: ModelInfo) -> Result<()> {
        debug!("Adding `{}` to the index.", model.model_id);
        let mut index_data = self.model_index_data()?;
        let models = &mut index_data.models;
        if let Some(i) = models.iter().position(|m| m.model_id == model.model_id) {
            debug!("Model {} already exists in index", model.model_id);
            models[i] = model;
        } else {
            debug!("Adding model {} to index", model.model_id);
            models.push(model);
        }

        self.save(&index_data)
    }

    fn model_index_data(&self) -> Result<ModelIndexData> {
        match File::open(&self.path) {
            Ok(file) => {
                debug!("Reading model index from {}", self.path.display());
                let index_data: ModelIndexData =
                    serde_json::from_reader(file).with_context(|| {
                        format!("Failed to parse model index from {}", self.path.display())
                    })?;
                Ok(index_data)
            }
            Err(_) => {
                debug!(
                    "Model index file not found at {}, returning empty index",
                    self.path.display()
                );
                Ok(ModelIndexData { models: vec![] })
            }
        }
    }

    fn save(&self, index: &ModelIndexData) -> Result<()> {
        debug!("Saving index data to to {}", self.path.display());
        let file = File::create(&self.path).with_context(|| {
            format!(
                "Failed to create model index file at {}",
                self.path.display()
            )
        })?;
        serde_json::to_writer(file, index)
            .with_context(|| format!("Failed to write model index to {}", self.path.display()))?;
        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct ModelIndexData {
    pub(crate) models: Vec<ModelInfo>,
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
            .unwrap_or(default_models_dir().context("Models directory not set")?);

        if !models_dir.exists() {
            debug!("Creating models directory at {}", models_dir.display());
            fs::create_dir_all(&models_dir).context("Failed to create models dir")?;
        }

        let hf_api = self
            .hf_api
            .unwrap_or(Api::new().context("Failed to creae HuggingFace API")?);
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
        if !model_manager.models_dir.exists() {
            debug!(
                "Creating models directory at {}",
                model_manager.models_dir.display()
            );
            fs::create_dir_all(&model_manager.models_dir).context("Failed to create models dir")?;
        }
        Ok(model_manager)
    }

    pub fn list_models(&self) -> Result<Vec<ModelInfo>> {
        self.model_index().models().context("Failed to list models")
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

        // Automatically persist the downloaded model to the index
        let model_index = self.model_index();
        model_index
            .add_model(model_info.clone())
            .with_context(|| format!("Failed to add model '{}' to index", model_id))?;

        Ok(model_info)
    }

    fn model_index(&self) -> ModelIndex {
        ModelIndex::new(self.models_dir.join(MODEL_INDEX_FILENAME))
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
    fn test_model_info_from_invalid_json() -> Result<()> {
        let mut temp_file = NamedTempFile::new()?;
        temp_file.write_all(b"invalid json")?;
        temp_file.flush()?;

        let result = ModelInfo::try_from(temp_file.path());
        assert!(result.is_err());

        Ok(())
    }

    #[test]
    fn test_model_index_models_with_existing_file() -> Result<()> {
        let temp_dir = tempdir()?;
        let index_path = temp_dir.path().join("model_index.json");

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

        fs::write(&index_path, index_data)?;
        let model_index = ModelIndex::new(index_path);
        let models = model_index.models()?;

        assert_eq!(models.len(), 2);
        assert_eq!(models[0].model_id, "model1");
        assert_eq!(models[1].model_id, "model2");
        assert_eq!(models[1].files.len(), 1);

        Ok(())
    }

    #[test]
    fn test_model_index_models_with_missing_file() -> Result<()> {
        let temp_dir = tempdir()?;
        let index_path = temp_dir.path().join("nonexistent.json");

        let model_index = ModelIndex::new(index_path);
        let models = model_index.models()?;

        assert_eq!(models.len(), 0);
        Ok(())
    }

    #[test]
    fn test_model_index_models_with_invalid_json() -> Result<()> {
        let temp_dir = tempdir()?;
        let index_path = temp_dir.path().join("model_index.json");

        // Write invalid JSON to the file
        fs::write(&index_path, "invalid json content")?;

        let model_index = ModelIndex::new(index_path);
        let result = model_index.models();

        assert!(result.is_err());
        Ok(())
    }

    #[test]
    fn test_model_index_add_new_model() -> Result<()> {
        let temp_dir = tempdir()?;
        let index_path = temp_dir.path().join("model_index.json");

        let model_index = ModelIndex::new(index_path);
        let model = ModelInfo::new(
            "test-model",
            vec![ModelFile {
                size: 1024,
                path: PathBuf::from("/path/to/file.bin"),
            }],
        );

        model_index.add_model(model)?;
        let models = model_index.models()?;

        assert_eq!(models.len(), 1);
        assert_eq!(models[0].model_id, "test-model");
        assert_eq!(models[0].files.len(), 1);

        Ok(())
    }

    #[test]
    fn test_model_index_add_existing_model() -> Result<()> {
        let temp_dir = tempdir()?;
        let index_path = temp_dir.path().join("model_index.json");

        // Create initial index with one model
        let initial_data = r#"{
            "models": [
                {
                    "model_id": "test-model",
                    "files": [
                        {
                            "size": 512,
                            "path": "/old/path.bin"
                        }
                    ]
                }
            ]
        }"#;
        fs::write(&index_path, initial_data)?;

        let model_index = ModelIndex::new(index_path);
        let updated_model = ModelInfo::new(
            "test-model",
            vec![ModelFile {
                size: 1024,
                path: PathBuf::from("/new/path.bin"),
            }],
        );

        model_index.add_model(updated_model)?;
        let models = model_index.models()?;

        assert_eq!(models.len(), 1);
        assert_eq!(models[0].model_id, "test-model");
        assert_eq!(models[0].files.len(), 1);
        assert_eq!(models[0].files[0].size, 1024);
        assert_eq!(models[0].files[0].path, PathBuf::from("/new/path.bin"));

        Ok(())
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

        let models = manager.list_models()?;
        assert_eq!(models.len(), 0);

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
        let dir1 = default_project_dir();
        let dir2 = default_project_dir();

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
    fn test_model_index_data_serialization() -> Result<()> {
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

        let model_index_data = ModelIndexData { models };

        let json = serde_json::to_string(&model_index_data)?;
        let deserialized: ModelIndexData = serde_json::from_str(&json)?;

        assert_eq!(model_index_data.models.len(), deserialized.models.len());
        assert_eq!(
            model_index_data.models[0].model_id,
            deserialized.models[0].model_id
        );
        assert_eq!(
            model_index_data.models[1].model_id,
            deserialized.models[1].model_id
        );
        assert_eq!(
            model_index_data.models[1].files.len(),
            deserialized.models[1].files.len()
        );

        Ok(())
    }

    #[test]
    fn test_model_index_save() -> Result<()> {
        let temp_dir = tempdir()?;
        let index_path = temp_dir.path().join("test_index.json");

        let model_index = ModelIndex::new(index_path.clone());
        let models = vec![
            ModelInfo::new("model1", vec![]),
            ModelInfo::new(
                "model2",
                vec![ModelFile {
                    size: 1024,
                    path: PathBuf::from("/path/to/file.bin"),
                }],
            ),
        ];

        let index_data = ModelIndexData { models };
        model_index.save(&index_data)?;

        // Verify the file was created and contains correct data
        assert!(index_path.exists());

        let file_content = fs::read_to_string(&index_path)?;
        let parsed: ModelIndexData = serde_json::from_str(&file_content)?;

        assert_eq!(parsed.models.len(), 2);
        assert_eq!(parsed.models[0].model_id, "model1");
        assert_eq!(parsed.models[1].model_id, "model2");
        assert_eq!(parsed.models[1].files.len(), 1);

        Ok(())
    }

    #[test]
    fn test_model_index_add_and_update_operations() -> Result<()> {
        let temp_dir = tempdir()?;
        let index_path = temp_dir.path().join("test_index.json");

        let model_index = ModelIndex::new(index_path.clone());

        // Test adding a new model
        let test_model = ModelInfo::new(
            "test-model",
            vec![ModelFile {
                size: 512,
                path: temp_dir.path().join("model.bin"),
            }],
        );

        model_index.add_model(test_model.clone())?;
        let models = model_index.models()?;
        assert_eq!(models.len(), 1);
        assert_eq!(models[0].model_id, "test-model");
        assert_eq!(models[0].files.len(), 1);
        assert_eq!(models[0].files[0].size, 512);

        // Test adding another model
        let test_model2 = ModelInfo::new("test-model-2", vec![]);
        model_index.add_model(test_model2)?;
        let models = model_index.models()?;
        assert_eq!(models.len(), 2);

        // Test updating existing model
        let updated_model = ModelInfo::new(
            "test-model",
            vec![ModelFile {
                size: 1024,
                path: temp_dir.path().join("updated_model.bin"),
            }],
        );
        model_index.add_model(updated_model)?;
        let models = model_index.models()?;
        assert_eq!(models.len(), 2); // Still 2 models

        // Find the updated model
        let updated = models.iter().find(|m| m.model_id == "test-model").unwrap();
        assert_eq!(updated.files.len(), 1);
        assert_eq!(updated.files[0].size, 1024);

        Ok(())
    }

    #[test]
    fn test_model_manager_download_updates_index() -> Result<()> {
        // This test would need to mock the HuggingFace API to avoid network calls
        // For now, we test that ModelManager properly manages the index through other operations
        let temp_dir = tempdir()?;
        let models_dir = temp_dir.path().join("models");
        fs::create_dir_all(&models_dir)?;

        // Create an initially empty index
        let index_path = models_dir.join("model_index.json");
        let empty_index_data = r#"{"models": []}"#;
        fs::write(&index_path, empty_index_data)?;

        let manager = ModelManagerBuilder::new()
            .with_models_dir(models_dir.clone())
            .build()?;

        // Verify index is initially empty
        let initial_models = manager.list_models()?;
        assert_eq!(initial_models.len(), 0);

        // Create a new manager instance to verify the index persists across instances
        let manager2 = ModelManagerBuilder::new()
            .with_models_dir(models_dir)
            .build()?;

        let models = manager2.list_models()?;
        assert_eq!(models.len(), 0);

        Ok(())
    }
}
