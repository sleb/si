//! Si - A CLI tool for AI image generation
//!
//! This library provides the core functionality for managing AI models
//! and generating images locally.

pub mod models;

pub use models::{ModelFile, ModelInfo, ModelManager, ModelManagerBuilder, SyncResult};
