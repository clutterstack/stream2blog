use crate::app::App;
use crate::block_styles::{bordered, titled};
use crate::state::AppState;
use crate::widgets::confirmation_dialog::{ConfirmationDialog, ConfirmationType};
use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    widgets::{List, ListItem, Paragraph, Wrap},
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
            AppState::ConfirmImageReplacement(prev_state, _, _) => {
                log::debug!("Drawing ConfirmImageReplacement modal, prev_state: {prev_state:?}");
                self.draw_image_replacement_modal(f, prev_state);
            }
            AppState::ConfirmImageRemoval(prev_state, _) => {
                log::debug!("Drawing ConfirmImageRemoval modal, prev_state: {prev_state:?}");
                self.draw_image_removal_modal(f, prev_state);
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
            .block(titled("Threads"))
            .highlight_style(Style::default().fg(Color::Yellow));

        // Ensure main_thread_list_state is synced with selected_thread_index
        self.main_thread_list_state.select(Some(self.selected_thread_index));

        // Store the thread list area for mouse click mapping
        self.thread_list_area = Some(chunks[0]);
        
        // Calculate and store individual thread positions for mouse selection
        self.calculate_thread_positions(chunks[0]);

        f.render_stateful_widget(threads_list, chunks[0], &mut self.main_thread_list_state);

        let help = Paragraph::new("[↑/↓: Navigate] [Enter: Open] [Mouse: Click to select/open] [n: New] [r: Rename] [d: New with Datestamp] [Del/Backspace: Delete] [q: Quit]")
            .block(bordered())
            .wrap(Wrap { trim: true });
        f.render_widget(help, chunks[1]);
    }

    fn draw_confirm_delete_thread(&mut self, f: &mut Frame) {
        self.draw_thread_list(f);

        let dialog = ConfirmationDialog::new(ConfirmationType::DeleteThread);
        dialog.render(f, f.area());
    }

    fn draw_confirm_delete_entry(&mut self, f: &mut Frame) {
        self.draw_thread_view(f);

        let dialog = ConfirmationDialog::new(ConfirmationType::DeleteEntry);
        dialog.render(f, f.area());
    }

    fn draw_confirm_discard_entry_changes(&mut self, f: &mut Frame) {
        // Draw the edit entry view underneath
        self.draw_edit_entry(f);

        let dialog = ConfirmationDialog::new(ConfirmationType::DiscardChanges);
        dialog.render(f, f.area());
    }

    fn draw_confirm_discard_new_entry(&mut self, f: &mut Frame) {
        // Draw the create entry view underneath
        self.draw_create_entry(f);

        let dialog = ConfirmationDialog::new(ConfirmationType::DiscardNewEntry);
        dialog.render(f, f.area());
    }
}
