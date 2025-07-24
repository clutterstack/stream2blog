use crate::image_preview::ImagePreview;
use crate::key_handler::{KeyHandler, KeyResult};
use ratatui::{
    crossterm::event::{KeyCode, KeyEvent, MouseEvent},
    layout::Rect,
    widgets::{Block, Borders},
};
use tui_textarea::TextArea;

pub struct TextEditor {
    textarea: TextArea<'static>,
    area: Option<Rect>,
    image_preview: ImagePreview,
    show_image_preview: bool,
}


impl TextEditor {
    pub fn new() -> Self {
        let mut textarea = TextArea::default();
        textarea.set_block(Block::default().borders(Borders::ALL).title("Input"));
        textarea.set_max_histories(50);
        
        // Enable wrapping by default with a reasonable width
        textarea.set_wrap(true);
        textarea.set_wrap_width(Some(80));

        // Set visible selection styling for drag selection
        textarea.set_selection_style(
            ratatui::style::Style::default()
                .bg(ratatui::style::Color::LightBlue)
                .fg(ratatui::style::Color::Black)
        );

        // Try using default cursor style like the working demo
        log::info!("TextEditor::new() - Using default cursor style (no custom styling)");

        Self {
            textarea,
            area: None,
            image_preview: ImagePreview::new(),
            show_image_preview: true,
        }
    }

    pub fn set_block(&mut self, block: Block<'static>) {
        self.textarea.set_block(block);
    }

    pub fn set_area(&mut self, area: Rect) {
        log::info!("TextEditor::set_area - Setting area to: {area:?}");
        self.area = Some(area);
        
        // Auto-enable wrapping based on new area size
        self.enable_wrapping();
    }

    pub fn widget(&self) -> &TextArea<'static> {
        // Remove logging that might interfere with rendering
        &self.textarea
    }

    pub fn lines(&self) -> &[String] {
        self.textarea.lines()
    }

    pub fn clear(&mut self) {
        self.select_all();
        self.cut();
        self.cancel_selection();
        // Clear any loaded image when clearing text editor
        self.image_preview.clear();
        self.show_image_preview = false;
    }


    pub fn set_text(&mut self, text: &str) {
        self.clear();
        self.insert_str(text);

        // Check if the text contains an image reference and load it into preview
        self.extract_and_load_image_from_text(text);
    }

    /// Sets text content without processing images - used when cached thumbnails are already loaded.
    /// This prevents redundant image loading when we want to preserve pre-cached image data.
    pub fn set_text_without_image_processing(&mut self, text: &str) {
        // Clear only the text content, not the image preview
        self.select_all();
        self.cut();
        self.cancel_selection();
        
        // Insert the new text
        self.insert_str(text);
    }

    pub fn move_cursor_to_start(&mut self) {
        // Move cursor to the start of the document
        self.move_cursor(tui_textarea::CursorMove::Top);
        self.move_cursor(tui_textarea::CursorMove::Head);
    }

    fn extract_and_load_image_from_text(&mut self, text: &str) {
        // Look for markdown image syntax: ![](path) - find the first valid image
        let mut pos = 0;
        while let Some(start) = text[pos..].find("![](") {
            let abs_start = pos + start;
            if let Some(end) = text[abs_start + 4..].find(')') {
                let image_path = &text[abs_start + 4..abs_start + 4 + end];
                log::debug!("Found image reference in text: {image_path}");

                // Only load if it's a file path (not a URL)
                if !image_path.starts_with("http") {
                    if let Some(resolved_path) = self.resolve_image_path(image_path) {
                        log::debug!("Resolved image path: {resolved_path}");
                        if let Err(e) = self.load_image(&resolved_path) {
                            log::error!("Failed to load image from resolved path: {e}");
                        } else {
                            log::debug!(
                                "Successfully loaded image from resolved path: {resolved_path}"
                            );
                            // Successfully loaded first image, show it by default for authoring
                            self.show_image_preview = true;
                            self.image_preview.set_visible(true);
                            return; // Load first valid image found
                        }
                    } else {
                        log::warn!("Could not resolve image path: {image_path}");
                    }
                }
                pos = abs_start + 4 + end;
            } else {
                pos = abs_start + 4;
            }
        }
    }

    fn resolve_image_path(&self, image_path: &str) -> Option<String> {
        let path = std::path::Path::new(image_path);

        // Try the path as-is first (handles absolute paths and relative paths that exist)
        if path.exists() {
            return Some(image_path.to_string());
        }

        // Try as relative to current directory
        let current_dir = std::env::current_dir().ok()?;
        let full_path = current_dir.join(image_path);
        if full_path.exists() {
            return Some(full_path.to_string_lossy().to_string());
        }

        // If it's just a filename, try to find it in the organized directory structure
        if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
            // Search in organized directories - try common patterns
            let search_paths = [
                // Global directory
                current_dir.join("images/global").join(filename),
                // Legacy - current directory
                current_dir.join(filename),
            ];

            for search_path in &search_paths {
                if search_path.exists() {
                    return Some(search_path.to_string_lossy().to_string());
                }
            }

            // Search in all thread directories (this is more expensive but thorough)
            if let Ok(images_dir) = std::fs::read_dir(current_dir.join("images/threads")) {
                for thread_entry in images_dir.flatten() {
                    if thread_entry.file_type().ok()?.is_dir() {
                        // Check direct thread directory
                        let thread_path = thread_entry.path().join(filename);
                        if thread_path.exists() {
                            return Some(thread_path.to_string_lossy().to_string());
                        }

                        // Check subdirectories (entry directories)
                        if let Ok(thread_contents) = std::fs::read_dir(thread_entry.path()) {
                            for entry_dir in thread_contents.flatten() {
                                if entry_dir.file_type().ok()?.is_dir() {
                                    let entry_path = entry_dir.path().join(filename);
                                    if entry_path.exists() {
                                        return Some(entry_path.to_string_lossy().to_string());
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        None
    }

    pub fn handle_key_event(&mut self, key: KeyEvent) -> Option<KeyResult> {
        log::debug!("TextEditor received key event: {key:?}");

        // Handle clipboard operations first (especially image paste)
        let has_existing_image = self.has_image();
        if let Some(result) = KeyHandler::handle_clipboard_keys(&mut self.textarea, key, has_existing_image) {
            match &result {
                KeyResult::ImageNamingModal(_) => {
                    log::debug!("Clipboard operation result: ImageNamingModal with image data")
                }
                KeyResult::ImageRemovalModal => {
                    log::debug!("Clipboard operation result: ImageRemovalModal for existing image")
                }
                other => log::debug!("Clipboard operation result: {other:?}"),
            }
            match result {
                KeyResult::ImageNamingModal(image_data) => {
                    return Some(KeyResult::ImageNamingModal(image_data));
                }
                KeyResult::ImageRemovalModal => {
                    return Some(KeyResult::ImageRemovalModal);
                }
                KeyResult::Handled(true) => {
                    // Successful clipboard operation handled
                    return Some(result);
                }
                KeyResult::Handled(false) => {
                    // Clipboard operation failed, fall through to normal textarea handling
                    log::debug!(
                        "Clipboard operation failed, falling through to normal textarea input"
                    );
                }
            }
        }

        // Handle visual line movement for wrapped text
        match key.code {
            KeyCode::Up => {
                if self.is_wrapping_enabled() && self.area.is_some() {
                    self.move_cursor(tui_textarea::CursorMove::VisualUp);
                } else {
                    let (cursor_row, _cursor_col) = self.cursor_position();
                    if cursor_row == 0 {
                        // If cursor is on first line, move to beginning
                        self.move_cursor(tui_textarea::CursorMove::Head);
                        return None;
                    }
                    // Let textarea handle normal Up movement
                    let _handled = self.textarea.input(key);
                }
            }
            KeyCode::Down => {
                if self.is_wrapping_enabled() && self.area.is_some() {
                    self.move_cursor(tui_textarea::CursorMove::VisualDown);
                } else {
                    let (cursor_row, _cursor_col) = self.cursor_position();
                    let lines = self.lines();
                    if cursor_row >= lines.len().saturating_sub(1) {
                        // If cursor is on last line, move to end
                        self.move_cursor(tui_textarea::CursorMove::End);
                        return None;
                    }
                    // Let textarea handle normal Down movement
                    let _handled = self.textarea.input(key);
                }
            }
            _ => {
                // For all other keys, use textarea's built-in input handling
                let _handled = self.textarea.input(key);
                log::debug!("Key event handled by textarea: {:?}", key.code);
            }
        }

        None
    }


    pub fn scroll_up(&mut self) {
        self.move_cursor(tui_textarea::CursorMove::Up);
    }

    pub fn scroll_down(&mut self) {
        self.move_cursor(tui_textarea::CursorMove::Down);
    }

    pub fn load_image(&mut self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        self.image_preview.load_image(path)?;
        // When an image is loaded, always show the preview by default for authoring
        self.show_image_preview = true;
        self.image_preview.set_visible(true);
        log::debug!(
            "Image loaded and preview enabled for authoring - show_image_preview: {}",
            self.show_image_preview
        );
        Ok(())
    }

    pub fn toggle_image_preview(&mut self) {
        if self.image_preview.has_image() {
            // If no cursor on image but we have an image loaded, allow manual toggle
            self.show_image_preview = !self.show_image_preview;
            log::debug!(
                "Manual toggle: image preview visibility to: {}",
                self.show_image_preview
            );
            self.image_preview.set_visible(self.show_image_preview);
        } else {
            log::debug!(
                "Toggle ignored: no image found for entry."
            );
        }
    }
  

    pub fn has_image(&self) -> bool {
        
        //log::debug!("TextEditor::has_image() returning: {}", has_img);
        self.image_preview.has_image()
    }

    pub fn is_image_preview_visible(&self) -> bool {
        self.show_image_preview
    }

    pub fn set_image_preview_visible(&mut self, visible: bool) {
        self.show_image_preview = visible;
    }

    pub fn image_preview_mut(&mut self) -> &mut ImagePreview {
        &mut self.image_preview
    }

    pub fn toggle_image_full_screen(&mut self) {
        self.image_preview.toggle_full_screen();
    }

    /// Enable text wrapping and set wrap width
    pub fn set_wrap_width(&mut self, width: usize) {
        self.textarea.set_wrap(true);
        self.textarea.set_wrap_width(Some(width));
        log::debug!("Text wrapping enabled with width: {width}");
    }

    /// Enable wrapping based on current area width
    pub fn enable_wrapping(&mut self) {
        if let Some(area) = self.area {
            // Calculate usable width accounting for borders and padding
            let usable_width = area.width.saturating_sub(4) as usize; // 2 for borders, 2 for padding
            let wrap_width = usable_width.max(20); // Minimum width of 20 characters
            
            self.textarea.set_wrap(true);
            self.textarea.set_wrap_width(Some(wrap_width));
            log::debug!("Auto-enabled wrapping with calculated width: {wrap_width}");
        } else {
            // Default wrapping width if no area is set
            self.textarea.set_wrap(true);
            self.textarea.set_wrap_width(Some(80));
            log::debug!("Auto-enabled wrapping with default width: 80");
        }
    }

    /// Check if wrapping is enabled
    pub fn is_wrapping_enabled(&self) -> bool {
        self.textarea.wrap_enabled()
    }

    /// Cancel current text selection
    pub fn cancel_selection(&mut self) {
        self.textarea.cancel_selection();
    }

    /// Select all text
    pub fn select_all(&mut self) {
        self.textarea.select_all();
    }

    /// Cut selected text (returns true if something was cut)
    pub fn cut(&mut self) -> bool {
        self.textarea.cut()
    }

    /// Get cursor position as (row, column)
    pub fn cursor_position(&self) -> (usize, usize) {
        self.textarea.cursor()
    }

    /// Get visual cursor position accounting for text wrapping
    /// Returns (visual_row, visual_col) for display purposes  
    /// Uses tui-textarea's built-in logical_to_screen_position method
    pub fn visual_cursor_position(&self, area_width: u16, area_height: u16) -> Option<(u16, u16)> {
        self.textarea.logical_to_screen_position(area_width, area_height)
    }

    /// Move cursor using textarea's built-in movement
    pub fn move_cursor(&mut self, movement: tui_textarea::CursorMove) {
        self.textarea.move_cursor(movement);
    }

    /// Insert text at cursor position
    pub fn insert_str(&mut self, text: &str) {
        self.textarea.insert_str(text);
    }


    /// Handle mouse events using textarea's built-in mouse support with drag selection
    pub fn handle_mouse_event(&mut self, mouse: MouseEvent) -> Result<bool, Box<dyn std::error::Error>> {
        use ratatui::crossterm::event::{MouseEventKind, MouseButton};
        use tui_textarea::Key;
        
        if let Some(area) = self.area {
            // Convert mouse events to tui-textarea Key events and handle with drag selection support
            let mouse_key = match mouse.kind {
                MouseEventKind::Down(MouseButton::Left) => {
                    Some(Key::MouseClick(mouse.column, mouse.row))
                }
                MouseEventKind::Drag(MouseButton::Left) => {
                    Some(Key::MouseDrag(mouse.column, mouse.row))
                }
                MouseEventKind::Up(MouseButton::Left) => {
                    Some(Key::MouseUp(mouse.column, mouse.row))
                }
                _ => None,
            };

            if let Some(key) = mouse_key {
                // Use the new handle_mouse_event method for drag selection
                let handled = self.textarea.handle_mouse_event(key, area);
                return Ok(handled);
            }
        } else {
            log::warn!("No area set for text editor - cannot handle mouse events");
        }
        
        // Fallback to general input method for other mouse events (scrolling, etc.)
        let handled = self.textarea.input(mouse);
        Ok(handled)
    }











}

impl Default for TextEditor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::layout::Rect;

    fn create_test_editor() -> TextEditor {
        let mut editor = TextEditor::new();
        editor.set_area(Rect::new(0, 0, 50, 10));
        editor
    }

    #[test]
    fn test_wrapping_functionality() {
        let mut editor = create_test_editor();
        
        // Test that wrapping is enabled by default
        assert!(editor.is_wrapping_enabled());
        
        // Test setting wrap width
        editor.set_wrap_width(40);
        assert!(editor.is_wrapping_enabled());
        
        // Test re-enabling wrapping
        editor.enable_wrapping();
        assert!(editor.is_wrapping_enabled());
    }

    #[test]
    fn test_word_navigation_ctrl_keys() {
        use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
        
        let mut editor = create_test_editor();
        
        // Set test text with multiple words
        editor.set_text("hello world test content");
        
        // Move cursor to end of text
        editor.move_cursor(tui_textarea::CursorMove::End);
        let (initial_row, initial_col) = editor.cursor_position();
        println!("Initial cursor position: row={}, col={}", initial_row, initial_col);
        
        // Test Ctrl+Left (should move cursor to start of "content")
        let key = KeyEvent::new(KeyCode::Left, KeyModifiers::CONTROL);
        let result = editor.handle_key_event(key);
        let (row_after_left, col_after_left) = editor.cursor_position();
        
        println!("After Ctrl+Left: row={}, col={}, result={:?}", row_after_left, col_after_left, result);
        
        // The cursor should have moved to the beginning of "content" (position 17)
        assert_eq!(row_after_left, 0);
        assert_eq!(col_after_left, 17); // Start of "content"
        assert!(matches!(result, Some(crate::key_handler::KeyResult::Handled(true))));
        
        // Test another Ctrl+Left (should move to start of "test")
        let key = KeyEvent::new(KeyCode::Left, KeyModifiers::CONTROL);
        let result = editor.handle_key_event(key);
        let (row_after_left2, col_after_left2) = editor.cursor_position();
        
        println!("After 2nd Ctrl+Left: row={}, col={}, result={:?}", row_after_left2, col_after_left2, result);
        
        // The cursor should have moved to the beginning of "test" (position 12)
        assert_eq!(row_after_left2, 0);
        assert_eq!(col_after_left2, 12); // Start of "test"
        assert!(matches!(result, Some(crate::key_handler::KeyResult::Handled(true))));
        
        // Test Ctrl+Right (should move back to start of "content")
        let key = KeyEvent::new(KeyCode::Right, KeyModifiers::CONTROL);
        let result = editor.handle_key_event(key);
        let (row_after_right, col_after_right) = editor.cursor_position();
        
        println!("After Ctrl+Right: row={}, col={}, result={:?}", row_after_right, col_after_right, result);
        
        // The cursor should have moved to the start of "content" (position 17)
        assert_eq!(row_after_right, 0);
        assert_eq!(col_after_right, 17); // Start of "content"
        assert!(matches!(result, Some(crate::key_handler::KeyResult::Handled(true))));
    }

   
  
   

}
