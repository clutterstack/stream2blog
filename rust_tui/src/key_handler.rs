use crate::clipboard_manager::get_clipboard_manager;
use crate::image_clip::get_image_from_clipboard;
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use tui_textarea::{TextArea, CursorMove};
pub struct KeyHandler;

#[derive(Debug)]
pub enum KeyResult {
    Handled(bool),
    ImageNamingModal(Vec<u8>), // Image data for naming modal
    ImageRemovalModal, // Show image removal modal for existing image
}

impl KeyHandler {
    pub fn handle_clipboard_keys(
        textarea: &mut TextArea<'static>,
        key: KeyEvent,
        has_existing_image: bool,
    ) -> Option<KeyResult> {
        // log::debug!("handle_clipboard_keys called with key: {:?}", key);
        match (key.code, key.modifiers) {
            // Handle Ctrl+C (copy)
            (KeyCode::Char('c'), KeyModifiers::CONTROL) => {
                log::debug!("Handling Ctrl+C copy operation");

                // First try to get selected text
                if let Some(selected_text) = Self::get_selected_text(textarea) {
                    log::debug!("Selected text found: '{selected_text}'");
                    if let Err(e) = Self::copy_to_system_clipboard(&selected_text) {
                        log::error!("Failed to copy to system clipboard: {e}");
                        return Some(KeyResult::Handled(false));
                    }
                    log::debug!(
                        "Successfully copied '{selected_text}' to system clipboard"
                    );
                } else {
                    // Fallback: if no selection, copy all text
                    log::debug!("No text selected, copying all text as fallback");
                    let all_text = textarea.lines().join("\n");
                    if !all_text.is_empty() {
                        if let Err(e) = Self::copy_to_system_clipboard(&all_text) {
                            log::error!("Failed to copy all text to system clipboard: {e}");
                            return Some(KeyResult::Handled(false));
                        }
                        log::debug!("Successfully copied all text to system clipboard");
                    } else {
                        log::debug!("No text to copy");
                        return Some(KeyResult::Handled(false));
                    }
                }
                Some(KeyResult::Handled(true))
            }
            // Handle Ctrl+X (cut)
            (KeyCode::Char('x'), KeyModifiers::CONTROL) => {
                log::debug!("Handling Ctrl+X cut operation");
                if let Some(selected_text) = Self::get_selected_text(textarea) {
                    if let Err(e) = Self::copy_to_system_clipboard(&selected_text) {
                        log::error!("Failed to copy to system clipboard: {e}");
                        return Some(KeyResult::Handled(false));
                    }
                    // Remove the selected text using tui-textarea's cut method
                    textarea.cut();
                    log::debug!("Successfully cut '{selected_text}' to system clipboard");
                } else {
                    log::debug!("No text selected for cut operation");
                    return Some(KeyResult::Handled(false));
                }
                Some(KeyResult::Handled(true))
            }
            // Handle Ctrl+V (paste)
            (KeyCode::Char('v'), KeyModifiers::CONTROL) => {
                log::debug!("Handling Ctrl+V paste operation");
                match Self::get_from_system_clipboard() {
                    Ok(clipboard_text) => {
                        Self::insert_text(textarea, &clipboard_text);
                        log::debug!("Successfully pasted from system clipboard");
                        Some(KeyResult::Handled(true))
                    }
                    Err(e) => {
                        log::error!("Failed to paste from system clipboard: {e}");
                        Some(KeyResult::Handled(false))
                    }
                }
            }
            // Handle Command+C (copy) - macOS
            // (KeyCode::Char('c'), KeyModifiers::SUPER) => {
            //     log::debug!("Handling Command+C copy operation");

            //     // First try to get selected text
            //     if let Some(selected_text) = Self::get_selected_text(textarea) {
            //         log::debug!("Selected text found: '{selected_text}'");
            //         if let Err(e) = Self::copy_to_system_clipboard(&selected_text) {
            //             log::error!("Failed to copy to system clipboard: {e}");
            //             return Some(KeyResult::Handled(false));
            //         }
            //         log::debug!(
            //             "Successfully copied '{selected_text}' to system clipboard"
            //         );
            //     } else {
            //         // Fallback: if no selection, copy all text
            //         log::debug!("No text selected, copying all text as fallback");
            //         let all_text = textarea.lines().join("\n");
            //         if !all_text.is_empty() {
            //             if let Err(e) = Self::copy_to_system_clipboard(&all_text) {
            //                 log::error!("Failed to copy all text to system clipboard: {e}");
            //                 return Some(KeyResult::Handled(false));
            //             }
            //             log::debug!("Successfully copied all text to system clipboard");
            //         } else {
            //             log::debug!("No text to copy");
            //             return Some(KeyResult::Handled(false));
            //         }
            //     }
            //     Some(KeyResult::Handled(true))
            // }
            // Handle Command+X (cut) - macOS
            // (KeyCode::Char('x'), KeyModifiers::SUPER) => {
            //     log::debug!("Handling Command+X cut operation");
            //     if let Some(selected_text) = Self::get_selected_text(textarea) {
            //         if let Err(e) = Self::copy_to_system_clipboard(&selected_text) {
            //             log::error!("Failed to copy to system clipboard: {e}");
            //             return Some(KeyResult::Handled(false));
            //         }
            //         // Remove the selected text using tui-textarea's cut method
            //         textarea.cut();
            //         log::debug!("Successfully cut '{selected_text}' to system clipboard");
            //     } else {
            //         log::debug!("No text selected for cut operation");
            //         return Some(KeyResult::Handled(false));
            //     }
            //     Some(KeyResult::Handled(true))
            // }
            // Handle Command+V (paste) - macOS
            // (KeyCode::Char('v'), KeyModifiers::SUPER) => {
            //     log::debug!("Handling Command+V paste operation");
            //     match Self::get_from_system_clipboard() {
            //         Ok(clipboard_text) => {
            //             Self::insert_text(textarea, &clipboard_text);
            //             log::debug!("Successfully pasted from system clipboard via Command+V");
            //             Some(KeyResult::Handled(true))
            //         }
            //         Err(e) => {
            //             log::error!("Failed to paste from system clipboard: {e}");
            //             Some(KeyResult::Handled(false))
            //         }
            //     }
            // }
            // Handle Ctrl+Z (undo/redo)
            (KeyCode::Char('z'), modifiers) if modifiers.contains(KeyModifiers::CONTROL) => {
                if modifiers.contains(KeyModifiers::SHIFT) {
                    log::debug!("Handling Ctrl+Shift+Z redo operation");
                    let modified = textarea.redo();
                    log::debug!("Redo operation modified text: {modified}");
                    Some(KeyResult::Handled(modified))
                } else {
                    log::debug!("Handling Ctrl+Z undo operation");
                    let modified = textarea.undo();
                    log::debug!("Undo operation modified text: {modified}");
                    Some(KeyResult::Handled(modified))
                }
            }
            // Handle Command+Z (undo/redo) - macOS
            // (KeyCode::Char('z'), modifiers) if modifiers.contains(KeyModifiers::SUPER) => {
            //     if modifiers.contains(KeyModifiers::SHIFT) {
            //         log::debug!("Handling Command+Shift+Z redo operation");
            //         let modified = textarea.redo();
            //         log::debug!("Redo operation modified text: {modified}");
            //         Some(KeyResult::Handled(modified))
            //     } else {
            //         log::debug!("Handling Command+Z undo operation");
            //         let modified = textarea.undo();
            //         log::debug!("Undo operation modified text: {modified}");
            //         Some(KeyResult::Handled(modified))
            //     }
            // }
            // Handle Ctrl+A (select all)
            (KeyCode::Char('a'), KeyModifiers::CONTROL) => {
                log::debug!("Handling Ctrl+A select all operation");
                textarea.select_all();
                Some(KeyResult::Handled(true))
            }
            // Handle Command+A (select all) - macOS
            // (KeyCode::Char('a'), KeyModifiers::SUPER) => {
            //     log::debug!("Handling Command+A select all operation");
            //     textarea.select_all();
            //     Some(KeyResult::Handled(true))
            // } 
            // Handle Command+A (select all) - macOS
            (KeyCode::Char('\x05'), _) => {
                log::debug!("Handling Command+A select all operation");
                textarea.select_all();
                Some(KeyResult::Handled(true))
            }
            // Handle Ctrl+P (insert image from clipboard)
            (KeyCode::Char('p'), KeyModifiers::CONTROL) => {
                log::debug!("Handling Ctrl+P insert image from clipboard operation");
                match get_image_from_clipboard() {
                    Ok(image_data) => {
                        log::debug!("Image retrieved from clipboard, opening naming modal");
                        Some(KeyResult::ImageNamingModal(image_data))
                    }
                    Err(e) => {
                        log::error!("Failed to get image from clipboard: {e}");
                        if has_existing_image {
                            log::debug!("No clipboard image but entry has existing image, showing removal modal");
                            Some(KeyResult::ImageRemovalModal)
                        } else {
                            log::debug!("No clipboard image and no existing image, operation failed");
                            Some(KeyResult::Handled(false))
                        }
                    }
                }
            }
            // Handle Command+Left (word left) - macOS
            // (KeyCode::Left, KeyModifiers::SUPER) => {
            //     log::debug!("Handling Command+Left word move to head of line (home)");
            //     textarea.move_cursor(CursorMove::Head);
            //     Some(KeyResult::Handled(true))
            // }
            // Handle Command+Right (word right) - macOS
            // (KeyCode::Right, KeyModifiers::SUPER) => {
            //     log::debug!("Handling Command+Right word move to end of line");
            //     Self::move_cursor_word_right(textarea);
            //     Some(KeyResult::Handled(true))
            // }
            // Handle Alt+Left (word left) - Alternative for terminals that intercept Ctrl+Left
            (KeyCode::Left, KeyModifiers::ALT) => {
                log::debug!("Handling Alt+Left word jump left");
                Self::move_cursor_word_left(textarea);
                Some(KeyResult::Handled(true))
            }
            // Handle Alt+Right (word right) - Alternative for terminals that intercept Ctrl+Right  
            (KeyCode::Right, KeyModifiers::ALT) => {
                log::debug!("Handling Alt+Right word jump right");
                Self::move_cursor_word_right(textarea);
                Some(KeyResult::Handled(true))
            }
            // Not a clipboard operation
            _ => {
                None
            }
        }
    }

    fn get_selected_text(textarea: &TextArea<'static>) -> Option<String> {
        let lines = textarea.lines();

        // Debug: print all lines
        for (i, line) in lines.iter().enumerate() {
            log::debug!("Line {i}: '{line}'");
        }

        // Check if there's a selection
        let selection_range = textarea.selection_range();
        log::debug!("Selection range: {selection_range:?}");
        log::debug!("Total lines: {}", lines.len());
        log::debug!("Current cursor position: {:?}", textarea.cursor());

        if let Some(((start_row, start_col), (end_row, end_col))) = selection_range {
            log::debug!(
                "Selection found: ({start_row}, {start_col}) to ({end_row}, {end_col})"
            );

            let text = lines.join("\n");

            // Convert row/col positions to character positions
            let mut start_pos = 0;
            let mut end_pos = 0;

            for (row, line) in lines.iter().enumerate() {
                if row < start_row {
                    start_pos += line.len() + 1; // +1 for newline
                } else if row == start_row {
                    start_pos += start_col;
                    break;
                }
            }

            for (row, line) in lines.iter().enumerate() {
                if row < end_row {
                    end_pos += line.len() + 1; // +1 for newline
                } else if row == end_row {
                    end_pos += end_col;
                    break;
                }
            }

            log::debug!(
                "Character positions: {} to {} (total text length: {})",
                start_pos,
                end_pos,
                text.len()
            );

            if start_pos <= end_pos && end_pos <= text.len() {
                let selected = text[start_pos..end_pos].to_string();
                log::debug!("Selected text: '{selected}'");
                Some(selected)
            } else {
                log::debug!("Invalid selection range");
                None
            }
        } else {
            log::debug!("No selection found");
            None
        }
    }

    fn copy_to_system_clipboard(text: &str) -> Result<(), Box<dyn std::error::Error>> {
        get_clipboard_manager().set_text(text).map_err(|e| -> Box<dyn std::error::Error> { e })?;
        Ok(())
    }

    fn get_from_system_clipboard() -> Result<String, Box<dyn std::error::Error>> {
        get_clipboard_manager().get_text().map_err(|e| -> Box<dyn std::error::Error> { e })
    }

    pub fn insert_text(textarea: &mut TextArea<'static>, text: &str) {
        for ch in text.chars() {
            if ch == '\n' {
                textarea.insert_newline();
            } else {
                textarea.insert_char(ch);
            }
        }
    }

    fn move_cursor_word_left(textarea: &mut TextArea<'static>) {
        let (row, col) = textarea.cursor();
        let lines = textarea.lines();
        
        if lines.is_empty() {
            return;
        }
        
        if row >= lines.len() {
            return;
        }
        
        let current_line = &lines[row];
        
        // If we're at the beginning of a line, move to the end of the previous line
        if col == 0 {
            if row > 0 {
                let prev_line = &lines[row - 1];
                textarea.move_cursor(CursorMove::Jump((row - 1) as u16, prev_line.len() as u16));
            }
            return;
        }
        
        // Find the start of the current word or the previous word
        let chars: Vec<char> = current_line.chars().collect();
        let mut pos = col.min(chars.len());
        
        // If we're in the middle of whitespace, skip backward to the end of the previous word
        while pos > 0 && chars[pos - 1].is_whitespace() {
            pos -= 1;
        }
        
        // If we're at the end of a word, move to the beginning of this word
        while pos > 0 && !chars[pos - 1].is_whitespace() {
            pos -= 1;
        }
        
        textarea.move_cursor(CursorMove::Jump(row as u16, pos as u16));
    }

    fn move_cursor_word_right(textarea: &mut TextArea<'static>) {
        let (row, col) = textarea.cursor();
        let lines = textarea.lines();
        
        if lines.is_empty() {
            return;
        }
        
        if row >= lines.len() {
            return;
        }
        
        let current_line = &lines[row];
        let chars: Vec<char> = current_line.chars().collect();
        
        // If we're at the end of a line, move to the beginning of the next line
        if col >= chars.len() {
            if row + 1 < lines.len() {
                textarea.move_cursor(CursorMove::Jump((row + 1) as u16, 0));
            }
            return;
        }
        
        let mut pos = col;
        
        // If we're in the middle of a word, move to the end of the current word
        while pos < chars.len() && !chars[pos].is_whitespace() {
            pos += 1;
        }
        
        // Skip any whitespace to get to the start of the next word
        while pos < chars.len() && chars[pos].is_whitespace() {
            pos += 1;
        }
        
        textarea.move_cursor(CursorMove::Jump(row as u16, pos as u16));
    }
}
