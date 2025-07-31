use std::{fmt::Debug, path::PathBuf};

use anyhow::{anyhow, Context, Result};
use clap::{Parser, Subcommand};

use log::debug;
use si::ModelManager;

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

    let cli = Cli::parse();

    match cli.command {
        Commands::Model { action } => handle_model_command(action).await,
        Commands::Config { action } => handle_config_command(action),
        Commands::Image { action } => handle_image_command(action),
    }
    .log_error()
}

async fn handle_model_command(action: ModelCommands) -> Result<()> {
    let model_manager = ModelManager::new()?;
    match action {
        ModelCommands::List => {
            let models = model_manager
                .list_models()
                .context("Failed to list models")?;

            if models.is_empty() {
                println!("No models available.");
                return Ok(());
            }

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

trait LogError<T> {
    fn log_error(self) -> Self;
}

impl<T, E: Debug> LogError<T> for Result<T, E> {
    fn log_error(self) -> Self {
        self.inspect_err(|e| debug!("{:?}", e))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use tempfile::tempdir;

    #[test]
    fn test_handle_config_show() {
        let action = ConfigCommands::Show;
        let result = handle_config_command(action);
        assert!(result.is_ok());
    }

    #[test]
    fn test_handle_config_set() {
        let action = ConfigCommands::Set {
            key: "test_key".to_string(),
            value: "test_value".to_string(),
        };
        let result = handle_config_command(action);
        assert!(result.is_ok());
    }

    #[test]
    fn test_handle_config_get() {
        let action = ConfigCommands::Get {
            key: "test_key".to_string(),
        };
        let result = handle_config_command(action);
        assert!(result.is_ok());
    }

    #[test]
    fn test_handle_config_reset() {
        let action = ConfigCommands::Reset;
        let result = handle_config_command(action);
        assert!(result.is_ok());
    }

    #[test]
    fn test_handle_image_generate() {
        let temp_dir = tempdir().unwrap();
        let input_path = temp_dir.path().join("input.jpg");
        let output_path = temp_dir.path().join("output.png");

        let action = ImageCommands::Generate {
            prompt: "A beautiful sunset".to_string(),
            model: "test-model".to_string(),
            input: input_path,
            output: output_path,
        };

        let result = handle_image_command(action);
        assert!(result.is_ok());
    }

    #[test]
    fn test_cli_parsing() {
        // Test that the CLI can be parsed (this tests the derive macros)
        use clap::CommandFactory;
        let _cmd = Cli::command();
    }

    #[test]
    fn test_model_commands_variants() {
        // Test all ModelCommands variants can be created
        let _list = ModelCommands::List;
        let _download = ModelCommands::Download {
            name: "test".to_string(),
        };
        let _delete = ModelCommands::Delete {
            name: "test".to_string(),
        };
        let _show = ModelCommands::Show {
            name: "test".to_string(),
        };
    }

    #[test]
    fn test_config_commands_variants() {
        // Test all ConfigCommands variants can be created
        let _show = ConfigCommands::Show;
        let _set = ConfigCommands::Set {
            key: "key".to_string(),
            value: "value".to_string(),
        };
        let _get = ConfigCommands::Get {
            key: "key".to_string(),
        };
        let _reset = ConfigCommands::Reset;
    }

    #[test]
    fn test_image_commands_variants() {
        // Test all ImageCommands variants can be created
        let _generate = ImageCommands::Generate {
            prompt: "test".to_string(),
            model: "model".to_string(),
            input: PathBuf::from("input.jpg"),
            output: PathBuf::from("output.png"),
        };
    }

    #[test]
    fn test_commands_variants() {
        // Test all Commands variants can be created
        let _model = Commands::Model {
            action: ModelCommands::List,
        };
        let _config = Commands::Config {
            action: ConfigCommands::Show,
        };
        let _image = Commands::Image {
            action: ImageCommands::Generate {
                prompt: "test".to_string(),
                model: "model".to_string(),
                input: PathBuf::from("input.jpg"),
                output: PathBuf::from("output.png"),
            },
        };
    }
}
