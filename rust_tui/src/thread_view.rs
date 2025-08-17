use crate::app::App;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Style, Stylize},
    text::{Line, Text},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap, Padding},
    Frame,
};

impl App {
    pub fn draw_thread_view(&mut self, f: &mut Frame) {
        let main_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(100), Constraint::Length(4)])
            .split(f.area());

        if let Some(thread) = self.current_thread.clone() {
            // Always split the main area horizontally for list and preview (when enabled)
            let (entry_list_chunk, preview_chunk) = if self.thread_view_image_preview_visible {
                let horizontal_chunks = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
                    .split(main_chunks[0]);
                (horizontal_chunks[0], Some(horizontal_chunks[1]))
            } else {
                // When preview is disabled, entry list takes full width
                (main_chunks[0], None)
            };


            // Split the entry list chunk vertically for entry list and word count
            let entry_vertical_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Min(0), Constraint::Length(7)])
                .split(entry_list_chunk);
            
            let actual_entry_list_chunk = entry_vertical_chunks[0];
            let word_count_chunk = entry_vertical_chunks[1];

            // Draw the entry list
            let items: Vec<ListItem> = thread
                .entries
                .iter()
                .enumerate()
                .map(|(index, entry)| {
                    let preview = if let Some(newline_pos) = entry.content.find('\n') {
                        if newline_pos < 50 {
                            format!("{}...", &entry.content[..newline_pos])
                        } else if entry.content.len() > 50 {
                            format!("{}...", &entry.content[..50])
                        } else {
                            entry.content.clone()
                        }
                    } else if entry.content.len() > 50 {
                        format!("{}...", &entry.content[..50])
                    } else {
                        entry.content.clone()
                    };
                    // Create the ListItem with unwrapped text - List widget will handle display wrapping
                    ListItem::new(format!("{}: {}", index + 1, preview))
                })
                .collect();

            let entries_list = List::new(items)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title(thread.title.as_str())
                        .title_style(Style::new().white().bold())
                        .padding(Padding::uniform(1)),
                )
                .highlight_style(Style::default().fg(Color::Yellow));

            // Ensure thread_list_state is synced with selected_entry_index
            self.thread_list_state.select(Some(self.selected_entry_index));

            // Store the entry list area for mouse click mapping (use actual chunk, not the split chunk)
            self.entry_list_area = Some(actual_entry_list_chunk);
            
            // Calculate and store individual entry positions for mouse selection
            self.calculate_entry_positions(actual_entry_list_chunk, &thread.entries);
            
            f.render_stateful_widget(entries_list, actual_entry_list_chunk, &mut self.thread_list_state);

            // Draw word count statistics
            let (total_words, entry_count, avg_words) = self.calculate_thread_word_count(&thread);
            let stats_text = format!(
                "Total words: {total_words}\nEntries: {entry_count}\nAvg/entry: {avg_words:.1} words"
            );
            
            let stats_paragraph = Paragraph::new(stats_text)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title("Statistics")
                        .padding(Padding::uniform(1))
                )
                .alignment(Alignment::Left)
                .wrap(Wrap { trim: true });
            
            f.render_widget(stats_paragraph, word_count_chunk);

            // Draw the preview panel if enabled
            if let Some(preview_area) = preview_chunk {
                self.draw_entry_preview(f, preview_area, &thread);
            }
        }

        let help = Paragraph::new("[↑/↓: Navigate] [Ctrl+Shift+↑/↓: Reorder] [Ctrl+t: Toggle Preview] [n: New Entry] [r: Rename Thread] [e: Edit] [x: Export] [Del/Backspace: Delete] [Esc/q: Back]")
            .block(Block::default().borders(Borders::ALL))
            .wrap(Wrap { trim: true });
        f.render_widget(help, main_chunks[1]);
    }

    fn draw_entry_preview(
        &mut self,
        f: &mut Frame,
        area: ratatui::layout::Rect,
        thread: &crate::models::Thread,
    ) {
        if thread.entries.is_empty() {
            let empty_message = Paragraph::new("No entries in this thread")
                .block(Block::default().borders(Borders::ALL).title("Preview").padding(Padding::uniform(1)))
                .alignment(Alignment::Center)
                .style(Style::default().fg(Color::Gray));
            f.render_widget(empty_message, area);
            return;
        }

        self.draw_single_entry_preview(f, area, thread);
    }

    fn draw_single_entry_preview(
        &mut self,
        f: &mut Frame,
        area: ratatui::layout::Rect,
        thread: &crate::models::Thread,
    ) {
        if self.selected_entry_index >= thread.entries.len() {
            let error_message = Paragraph::new("Invalid entry selection")
                .block(Block::default().borders(Borders::ALL).title("Preview").padding(Padding::uniform(1)))
                .alignment(Alignment::Center)
                .style(Style::default().fg(Color::Red));
            f.render_widget(error_message, area);
            return;
        }

        let selected_entry = &thread.entries[self.selected_entry_index];

        // Get context from adjacent entries
        let previous_context = if self.selected_entry_index > 0 {
            let prev_entry = &thread.entries[self.selected_entry_index - 1];
            self.get_last_lines(&prev_entry.content, 2)
        } else {
            String::new()
        };

        let next_context = if self.selected_entry_index + 1 < thread.entries.len() {
            let next_entry = &thread.entries[self.selected_entry_index + 1];
            self.get_first_lines(&next_entry.content, 2)
        } else {
            String::new()
        };

        // Check if we have a cached thumbnail for this entry
        let has_cached_thumbnail = self.entry_thumbnails.contains_key(&selected_entry.id);

        if has_cached_thumbnail {
            // Split the preview area for text and image
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Fill(0), Constraint::Length(12)])
                .split(area);

            // Draw text content with context
            let preview_content = self.build_preview_with_context(
                selected_entry,
                &previous_context,
                &next_context,
                self.selected_entry_index + 1,
            );

            let preview_paragraph = Paragraph::new(preview_content)
                .block(Block::default().borders(Borders::ALL).title("Preview").padding(Padding::uniform(1)))
                .wrap(Wrap { trim: true })
                .scroll((self.preview_scroll_offset, 0));

            f.render_widget(preview_paragraph, chunks[0]);

            // Draw cached thumbnail
            if let Some(thumbnail) = self.entry_thumbnails.get_mut(&selected_entry.id) {
                thumbnail.set_visible(true);
                thumbnail.render(f, chunks[1]);
            }
        } else {
            // Draw text content only with context
            let preview_content = self.build_preview_with_context(
                selected_entry,
                &previous_context,
                &next_context,
                self.selected_entry_index + 1,
            );

            let preview_paragraph = Paragraph::new(preview_content)
                .block(Block::default().borders(Borders::ALL).title("Preview").padding(Padding::uniform(1)))
                .wrap(Wrap { trim: true })
                .scroll((self.preview_scroll_offset, 0));

            f.render_widget(preview_paragraph, area);
        }
    }

    fn get_last_lines(&self, content: &str, num_lines: usize) -> String {
        let lines: Vec<&str> = content.lines().collect();
        if lines.is_empty() {
            return String::new();
        }
        
        let start_index = if lines.len() > num_lines {
            lines.len() - num_lines
        } else {
            0
        };
        
        let joined = lines[start_index..].join("\n");
        
        // Truncate to max 100 characters to prevent very long preview context
        if joined.len() > 100 {
            format!("{}...", &joined[..100])
        } else {
            joined
        }
    }

    fn get_first_lines(&self, content: &str, num_lines: usize) -> String {
        let joined = content
            .lines()
            .take(num_lines)
            .collect::<Vec<_>>()
            .join("\n");
            
        // Truncate to max 100 characters to prevent very long preview context
        if joined.len() > 100 {
            format!("{}...", &joined[..100])
        } else {
            joined
        }
    }

    fn build_preview_with_context(
        &self,
        selected_entry: &crate::models::Entry,
        previous_context: &str,
        next_context: &str,
        _display_number: usize,
    ) -> Text<'static> {
        let mut lines = Vec::new();

        // Add previous entry context if available
        if !previous_context.is_empty() {
            lines.push(Line::raw(""));
            lines.push(Line::styled(
                format!("... {previous_context}"),
                Style::default().fg(Color::Blue),
            ));
        } else {
            lines.push(Line::styled(
              "━━━ Start of thread ━━━",
              Style::default().fg(Color::Blue),
            ));
        }
        lines.push(Line::raw(""));

        // Add current entry separator with blue styling
        // lines.push(Line::styled(
        //     format!("━ Entry {}", display_number),
        //     Style::default().fg(Color::Blue),
        // ));
        // lines.push(Line::raw(""));
        
        // Add current entry content, splitting by lines
        for line in selected_entry.content.lines() {
            lines.push(Line::raw(line.to_string()));
        }

        lines.push(Line::raw(""));

        // Add current entry separator with blue styling
        // lines.push(Line::styled(
        //     format!("━"),
        //     Style::default().fg(Color::Blue),
        // ));
        // Add next entry context if available
        if !next_context.is_empty() {
            lines.push(Line::styled(
                format!("{next_context} ..."),
                Style::default().fg(Color::Blue),
            ));
        } else {
            lines.push(Line::styled(
              "━━━ End of thread ━━━".to_string(),
            Style::default().fg(Color::Blue),
            ));
        }

        Text::from(lines)
    }

    fn calculate_entry_positions(&mut self, list_area: ratatui::layout::Rect, entries: &[crate::models::Entry]) {
        self.entry_positions.clear();
        
        // Get the current scroll offset from ListState
        let scroll_offset = self.thread_list_state.offset();
        
        // Account for List widget borders and padding
        // List widget has 1 unit border on all sides, plus 1 unit vertical padding as configured
        let content_x = list_area.x + 1;
        let content_y = list_area.y + 1 + 1; // +1 for border, +1 for vertical padding
        let content_width = list_area.width.saturating_sub(2); // -2 for left and right borders
        
        // Calculate visible area height for bounds checking
        let visible_height = list_area.height.saturating_sub(3); // -2 for borders, -1 for padding
        
        // Each list item takes exactly 1 row
        for (index, _entry) in entries.iter().enumerate() {
            // Calculate the screen position accounting for scroll offset
            let screen_row = if index >= scroll_offset {
                content_y + (index - scroll_offset) as u16
            } else {
                // Entry is scrolled out of view above - set to 0 to mark as invisible
                0
            };
            
            let entry_rect = ratatui::layout::Rect {
                x: content_x,
                y: screen_row,
                width: content_width,
                height: 1,
            };
            
            // Only add positions that are visible within the list area
            if index >= scroll_offset && screen_row < content_y + visible_height {
                self.entry_positions.push(entry_rect);
            } else {
                // Entry is not visible (either scrolled out or beyond visible area)
                self.entry_positions.push(ratatui::layout::Rect {
                    x: 0,
                    y: 0,
                    width: 0,
                    height: 0,
                });
            }
        }
    }

    pub fn get_clicked_entry_index(&self, column: u16, row: u16) -> Option<usize> {
        for (index, rect) in self.entry_positions.iter().enumerate() {
            if rect.width > 0 && rect.height > 0 {  // Only check visible entries
                if column >= rect.x 
                    && column < rect.x + rect.width 
                    && row >= rect.y 
                    && row < rect.y + rect.height 
                {
                    return Some(index);
                }
            }
        }
        None
    }
}
