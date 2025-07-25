use crate::clipboard_manager::get_clipboard_manager;
use image::DynamicImage;
use std::fs;

pub fn get_image_from_clipboard(
) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync + 'static>> {
    let image = get_clipboard_manager().get_image()?;

    let buffer = image.bytes.into_owned();
    let (width, height) = (image.width as u32, image.height as u32);

    let dyn_img = DynamicImage::ImageRgba8(
        image::ImageBuffer::from_raw(width, height, buffer)
            .ok_or("Failed to create image buffer")?,
    );

    // Resize to a max width of 512 px, preserving aspect ratio
    // let's actually not right now
    //let resized = dyn_img.resize(512, u32::MAX, image::imageops::Lanczos3);

    // Encode to PNG bytes
    let mut png_bytes = Vec::new();
    // resized.write_to(
    dyn_img.write_to(
        &mut std::io::Cursor::new(&mut png_bytes),
        image::ImageFormat::Png,
    )?;

    Ok(png_bytes)
}

pub fn save_image_with_context(
    image_data: &[u8],
    filename: &str,
    thread_id: Option<&str>,
    entry_id: Option<&str>,
) -> Result<String, Box<dyn std::error::Error + Send + Sync + 'static>> {
    let current_dir = std::env::current_dir()?;

    // Ensure filename has .png extension
    let filename = if filename.ends_with(".png") {
        filename.to_string()
    } else {
        format!("{filename}.png")
    };

    // Create organized directory structure
    let image_dir = match (thread_id, entry_id) {
        (Some(t_id), Some(e_id)) => {
            // For editing existing entry: images/threads/{thread_id}/{entry_id}/
            current_dir
                .join("images")
                .join("threads")
                .join(t_id)
                .join(e_id)
        }
        (Some(t_id), None) => {
            // For creating new entry: images/threads/{thread_id}/
            current_dir.join("images").join("threads").join(t_id)
        }
        (None, None) => {
            // For creating new thread or global images: images/global/
            current_dir.join("images").join("global")
        }
        (None, Some(_)) => {
            // Shouldn't happen, but fallback to global
            current_dir.join("images").join("global")
        }
    };

    // Create directory structure if it doesn't exist
    fs::create_dir_all(&image_dir)?;

    let output_path = image_dir.join(filename);
    std::fs::write(&output_path, image_data)?;

    // Return relative path for markdown reference
    let relative_path = output_path
        .strip_prefix(&current_dir)
        .unwrap_or(&output_path)
        .to_string_lossy()
        .to_string();

    Ok(relative_path)
}

pub fn delete_image_file(image_path: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
    let current_dir = std::env::current_dir()?;
    let full_path = current_dir.join(image_path);
    
    if full_path.exists() {
        fs::remove_file(&full_path)?;
        log::info!("Successfully deleted image file: {}", image_path);
    } else {
        log::warn!("Image file not found, skipping deletion: {}", image_path);
    }
    
    Ok(())
}
