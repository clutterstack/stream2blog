use crate::text_editor::TextEditor;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    Frame,
};

pub struct TextEditorWrapper<'a> {
    text_editor: &'a mut TextEditor,
}

impl<'a> TextEditorWrapper<'a> {
    pub fn new(text_editor: &'a mut TextEditor) -> Self {
        Self {
            text_editor,
        }
    }


    pub fn render(&mut self, f: &mut Frame, area: Rect) {
        // Check if we should show image preview based on text editor state
        let has_image = self.text_editor.has_image();
        let preview_visible = has_image && self.text_editor.is_image_preview_visible();
        let is_full_screen = self.text_editor.image_preview_mut().is_full_screen();

        if preview_visible {
            if is_full_screen {
                // Full screen mode - image takes the entire area
                self.text_editor.image_preview_mut().render(f, area);
            } else {
                // Split the area horizontally for text editor and image preview
                let chunks = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
                    .split(area);

                // Render text editor
                self.render_text_editor(f, chunks[0]);

                // Render image preview
                self.text_editor.image_preview_mut().render(f, chunks[1]);
            }
        } else {
            // Render text editor only
            self.render_text_editor(f, area);
        }
    }

    fn render_text_editor(&mut self, f: &mut Frame, area: Rect) {
        // Store the textarea area for mouse handling
        self.text_editor.set_area(area);
        f.render_widget(self.text_editor.widget(), area);

        // Set cursor position using visual coordinates for wrapped text
        let text_area = if let Some(block) = self.text_editor.widget().block() {
            block.inner(area)
        } else {
            area
        };
        
        if let Some((cursor_col, cursor_row)) = self.text_editor.visual_cursor_position(text_area.width, text_area.height) {
            f.set_cursor_position((text_area.x + cursor_col, text_area.y + cursor_row));
        }
    }
}