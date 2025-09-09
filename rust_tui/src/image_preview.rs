use crate::block_styles::{content_block};
use ratatui::{
    layout::Rect,
    widgets::{Block, Paragraph},
    Frame,
};
use ratatui_image::{picker::Picker, protocol::StatefulProtocol, StatefulImage};
use std::path::Path;

pub struct ImagePreview {
    image_protocol: Option<StatefulProtocol>,
    visible: bool,
    error_message: Option<String>,
    image_bytes: Option<Vec<u8>>,
    image_format: Option<image::ImageFormat>,
    cached_protocol: Option<StatefulProtocol>,
    full_screen: bool,
    pub cached_image: Option<image::DynamicImage>,
    pub cached_picker: Option<Picker>,
}

impl ImagePreview {
    pub fn new() -> Self {
        Self {
            image_protocol: None,
            visible: false,
            error_message: None,
            image_bytes: None,
            image_format: None,
            cached_protocol: None,
            full_screen: false,
            cached_image: None,
            cached_picker: None,
        }
    }

    pub fn load_image(&mut self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        log::debug!("Loading image from path: {path}");
        if !Path::new(path).exists() {
            let error_msg = format!("Image file not found: {path}");
            log::error!("{error_msg}");
            self.error_message = Some(error_msg);
            self.image_protocol = None;
            return Ok(());
        }

        // Read image bytes and determine format
        let image_bytes = std::fs::read(path)?;
        match image::ImageReader::open(path)?
            .with_guessed_format()?
            .format()
        {
            Some(format) => {
                log::debug!("Successfully loaded image format: {format:?}");

                // Create DynamicImage and cache it for later protocol creation
                match image::load_from_memory_with_format(&image_bytes, format) {
                    Ok(dyn_img) => {
                        log::debug!(
                            "Created DynamicImage {}x{} at load time",
                            dyn_img.width(),
                            dyn_img.height()
                        );

                        self.image_bytes = Some(image_bytes);
                        self.image_format = Some(format);
                        self.cached_image = Some(dyn_img.clone());
                        self.cached_protocol = None; // Clear any previous protocol
                        self.error_message = None;
                        self.visible = true;

                        // Initialize the picker and create protocol once during image load
                        self.create_or_update_picker();
                        self.create_fixed_protocol(&dyn_img);

                        log::debug!("Image loaded successfully with fixed-size protocol created");
                    }
                    Err(e) => {
                        let error_msg = format!("Failed to create image at load time: {e}");
                        log::error!("{error_msg}");
                        self.error_message = Some(error_msg);
                        self.image_protocol = None;
                        self.cached_protocol = None;
                        self.cached_image = None;
                    }
                }
            }
            None => {
                let error_msg = "Could not determine image format".to_string();
                log::error!("{error_msg}");
                self.error_message = Some(error_msg);
                self.image_protocol = None;
                self.cached_protocol = None;
                self.cached_image = None;
            }
        }
        Ok(())
    }

    pub fn set_visible(&mut self, visible: bool) {
        self.visible = visible;
    }

    pub fn has_image(&self) -> bool {
        let has_img = self.cached_image.is_some();
        log::debug!(
            "ImagePreview::has_image() returning: {} (cached_image is_some: {})",
            has_img,
            self.cached_image.is_some()
        );
        has_img
    }

    pub fn clear(&mut self) {
        self.image_protocol = None;
        self.image_bytes = None;
        self.image_format = None;
        self.cached_protocol = None;
        self.cached_image = None;
        self.visible = false;
        self.error_message = None;
        self.full_screen = false;
        self.cached_picker = None;
    }

    pub fn toggle_full_screen(&mut self) {
        self.full_screen = !self.full_screen;
    }

    pub fn is_full_screen(&self) -> bool {
        self.full_screen
    }


    fn create_or_update_picker(&mut self) {
        // Create a picker that can be reused across renders
        match Picker::from_query_stdio() {
            Ok(picker) => {
                log::debug!("Successfully created picker from stdio query");
                self.cached_picker = Some(picker);
            }
            Err(e) => {
                log::debug!(
                    "Failed to query stdio for font size ({e}), using fallback picker"
                );
                self.cached_picker = Some(Picker::from_fontsize((8, 12)));
            }
        }
    }

    fn create_fixed_protocol(&mut self, image: &image::DynamicImage) {
        if let Some(ref picker) = self.cached_picker {
            // Define maximum bounds for image display
            let max_width = 512u32;
            let max_height = 512u32;

            let original_width = image.width();
            let original_height = image.height();

            // Check if image is smaller than our maximum bounds
            let final_image = if original_width <= max_width && original_height <= max_height {
                // Small image - use original size to avoid upscaling/pixelation
                log::debug!(
                    "Small image detected - using original size: {original_width}x{original_height}"
                );
                image.clone()
            } else {
                // Large image - scale down to fit within bounds (preserves aspect ratio)
                log::debug!(
                    "Large image detected - scaling down from {original_width}x{original_height} to fit within {max_width}x{max_height}"
                );
                image.resize(max_width, max_height, image::imageops::FilterType::Lanczos3)
            };

            log::debug!(
                "Creating protocol - Original: {}x{}, Final: {}x{}",
                original_width,
                original_height,
                final_image.width(),
                final_image.height()
            );

            // Create the protocol once with the appropriately sized image
            let protocol = picker.new_resize_protocol(final_image);
            self.cached_protocol = Some(protocol);

            log::debug!("Protocol created successfully");
        } else {
            log::error!("Cannot create protocol: no picker available");
        }
    }

    /// Creates a protocol from already cached image data without reprocessing.
    /// Used when copying cached thumbnails between ImagePreview instances.
    pub fn create_fixed_protocol_from_cached(&mut self) {
        if let (Some(ref image), Some(ref picker)) = (&self.cached_image, &self.cached_picker) {
            log::debug!("Creating protocol from cached image data");
            let protocol = picker.new_resize_protocol(image.clone());
            self.cached_protocol = Some(protocol);
            self.visible = true;
            log::debug!("Protocol created from cached data successfully");
        } else {
            log::error!("Cannot create protocol from cached data: missing image or picker");
        }
    }

    pub fn render(&mut self, frame: &mut Frame, area: Rect) {
        log::debug!(
            "ImagePreview::render called - visible: {}, has_image: {}",
            self.visible,
            self.has_image()
        );
        if !self.visible {
            log::debug!("ImagePreview not visible, skipping render");
            return;
        }

        let (block, inner_area) = if self.full_screen {
            // Full screen mode - no borders
            (Block::default(), area)
        } else {
            // Normal mode - with borders
            (
                content_block("Image Preview"),

                ratatui::layout::Rect {
                    x: area.x + 1,
                    y: area.y + 1,
                    width: area.width.saturating_sub(2),
                    height: area.height.saturating_sub(2),
                },
            )
        };

        if let Some(ref error) = self.error_message {
            let error_widget = Paragraph::new(error.as_str()).block(block.clone().title("Error"));
            frame.render_widget(error_widget, area);
            return;
        }

        if self.cached_image.is_some() {
            log::debug!(
                "Available render area: {}x{}, inner area: {}x{}",
                area.width,
                area.height,
                inner_area.width,
                inner_area.height
            );

            // Use the cached protocol directly - no expensive operations on render path
            if let Some(mut protocol) = self.cached_protocol.take() {
                // Render the image using the pre-created fixed-size protocol
                let image_widget = StatefulImage::default();
                frame.render_stateful_widget(image_widget, inner_area, &mut protocol);

                // Put the protocol back for next render
                self.cached_protocol = Some(protocol);

                // Render the block border around the image (only if not full screen)
                if !self.full_screen {
                    frame.render_widget(block, area);
                }
                log::debug!("Image render completed using fixed-size cached protocol");
            } else {
                log::error!("No cached protocol available for rendering");
                let error_widget =
                    Paragraph::new("Image protocol not ready").block(block.clone().title("Error"));
                frame.render_widget(error_widget, area);
            }
        } else {
            log::debug!("No cached image, showing 'No Image' placeholder");
            let empty_widget = Paragraph::new("No Image").block(block.clone());
            frame.render_widget(empty_widget, area);
        }
    }
}

impl Default for ImagePreview {
    fn default() -> Self {
        Self::new()
    }
}
