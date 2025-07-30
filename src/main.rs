use std::path::PathBuf;

use anyhow::{Context, Result, anyhow};
use clap::{Parser, Subcommand};
use hf_hub::api::tokio::Api;
use log::debug;

use crate::models::ModelManager;

mod models;

#[derive(Parser)]
#[command(name = "si")]
#[command(about = "A CLI for the Si (see) AI image generator")]
#[command(version = "0.1.0")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Model-related operations
    Model {
        #[command(subcommand)]
        action: ModelCommands,
    },
    /// Configuration management
    Config {
        #[command(subcommand)]
        action: ConfigCommands,
    },
    /// Image-related operations
    Image {
        #[command(subcommand)]
        action: ImageCommands,
    },
}

#[derive(Subcommand)]
enum ModelCommands {
    /// List available models
    List,
    /// Download a new model
    Download {
        /// Name of the model to download
        name: String,
    },
    /// Delete a model
    Delete {
        /// Name of the model to delete
        name: String,
    },
    /// Show model details
    Show {
        /// Name of the model to show
        name: String,
    },
}

#[derive(Subcommand)]
enum ConfigCommands {
    /// Show current configuration
    Show,
    /// Set a configuration value
    Set {
        /// Configuration key
        key: String,
        /// Configuration value
        value: String,
    },
    /// Get a configuration value
    Get {
        /// Configuration key
        key: String,
    },
    /// Reset configuration to defaults
    Reset,
}

#[derive(Subcommand)]
enum ImageCommands {
    /// Generate an image
    Generate {
        /// Prompt for the image generation
        prompt: String,
        /// Model to use for generation
        #[arg(short, long)]
        model: String,
        /// Input image file (jpg, png, gif, etc.)
        #[arg(short, long)]
        input: PathBuf,
        /// Output image file
        #[arg(short, long)]
        output: PathBuf,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    let hf = Api::new().context("Failed to create HuggingFace client")?;
    let model = hf.model(String::from("openai/clip-vit-base-patch32"));
    let info = model
        .info()
        .await
        .context("Failed to download model from HuggingFace")?;
    for file in info.siblings {
        println!("downloading file: {}...", file.rfilename);
        model
            .download(&file.rfilename)
            .await
            .context(format!("Failed to download file: {}", file.rfilename))?;
        println!("   done");
    }

    let cli = Cli::parse();

    match cli.command {
        Commands::Model { action } => handle_model_command(action).await,
        Commands::Config { action } => handle_config_command(action),
        Commands::Image { action } => handle_image_command(action),
    }
}

async fn handle_model_command(action: ModelCommands) -> Result<()> {
    let model_manager = ModelManager::new()?;
    match action {
        ModelCommands::List => {
            let models = model_manager
                .list_models()
                .map_err(|e| anyhow::anyhow!("Failed to list models: {}", e))?;

            for model in models {
                println!("Model: {}", model.model_id);
                println!("  Files:");
                for file in &model.files {
                    let file_name = file
                        .path
                        .file_name()
                        .ok_or_else(|| anyhow!("Illegal file path: {}", file.path.display()))?;
                    println!(
                        "    - {} ({})",
                        file_name.display(),
                        humansize::format_size(file.size, humansize::DECIMAL)
                    );
                }
            }
        }
        ModelCommands::Download { name } => {
            let model_info = model_manager.download_model(&name).await?;
            debug!("Downloaded model: {:?}", model_info);
            println!("Model {name} downloaded successfully.");
        }
        ModelCommands::Delete { name } => {
            println!("Deleting model: {}", name);
            // TODO: Implement model deletion logic
        }
        ModelCommands::Show { name } => {
            println!("Showing details for model: {}", name);
            // TODO: Implement model show logic
        }
    }
    Ok(())
}

fn handle_config_command(action: ConfigCommands) -> Result<()> {
    match action {
        ConfigCommands::Show => {
            println!("Showing current configuration...");
            // TODO: Implement config show logic
        }
        ConfigCommands::Set { key, value } => {
            println!("Setting config: {} = {}", key, value);
            // TODO: Implement config set logic
        }
        ConfigCommands::Get { key } => {
            println!("Getting config value for: {}", key);
            // TODO: Implement config get logic
        }
        ConfigCommands::Reset => {
            println!("Resetting configuration to defaults...");
            // TODO: Implement config reset logic
        }
    }
    Ok(())
}

fn handle_image_command(action: ImageCommands) -> Result<()> {
    match action {
        ImageCommands::Generate {
            prompt,
            model,
            input,
            output,
        } => {
            println!("Generating image with prompt: {}", prompt);
            println!("Using model: {}", model);
            println!("Input image: {}", input.display());
            println!("Output image: {}", output.display());
            // TODO: Implement image generation logic
        }
    }
    Ok(())
}
