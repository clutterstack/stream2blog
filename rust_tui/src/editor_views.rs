use crate::app::App;
use crate::block_styles::{bordered, titled, content_block};
use crate::state::AppState;
use crate::ui_utils::centered_rect_fixed_height;
use crate::widgets::text_editor_wrapper::TextEditorWrapper;
use crate::widgets::status_bar::{StatusBar, CharacterThresholds};
use crate::widgets::confirmation_dialog::{ConfirmationDialog, ConfirmationType};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Style, Stylize},
    widgets::{Clear, Paragraph},
    Frame,
};

impl App {
    fn render_text_editor_with_image_preview(&mut self, f: &mut Frame, main_area: ratatui::layout::Rect) {
        let mut wrapper = TextEditorWrapper::new(&mut self.text_editor);
        wrapper.render(f, main_area);
    }

    fn render_status_bar(&mut self, f: &mut Frame, area: ratatui::layout::Rect, button_text: &str) {
        let text_len = self.text_editor.lines().join("\n").len();
        let status_bar = StatusBar::new(text_len, 400, button_text)
            .with_thresholds(CharacterThresholds {
                warning: 300,
                caution: 350,
                danger: 400,
            });
        
        self.submit_button_area = status_bar.render(f, area);
    }

    fn render_help_text(&self, f: &mut Frame, area: ratatui::layout::Rect, action_text: &str) {
        let help = Paragraph::new(format!("[Ctrl+S: {action_text}] [Esc: Cancel] [Ctrl+A: Select All] [Ctrl+C/X/V: Copy/Cut/Paste] [Ctrl+P: Paste Image] [Ctrl+T: Toggle Image Preview] [Ctrl+F: Toggle Full Screen] [Mouse: Click/Drag to select]"))
            .block(bordered());
        f.render_widget(help, area);
    }

    pub fn draw_create_thread(&mut self, f: &mut Frame) {
        // Draw the thread list in the background
        self.draw_thread_list(f);

        // Draw modal popup
        let popup_area = centered_rect_fixed_height(60, 7, f.area());
        f.render_widget(Clear, popup_area);
        
        // Add warm grey background
        let background = bordered();
            // .style(Style::default().bg(Color::Rgb(120, 115, 110)));
        f.render_widget(background, popup_area);

        let popup_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1), // Background padding line
                Constraint::Length(3), // Input field
                Constraint::Length(2), // Help text
            ])
            .split(popup_area);

        // Create horizontal layout for input area with side columns
        let input_horizontal_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(2), // Left column
                Constraint::Min(0),    // Input field
                Constraint::Length(2), // Right column
            ])
            .split(popup_chunks[1]);

        // Text input area with label on the border
        let input_block = titled("Enter thread title");
        let _inner = input_block.inner(input_horizontal_chunks[1]);
        self.text_editor.set_block(input_block);
        f.render_widget(self.text_editor.widget(), input_horizontal_chunks[1]);

        // Set cursor position using visual coordinates for wrapped text
        let text_area = if let Some(block) = self.text_editor.widget().block() {
            block.inner(input_horizontal_chunks[1])
        } else {
            input_horizontal_chunks[1]
        };
        
        if let Some((cursor_col, cursor_row)) = self.text_editor.visual_cursor_position(text_area.width, text_area.height) {
            f.set_cursor_position((text_area.x + cursor_col, text_area.y + cursor_row));
        }

        // Help text
        let help = Paragraph::new("[Enter/Ctrl+S] Create   [Esc] Cancel")
            .alignment(Alignment::Center);
        f.render_widget(help, popup_chunks[2]);
    }

    pub fn draw_edit_thread(&mut self, f: &mut Frame) {
        // Draw the thread list in the background
        self.draw_thread_list(f);

        // Draw modal popup
        let popup_area = centered_rect_fixed_height(60, 7, f.area());
        f.render_widget(Clear, popup_area);
        
        // Add warm grey background
        let background = bordered();
            // .style(Style::default().bg(Color::Rgb(120, 115, 110)));
        f.render_widget(background, popup_area);

        let popup_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1), // Background padding line
                Constraint::Length(3), // Input field
                Constraint::Length(2), // Help text
            ])
            .split(popup_area);

        // Create horizontal layout for input area with side columns
        let input_horizontal_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(2), // Left column
                Constraint::Min(0),    // Input field
                Constraint::Length(2), // Right column
            ])
            .split(popup_chunks[1]);

        // Text input area with label on the border
        let input_block = titled("Edit thread title");
        let _inner = input_block.inner(input_horizontal_chunks[1]);
        self.text_editor.set_block(input_block);
        f.render_widget(self.text_editor.widget(), input_horizontal_chunks[1]);

        // Set cursor position using visual coordinates for wrapped text
        let text_area = if let Some(block) = self.text_editor.widget().block() {
            block.inner(input_horizontal_chunks[1])
        } else {
            input_horizontal_chunks[1]
        };
        
        if let Some((cursor_col, cursor_row)) = self.text_editor.visual_cursor_position(text_area.width, text_area.height) {
            f.set_cursor_position((text_area.x + cursor_col, text_area.y + cursor_row));
        }

        // Help text
        let help = Paragraph::new("[Enter/Ctrl+S] Update   [Esc] Cancel")
            .alignment(Alignment::Center);
        f.render_widget(help, popup_chunks[2]);
    }

    pub fn draw_create_entry(&mut self, f: &mut Frame) {
        // Set the editor title with thread title if available
        let title = if let Some(thread) = &self.current_thread {
            let truncated_title = if thread.title.len() > 30 {
                format!("{}...", &thread.title[..30])
            } else {
                thread.title.clone()
            };
            format!("Creating entry in {truncated_title}")
        } else {
            "Creating new entry".to_string()
        };
        
        let block = content_block(title)
            .title_style(Style::new().white().bold());

        self.text_editor.set_block(block);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(3),
                Constraint::Length(3),
                Constraint::Length(3),
            ])
            .split(f.area());

        self.render_text_editor_with_image_preview(f, chunks[0]);
        self.render_status_bar(f, chunks[1], "Submit");
        self.render_help_text(f, chunks[2], "Create");
    }

    pub fn draw_edit_entry(&mut self, f: &mut Frame) {
        // Set the editor title with thread title if available
        let title = if let Some(thread) = &self.current_thread {
            let truncated_title = if thread.title.len() > 30 {
                format!("{}...", &thread.title[..30])
            } else {
                thread.title.clone()
            };
            format!("Editing entry in {truncated_title}")
        } else {
            "Editing entry".to_string()
        };
        
        let block = content_block(title)
            .title_style(Style::new().white().bold());

        self.text_editor.set_block(block);

        // Check if current entry has an image path to determine layout
        let has_image_path = if let AppState::EditEntry(_, entry_id) = &self.state {
            if let Some(thread) = &self.current_thread {
                if let Some(entry) = thread.entries.iter().find(|e| e.id == *entry_id) {
                    entry.image_path.is_some()
                } else { false }
            } else { false }
        } else { false };

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(if has_image_path {
                vec![
                    Constraint::Min(3),    // Text editor
                    Constraint::Length(3), // Character count and submit button
                    Constraint::Length(3), // Image path
                    Constraint::Length(3), // Help
                ]
            } else {
                vec![
                    Constraint::Min(3),    // Text editor  
                    Constraint::Length(3), // Character count and submit button
                    Constraint::Length(3), // Help
                ]
            })
            .split(f.area());

        self.render_text_editor_with_image_preview(f, chunks[0]);

        self.render_status_bar(f, chunks[1], "Update");

        // Render image path if available
        let help_chunk_index = if has_image_path {
            // Show image path in a full-width row
            if let AppState::EditEntry(_, entry_id) = &self.state {
                if let Some(thread) = &self.current_thread {
                    if let Some(entry) = thread.entries.iter().find(|e| e.id == *entry_id) {
                        if let Some(image_path) = &entry.image_path {
                            let image_display = Paragraph::new(format!("Image: {image_path}"))
                                .block(titled("Attached Image"))
                                .style(Style::default().fg(Color::Cyan));
                            f.render_widget(image_display, chunks[2]);
                        }
                    }
                }
            }
            3 // Help goes in chunk 3 when image is shown
        } else {
            2 // Help goes in chunk 2 when no image
        };

        self.render_help_text(f, chunks[help_chunk_index], "Update");
    }

    pub fn draw_image_naming_modal(&mut self, f: &mut Frame, prev_state: &AppState) {
        log::debug!(
            "draw_image_naming_modal called with prev_state: {prev_state:?}"
        );
        // Draw the previous state in the background
        match prev_state {
            AppState::CreateThread => self.draw_create_thread(f),
            AppState::EditThread(_) => self.draw_edit_thread(f),
            AppState::CreateEntry(_) => self.draw_create_entry(f),
            AppState::EditEntry(_, _) => self.draw_edit_entry(f),
            _ => {} // Other states don't need background
        }

        // Draw modal popup
        let popup_area = centered_rect_fixed_height(60, 7, f.area());
        log::debug!("Modal popup area: {popup_area:?}");
        f.render_widget(Clear, popup_area);
        
        // Add warm grey background
        let background = bordered();
            // .style(Style::default().bg(Color::Rgb(120, 115, 110)));
        f.render_widget(background, popup_area);

        let popup_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1), // Background padding line
                Constraint::Length(3), // Input field
                Constraint::Length(2), // Help text
            ])
            .split(popup_area);

        // Create horizontal layout for input area with side columns
        let input_horizontal_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(2), // Left column
                Constraint::Min(0),    // Input field
                Constraint::Length(2), // Right column
            ])
            .split(popup_chunks[1]);

        // Text input area with label on the border
        let input_block = titled("Name your image file");
        let _inner = input_block.inner(input_horizontal_chunks[1]);
        self.modal_text_editor.set_block(input_block);
        f.render_widget(self.modal_text_editor.widget(), input_horizontal_chunks[1]);

        // Set cursor position using visual coordinates for wrapped text
        let text_area = if let Some(block) = self.modal_text_editor.widget().block() {
            block.inner(input_horizontal_chunks[1])
        } else {
            input_horizontal_chunks[1]
        };
        
        if let Some((cursor_col, cursor_row)) = self.modal_text_editor.visual_cursor_position(text_area.width, text_area.height) {
            f.set_cursor_position((text_area.x + cursor_col, text_area.y + cursor_row));
        }

        // Help text
        let help = Paragraph::new("[Enter] Save   [Esc] Cancel")
            .alignment(Alignment::Center);
        f.render_widget(help, popup_chunks[2]);
        log::debug!("Image naming modal fully rendered");
    }

    pub fn draw_image_replacement_modal(&mut self, f: &mut Frame, prev_state: &AppState) {
        log::debug!("draw_image_replacement_modal called with prev_state: {prev_state:?}");
        
        // Draw the previous state in the background
        match prev_state {
            AppState::CreateThread => self.draw_create_thread(f),
            AppState::EditThread(_) => self.draw_edit_thread(f),
            AppState::CreateEntry(_) => self.draw_create_entry(f),
            AppState::EditEntry(_, _) => self.draw_edit_entry(f),
            _ => {} // Other states don't need background
        }

        let dialog = ConfirmationDialog::new(ConfirmationType::ReplaceImage);
        dialog.render(f, f.area());
        
        log::debug!("Image replacement modal fully rendered");
    }

    pub fn draw_image_removal_modal(&mut self, f: &mut Frame, prev_state: &AppState) {
        log::debug!("draw_image_removal_modal called with prev_state: {prev_state:?}");
        
        // Draw the previous state in the background
        match prev_state {
            AppState::CreateThread => self.draw_create_thread(f),
            AppState::EditThread(_) => self.draw_edit_thread(f),
            AppState::CreateEntry(_) => self.draw_create_entry(f),
            AppState::EditEntry(_, _) => self.draw_edit_entry(f),
            _ => {} // Other states don't need background
        }

        let dialog = ConfirmationDialog::new(ConfirmationType::RemoveImage);
        dialog.render(f, f.area());

        log::debug!("Image removal modal fully rendered");
    }

    pub fn draw_character_limit_error_modal(&mut self, f: &mut Frame, prev_state: &AppState) {
        log::debug!(
            "draw_character_limit_error_modal called with prev_state: {prev_state:?}"
        );
        // Draw the previous state in the background
        match prev_state {
            AppState::CreateThread => self.draw_create_thread(f),
            AppState::EditThread(_) => self.draw_edit_thread(f),
            AppState::CreateEntry(_) => self.draw_create_entry(f),
            AppState::EditEntry(_, _) => self.draw_edit_entry(f),
            _ => {} // Other states don't need background
        }

        let dialog = ConfirmationDialog::new(ConfirmationType::CharacterLimit);
        dialog.render(f, f.area());
    }
}
