use crate::app::App;
use crate::state::AppState;
use crate::ui_utils::centered_rect;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Style},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap},
    Frame,
};

impl App {
    pub fn draw(&mut self, f: &mut Frame) {
        let state_clone = self.state.clone();
        match &state_clone {
            AppState::ThreadList => {
                self.draw_thread_list(f);
            }
            AppState::ThreadView(_) => {
                self.draw_thread_view(f);
            }
            AppState::CreateThread => self.draw_create_thread(f),
            AppState::EditThread(_) => self.draw_edit_thread(f),
            AppState::CreateEntry(_) => self.draw_create_entry(f),
            AppState::EditEntry(_, _) => self.draw_edit_entry(f),
            AppState::ConfirmDeleteThread(_) => self.draw_confirm_delete_thread(f),
            AppState::ConfirmDeleteEntry(_, _) => self.draw_confirm_delete_entry(f),
            AppState::ConfirmDiscardEntryChanges(_, _) => self.draw_confirm_discard_entry_changes(f),
            AppState::ConfirmDiscardNewEntry(_) => self.draw_confirm_discard_new_entry(f),
            AppState::ImageNaming(prev_state, _) => {
                log::debug!("Drawing ImageNaming modal, prev_state: {prev_state:?}");
                self.draw_image_naming_modal(f, prev_state);
            }
            AppState::CharacterLimitError(prev_state) => {
                log::debug!("Drawing CharacterLimitError modal, prev_state: {prev_state:?}");
                self.draw_character_limit_error_modal(f, prev_state);
            }
        }
    }

    pub fn draw_thread_list(&mut self, f: &mut Frame) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(0), Constraint::Length(4)])
            .split(f.area());

        let items: Vec<ListItem> = self
            .threads
            .iter()
            .map(|thread| {
                let entry_count = thread.entries.len();
                ListItem::new(format!("{} ({} entries)", thread.title, entry_count))
            })
            .collect();

        let threads_list = List::new(items)
            .block(Block::default().borders(Borders::ALL).title("Threads"))
            .highlight_style(Style::default().fg(Color::Yellow));

        // Ensure main_thread_list_state is synced with selected_thread_index
        self.main_thread_list_state.select(Some(self.selected_thread_index));

        // Store the thread list area for mouse click mapping
        self.thread_list_area = Some(chunks[0]);
        
        // Calculate and store individual thread positions for mouse selection
        self.calculate_thread_positions(chunks[0]);

        f.render_stateful_widget(threads_list, chunks[0], &mut self.main_thread_list_state);

        let help = Paragraph::new("[↑/↓: Navigate] [Enter: Open] [Mouse: Click to select/open] [n: New] [r: Rename] [d: New with Datestamp] [Del/Backspace: Delete] [q: Quit]")
            .block(Block::default().borders(Borders::ALL))
            .wrap(Wrap { trim: true });
        f.render_widget(help, chunks[1]);
    }

    fn draw_confirm_delete_thread(&mut self, f: &mut Frame) {
        self.draw_thread_list(f);

        // Draw confirmation popup
        let popup_area = centered_rect(50, 30, f.area());
        f.render_widget(Clear, popup_area);

        let confirmation = Paragraph::new("Are you sure you want to delete this thread?\n\nThis will delete the thread and all its entries.\n\n[Y] Yes   [N] No   [Esc] Cancel")
            .block(Block::default().borders(Borders::ALL).title("Confirm Delete"))
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::Red));
        f.render_widget(confirmation, popup_area);
    }

    fn draw_confirm_delete_entry(&mut self, f: &mut Frame) {
        self.draw_thread_view(f);

        // Draw confirmation popup
        let popup_area = centered_rect(50, 25, f.area());
        f.render_widget(Clear, popup_area);

        let confirmation = Paragraph::new(
            "Are you sure you want to delete this entry?\n\n[Y] Yes   [N] No   [Esc] Cancel",
        )
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Confirm Delete"),
        )
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::Red));
        f.render_widget(confirmation, popup_area);
    }

    fn draw_confirm_discard_entry_changes(&mut self, f: &mut Frame) {
        // Draw the edit entry view underneath
        self.draw_edit_entry(f);

        // Draw confirmation popup
        let popup_area = centered_rect(60, 25, f.area());
        f.render_widget(Clear, popup_area);

        let confirmation = Paragraph::new(
            "You have unsaved changes to the entry content.\n\nDiscard changes and exit?\n\n[Y] Yes   [N] No   [Esc] Cancel",
        )
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Discard Changes?"),
        )
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::Yellow));
        f.render_widget(confirmation, popup_area);
    }

    fn draw_confirm_discard_new_entry(&mut self, f: &mut Frame) {
        // Draw the create entry view underneath
        self.draw_create_entry(f);

        // Draw confirmation popup
        let popup_area = centered_rect(60, 25, f.area());
        f.render_widget(Clear, popup_area);

        let confirmation = Paragraph::new(
            "You have unsaved content for this new entry.\n\nDiscard new entry and exit?\n\n[Y] Yes   [N] No   [Esc] Cancel",
        )
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Discard New Entry?"),
        )
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::Yellow));
        f.render_widget(confirmation, popup_area);
    }
}
