use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use image::{DynamicImage, ImageBuffer, Rgb, RgbImage};
use log::{debug, info};
use palette::{FromColor, Hsl, Srgb};
use serde::{Deserialize, Serialize};

use crate::ModelManager;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TryOnRequest {
    pub input_image_path: PathBuf,
    pub clothing_description: String,
    pub output_path: PathBuf,
    pub model_name: Option<String>,
    pub strength: Option<f64>, // 0.0-1.0, how much to change the image
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TryOnResult {
    pub output_path: PathBuf,
    pub processing_time_ms: u64,
    pub model_used: String,
}

#[derive(Debug, Clone)]
struct ColorTransform {
    hue_shift: f32,
    saturation_mult: f32,
    lightness_mult: f32,
}

impl ColorTransform {
    fn new(hue_shift: f32, saturation_mult: f32, lightness_mult: f32) -> Self {
        Self {
            hue_shift,
            saturation_mult,
            lightness_mult,
        }
    }
}

pub struct VirtualTryOn {
    model_manager: ModelManager,
    current_model: Option<String>,
}

impl VirtualTryOn {
    pub fn new(model_manager: ModelManager) -> Result<Self> {
        info!("Initialized Virtual Try-On (MVP mode with image processing)");

        Ok(Self {
            model_manager,
            current_model: None,
        })
    }

    /// Load a model (for MVP, this just tracks which model the user wants to use)
    pub async fn load_model(&mut self, model_name: &str) -> Result<()> {
        info!("Loading model: {} (MVP mode)", model_name);

        // Check if model is already loaded
        if let Some(ref current) = self.current_model {
            if current == model_name {
                debug!("Model {} already loaded", model_name);
                return Ok(());
            }
        }

        // Ensure model is downloaded (for future use)
        let models = self.model_manager.list_models()?;
        let model_exists = models.iter().any(|m| m.model_id == model_name);

        if !model_exists {
            info!("Model {} not found locally, downloading...", model_name);
            self.model_manager.download_model(model_name).await?;
        }

        info!("Model {} ready (MVP mode)", model_name);
        self.current_model = Some(model_name.to_string());

        Ok(())
    }

    /// Perform virtual clothing try-on using image processing techniques
    pub async fn try_on(&mut self, request: TryOnRequest) -> Result<TryOnResult> {
        let start_time = std::time::Instant::now();

        info!(
            "Starting virtual try-on with prompt: {}",
            request.clothing_description
        );

        // Load default model if none specified
        let model_name = request
            .model_name
            .as_deref()
            .unwrap_or("runwayml/stable-diffusion-v1-5");

        self.load_model(model_name).await?;

        // Load input image
        let input_image = self.load_image(&request.input_image_path)?;

        // Apply clothing transformations
        let result_image = self.apply_clothing_transformation(
            &input_image,
            &request.clothing_description,
            request.strength.unwrap_or(0.5),
        )?;

        // Save result
        self.save_image(&result_image, &request.output_path)?;

        let processing_time = start_time.elapsed().as_millis() as u64;

        info!("Virtual try-on completed in {}ms", processing_time);

        Ok(TryOnResult {
            output_path: request.output_path,
            processing_time_ms: processing_time,
            model_used: model_name.to_string(),
        })
    }

    fn load_image(&self, path: &Path) -> Result<DynamicImage> {
        debug!("Loading image from: {}", path.display());
        image::open(path).with_context(|| format!("Failed to load image from {}", path.display()))
    }

    fn save_image(&self, img: &DynamicImage, path: &Path) -> Result<()> {
        debug!("Saving image to: {}", path.display());

        // Create parent directory if it doesn't exist
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create directory {}", parent.display()))?;
        }

        img.save(path)
            .with_context(|| format!("Failed to save image to {}", path.display()))
    }

    fn apply_clothing_transformation(
        &self,
        image: &DynamicImage,
        clothing_description: &str,
        strength: f64,
    ) -> Result<DynamicImage> {
        debug!("Applying clothing transformation: {}", clothing_description);

        // Convert to RGB for processing
        let rgb_image = image.to_rgb8();

        // Detect clothing regions (simplified approach for MVP)
        let clothing_mask = self.detect_clothing_regions(&rgb_image)?;

        // Extract clothing attributes from description
        let color_transform = self.extract_color_transform(clothing_description)?;
        let style_adjustments = self.extract_style_adjustments(clothing_description);

        // Apply transformations
        let transformed_image = self.apply_color_and_style_transformation(
            &rgb_image,
            &clothing_mask,
            &color_transform,
            &style_adjustments,
            strength as f32,
        )?;

        Ok(DynamicImage::ImageRgb8(transformed_image))
    }

    fn detect_clothing_regions(
        &self,
        image: &RgbImage,
    ) -> Result<ImageBuffer<image::Luma<u8>, Vec<u8>>> {
        // Simplified clothing detection for MVP
        // In a real implementation, this would use ML models for segmentation

        let (width, height) = image.dimensions();
        let mut mask = image::ImageBuffer::new(width, height);

        // Simple heuristic: assume clothing is in the middle region of the image
        // and has certain color characteristics
        for (x, y, pixel) in image.enumerate_pixels() {
            let is_clothing_region = self.is_likely_clothing_pixel(pixel, x, y, width, height);
            let mask_value = if is_clothing_region { 255 } else { 0 };
            mask.put_pixel(x, y, image::Luma([mask_value]));
        }

        debug!("Generated clothing mask");
        Ok(mask)
    }

    fn is_likely_clothing_pixel(
        &self,
        pixel: &Rgb<u8>,
        x: u32,
        y: u32,
        width: u32,
        height: u32,
    ) -> bool {
        // Simple heuristics for detecting clothing regions in MVP

        // Focus on middle region of image (typical clothing area)
        let x_ratio = x as f32 / width as f32;
        let y_ratio = y as f32 / height as f32;

        let in_clothing_region = x_ratio > 0.2 && x_ratio < 0.8 && y_ratio > 0.3 && y_ratio < 0.9;

        if !in_clothing_region {
            return false;
        }

        // Check if pixel looks like fabric (not skin or background)
        let [r, g, b] = pixel.0;
        let brightness = (r as f32 + g as f32 + b as f32) / 3.0;

        // Avoid skin tones (simplified)
        let is_skin_tone = r > 150
            && g > 100
            && b > 80
            && (r as i32 - g as i32).abs() < 50
            && (r as i32 - b as i32) < 80;

        // Avoid very bright or very dark regions (likely background)
        let reasonable_brightness = brightness > 30.0 && brightness < 240.0;

        !is_skin_tone && reasonable_brightness
    }

    fn extract_color_transform(&self, description: &str) -> Result<ColorTransform> {
        let desc_lower = description.to_lowercase();

        // Extract color information from description
        let (hue_shift, saturation_mult, lightness_mult) = if desc_lower.contains("red") {
            (0.0, 1.3, 1.0) // Enhance red
        } else if desc_lower.contains("blue") {
            (240.0, 1.2, 0.95) // Shift towards blue
        } else if desc_lower.contains("green") {
            (120.0, 1.2, 1.0) // Shift towards green
        } else if desc_lower.contains("yellow") {
            (60.0, 1.4, 1.1) // Shift towards yellow, brighten
        } else if desc_lower.contains("purple") || desc_lower.contains("violet") {
            (280.0, 1.3, 0.9) // Shift towards purple
        } else if desc_lower.contains("orange") {
            (30.0, 1.3, 1.05) // Shift towards orange
        } else if desc_lower.contains("pink") {
            (320.0, 1.2, 1.1) // Shift towards pink, brighten
        } else if desc_lower.contains("black") {
            (0.0, 0.8, 0.4) // Darken significantly
        } else if desc_lower.contains("white") {
            (0.0, 0.5, 1.6) // Desaturate and brighten
        } else if desc_lower.contains("gray") || desc_lower.contains("grey") {
            (0.0, 0.3, 0.8) // Desaturate and slightly darken
        } else {
            (0.0, 1.0, 1.0) // No change
        };

        Ok(ColorTransform::new(
            hue_shift,
            saturation_mult,
            lightness_mult,
        ))
    }

    fn extract_style_adjustments(&self, description: &str) -> (f32, f32) {
        let desc_lower = description.to_lowercase();

        // (contrast_mult, brightness_offset)
        if desc_lower.contains("silk") || desc_lower.contains("satin") {
            (1.15, 0.05) // Higher contrast, slight brightness boost
        } else if desc_lower.contains("leather") {
            (1.25, -0.1) // High contrast, darker
        } else if desc_lower.contains("denim") {
            (1.1, -0.05) // Slight contrast boost, slightly darker
        } else if desc_lower.contains("cotton") {
            (1.05, 0.02) // Subtle adjustments
        } else if desc_lower.contains("velvet") {
            (1.2, -0.08) // Higher contrast, darker
        } else if desc_lower.contains("linen") {
            (0.95, 0.08) // Lower contrast, brighter
        } else {
            (1.0, 0.0) // No change
        }
    }

    fn apply_color_and_style_transformation(
        &self,
        image: &RgbImage,
        mask: &ImageBuffer<image::Luma<u8>, Vec<u8>>,
        color_transform: &ColorTransform,
        style_adjustments: &(f32, f32),
        strength: f32,
    ) -> Result<RgbImage> {
        let (_width, _height) = image.dimensions();
        let mut result = image.clone();
        let (contrast_mult, brightness_offset) = *style_adjustments;

        for (x, y, pixel) in image.enumerate_pixels() {
            let mask_pixel = mask.get_pixel(x, y);
            let mask_strength = (mask_pixel.0[0] as f32 / 255.0) * strength;

            if mask_strength > 0.1 {
                // Transform this pixel
                let transformed_pixel = self.transform_pixel(
                    pixel,
                    color_transform,
                    contrast_mult,
                    brightness_offset,
                    mask_strength,
                )?;
                result.put_pixel(x, y, transformed_pixel);
            }
        }

        debug!("Applied color and style transformation");
        Ok(result)
    }

    fn transform_pixel(
        &self,
        pixel: &Rgb<u8>,
        color_transform: &ColorTransform,
        contrast_mult: f32,
        brightness_offset: f32,
        strength: f32,
    ) -> Result<Rgb<u8>> {
        let [r, g, b] = pixel.0;

        // Convert to HSL for color manipulation
        let rgb = Srgb::new(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0);
        let hsl = Hsl::from_color(rgb);

        // Apply color transformations
        let new_hue = if color_transform.hue_shift != 0.0 {
            (hsl.hue.into_positive_degrees() + color_transform.hue_shift * strength) % 360.0
        } else {
            hsl.hue.into_positive_degrees()
        };

        let new_saturation = (hsl.saturation
            * (1.0 + (color_transform.saturation_mult - 1.0) * strength))
            .clamp(0.0, 1.0);

        let new_lightness = (hsl.lightness
            * (1.0 + (color_transform.lightness_mult - 1.0) * strength))
            .clamp(0.0, 1.0);

        // Create new HSL color
        let new_hsl = Hsl::new(new_hue, new_saturation, new_lightness);
        let new_rgb = Srgb::from_color(new_hsl);

        // Apply contrast and brightness adjustments
        let final_r = ((new_rgb.red * contrast_mult + brightness_offset) * strength
            + (r as f32 / 255.0) * (1.0 - strength))
            .clamp(0.0, 1.0);
        let final_g = ((new_rgb.green * contrast_mult + brightness_offset) * strength
            + (g as f32 / 255.0) * (1.0 - strength))
            .clamp(0.0, 1.0);
        let final_b = ((new_rgb.blue * contrast_mult + brightness_offset) * strength
            + (b as f32 / 255.0) * (1.0 - strength))
            .clamp(0.0, 1.0);

        Ok(Rgb([
            (final_r * 255.0) as u8,
            (final_g * 255.0) as u8,
            (final_b * 255.0) as u8,
        ]))
    }

    /// Get recommended models for virtual try-on
    pub fn get_recommended_models() -> Vec<&'static str> {
        vec![
            "runwayml/stable-diffusion-v1-5",
            "stabilityai/stable-diffusion-2-1",
            "stabilityai/stable-diffusion-xl-base-1.0",
        ]
    }

    /// Get current device info for debugging
    pub fn device_info(&self) -> String {
        "CPU (Image Processing Mode)".to_string()
    }
}

// Helper functions for the CLI integration
impl VirtualTryOn {
    pub fn validate_input_image(path: &Path) -> Result<()> {
        if !path.exists() {
            return Err(anyhow::anyhow!(
                "Input image does not exist: {}",
                path.display()
            ));
        }

        // Check if it's a valid image format
        match image::open(path) {
            Ok(_) => Ok(()),
            Err(e) => Err(anyhow::anyhow!("Invalid image format: {}", e)),
        }
    }

    pub fn suggest_output_path(input_path: &Path, clothing_description: &str) -> PathBuf {
        let input_stem = input_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("output");

        let safe_description = clothing_description
            .chars()
            .filter(|c| c.is_alphanumeric() || *c == ' ')
            .collect::<String>()
            .replace(' ', "_")
            .to_lowercase();

        let output_name = format!("{}_{}_tryon.png", input_stem, safe_description);

        input_path
            .parent()
            .map(|p| p.join(output_name))
            .unwrap_or_else(|| PathBuf::from(output_name))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_extract_color_transform() {
        let temp_dir = tempdir().unwrap();
        let model_manager = crate::ModelManagerBuilder::new()
            .with_models_dir(temp_dir.path().to_path_buf())
            .build()
            .unwrap();

        let tryon = VirtualTryOn::new(model_manager).unwrap();

        let red_transform = tryon.extract_color_transform("red dress").unwrap();
        assert_eq!(red_transform.hue_shift, 0.0);
        assert!(red_transform.saturation_mult > 1.0);

        let blue_transform = tryon.extract_color_transform("blue shirt").unwrap();
        assert_eq!(blue_transform.hue_shift, 240.0);
    }

    #[test]
    fn test_extract_style_adjustments() {
        let temp_dir = tempdir().unwrap();
        let model_manager = crate::ModelManagerBuilder::new()
            .with_models_dir(temp_dir.path().to_path_buf())
            .build()
            .unwrap();

        let tryon = VirtualTryOn::new(model_manager).unwrap();

        let (contrast, brightness) = tryon.extract_style_adjustments("silk dress");
        assert!(contrast > 1.0);
        assert!(brightness > 0.0);

        let (contrast2, brightness2) = tryon.extract_style_adjustments("leather jacket");
        assert!(contrast2 > 1.0);
        assert!(brightness2 < 0.0);
    }

    #[test]
    fn test_validate_input_image() {
        let temp_dir = tempdir().unwrap();
        let nonexistent_path = temp_dir.path().join("nonexistent.jpg");

        assert!(VirtualTryOn::validate_input_image(&nonexistent_path).is_err());
    }

    #[test]
    fn test_suggest_output_path() {
        let input_path = PathBuf::from("/path/to/person.jpg");
        let output_path = VirtualTryOn::suggest_output_path(&input_path, "red silk dress");

        assert!(output_path.to_string_lossy().contains("person"));
        assert!(output_path.to_string_lossy().contains("red_silk_dress"));
        assert!(output_path.to_string_lossy().ends_with("_tryon.png"));
    }

    #[test]
    fn test_get_recommended_models() {
        let models = VirtualTryOn::get_recommended_models();
        assert!(!models.is_empty());
        assert!(models.contains(&"runwayml/stable-diffusion-v1-5"));
    }

    #[test]
    fn test_is_likely_clothing_pixel() {
        let temp_dir = tempdir().unwrap();
        let model_manager = crate::ModelManagerBuilder::new()
            .with_models_dir(temp_dir.path().to_path_buf())
            .build()
            .unwrap();

        let tryon = VirtualTryOn::new(model_manager).unwrap();

        // Test clothing-like pixel in clothing region
        let clothing_pixel = Rgb([100, 80, 120]); // Purple-ish
        assert!(tryon.is_likely_clothing_pixel(&clothing_pixel, 200, 300, 400, 600));

        // Test skin-tone pixel
        let skin_pixel = Rgb([200, 170, 150]); // Skin tone
        assert!(!tryon.is_likely_clothing_pixel(&skin_pixel, 200, 300, 400, 600));

        // Test pixel outside clothing region
        assert!(!tryon.is_likely_clothing_pixel(&clothing_pixel, 50, 100, 400, 600));
    }
}
