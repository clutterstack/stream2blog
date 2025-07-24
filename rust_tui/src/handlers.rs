use crate::app::App;
use crate::image_clip::save_image_with_context;
use crate::key_handler::KeyResult;
use crate::state::AppState;
use chrono::{DateTime, Utc};
use ratatui::crossterm::event::{KeyCode, KeyEvent, MouseEvent};
use ratatui::widgets::{Block, Borders, Padding};

impl App {
    fn generate_datestamp_title() -> String {
        let now: DateTime<Utc> = Utc::now();
        now.format("%Y%m%d%H%M%S").to_string()
    }

    pub async fn create_thread_with_datestamp(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let title = Self::generate_datestamp_title();
        log::debug!("Creating thread with datestamp title: '{title}'");

        match self.api_client.create_thread(&title).await {
            Ok(thread) => {
                // Refresh the thread list to include the new thread
                self.load_threads().await?;

                // Load the newly created thread and navigate to it
                self.load_thread(&thread.id).await?;
                self.state = AppState::ThreadView(thread.id.clone());
                // Update selected index to point to the newly created thread
                if let Some(index) = self.find_thread_index_by_id(&thread.id) {
                    self.selected_thread_index = index;
                }
                self.mark_dirty();

                // Immediately transition to CreateEntry to open editor for first entry
                self.state = AppState::CreateEntry(thread.id);
                self.current_entry_image_path = None;
                self.text_editor.clear();
                self.setup_text_editor_block();
            }
            Err(e) => {
                log::error!("Failed to create thread with datestamp: {e}");
            }
        }
        Ok(())
    }

    pub async fn handle_save_operation(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let text = self.text_editor.lines().join("\n");
        log::debug!("Save operation - text: '{text}'");

        if text.trim().is_empty() {
            log::debug!("Save operation skipped - text is empty");
            return Ok(());
        }

        if text.len() > 500 {
            log::debug!("Save operation skipped - text too long: {} chars", text.len());
            let prev_state = self.state.clone();
            self.state = AppState::CharacterLimitError(Box::new(prev_state));
            return Ok(());
        }

        let current_state = self.state.clone();
        match current_state {
            AppState::CreateThread => match self.api_client.create_thread(&text).await {
                Ok(thread) => {
                    // Refresh the thread list to include the new thread
                    self.load_threads().await?;

                    // Load the newly created thread and navigate to it
                    self.load_thread(&thread.id).await?;
                    self.state = AppState::ThreadView(thread.id.clone());
                    // Update selected index to point to the newly created thread
                    if let Some(index) = self.find_thread_index_by_id(&thread.id) {
                        self.selected_thread_index = index;
                    }
                    self.text_editor.clear();
                    self.mark_dirty();

                    // Create backup after successful thread creation
                    if let Err(e) = self.create_backup().await {
                        log::warn!("Failed to create backup after thread creation: {e}");
                    }

                    // Immediately transition to CreateEntry to open editor for first entry
                    self.state = AppState::CreateEntry(thread.id);
                    self.current_entry_image_path = None;
                    self.setup_text_editor_block();
                }
                Err(e) => {
                    log::error!("Failed to create thread: {e}");
                }
            },
            AppState::EditThread(thread_id) => match self.api_client.update_thread(&thread_id, &text).await {
                Ok(_) => {
                    // Refresh the thread list to show updated title
                    self.load_threads().await?;
                    
                    // If we were in a thread view, reload the thread to show updated title
                    if let Some(current_thread) = &self.current_thread {
                        if current_thread.id == thread_id {
                            self.load_thread(&thread_id).await?;
                            self.state = AppState::ThreadView(thread_id.clone());
                            // Update selected index to point to the updated thread
                            if let Some(index) = self.find_thread_index_by_id(&thread_id) {
                                self.selected_thread_index = index;
                            }
                        } else {
                            self.state = AppState::ThreadList;
                        }
                    } else {
                        self.state = AppState::ThreadList;
                    }
                    
                    self.text_editor.clear();
                    self.mark_dirty();

                    // Create backup after successful thread update
                    if let Err(e) = self.create_backup().await {
                        log::warn!("Failed to create backup after thread update: {e}");
                    }
                }
                Err(e) => {
                    log::error!("Failed to update thread: {e}");
                }
            },
            AppState::CreateEntry(thread_id) => {
                let next_order = if let Some(thread) = &self.current_thread {
                    thread.entries.len() as i32 + 1
                } else {
                    1
                };

                match self
                    .api_client
                    .create_entry(&thread_id, &text, next_order, self.current_entry_image_path.clone())
                    .await
                {
                    Ok(_) => {
                        let thread_id_clone = thread_id.clone();
                        self.load_thread(&thread_id).await?;
                        self.state = AppState::ThreadView(thread_id_clone);
                        // Update selected index to point to the thread with new entry
                        if let Some(index) = self.find_thread_index_by_id(&thread_id) {
                            self.selected_thread_index = index;
                        }
                        // Update selected entry index to point to the newly created entry (last in list)
                        if let Some(thread) = &self.current_thread {
                            if !thread.entries.is_empty() {
                                self.selected_entry_index = thread.entries.len() - 1;
                            }
                        }
                        self.text_editor.clear();
                        self.mark_dirty();

                        // Create backup after successful entry creation
                        if let Err(e) = self.create_backup().await {
                            log::warn!("Failed to create backup after entry creation: {e}");
                        }
                    }
                    Err(e) => {
                        log::error!("Failed to create entry: {e}");
                    }
                }
            }
            AppState::EditEntry(thread_id, entry_id) => {
                if let Some(thread) = &self.current_thread {
                    if let Some(entry) = thread.entries.iter().find(|e| e.id == entry_id) {
                        match self
                            .api_client
                            .update_entry(&entry_id, &text, entry.order_num, self.current_entry_image_path.clone())
                            .await
                        {
                            Ok(_) => {
                                let thread_id_clone = thread_id.clone();
                                let entry_id_clone = entry_id.clone();
                                self.load_thread(&thread_id).await?;
                                self.state = AppState::ThreadView(thread_id_clone);
                                // Update selected index to point to the thread with updated entry
                                if let Some(index) = self.find_thread_index_by_id(&thread_id) {
                                    self.selected_thread_index = index;
                                }
                                // Update selected entry index to point to the updated entry
                                if let Some(entry_index) = self.find_entry_index_by_id(&entry_id_clone) {
                                    self.selected_entry_index = entry_index;
                                }
                                self.text_editor.clear();
                                self.original_entry_content = None;
                                self.mark_dirty();

                                // Create backup after successful entry update
                                if let Err(e) = self.create_backup().await {
                                    log::warn!("Failed to create backup after entry update: {e}");
                                }
                            }
                            Err(e) => {
                                log::error!("Failed to update entry: {e}");
                            }
                        }
                    }
                }
            }
            _ => {
                log::warn!(
                    "Save operation called from invalid state: {current_state:?}"
                );
            }
        }
        Ok(())
    }

    fn transition_to_image_naming(&mut self, image_data: Vec<u8>) {
        log::debug!("ImageNamingModal result received, transitioning to ImageNaming state");

        // Extract previous state
        let prev_state = match &self.state {
            AppState::CreateThread => AppState::CreateThread,
            AppState::CreateEntry(id) => AppState::CreateEntry(id.clone()),
            AppState::EditEntry(t_id, e_id) => AppState::EditEntry(t_id.clone(), e_id.clone()),
            _ => AppState::CreateThread, // fallback
        };

        log::debug!(
            "Previous state: {:?}, image data length: {} bytes",
            prev_state,
            image_data.len()
        );

        // Save current text editor content before transitioning
        self.saved_text_content = Some(self.text_editor.lines().join("\n"));

        // Transition to ImageNaming state
        self.state = AppState::ImageNaming(Box::new(prev_state), image_data);

        log::debug!("State changed to ImageNaming");

        // Clear and setup modal text editor for filename input
        self.modal_text_editor.clear();
        self.modal_text_editor.set_block(
            ratatui::widgets::Block::default()
                .borders(ratatui::widgets::Borders::ALL)
                .title("Enter filename"),
        );
        self.mark_dirty();

        log::debug!("Text editor cleared and block set for filename input");
    }

    fn setup_text_editor_block(&mut self) {
        match &self.state {
            AppState::CreateThread => {
                self.text_editor
                    .set_block(Block::default().borders(Borders::ALL).title("Thread Title"));
            }
            AppState::EditThread(_) => {
                self.text_editor
                    .set_block(Block::default().borders(Borders::ALL).title("Edit Thread Title"));
            }
            AppState::CreateEntry(_) => {
                self.text_editor.set_block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title("New Entry Content")
                        .padding(Padding::vertical(1)),
                );
            }
            AppState::EditEntry(_, _) => {
                self.text_editor
                    .set_block(Block::default().borders(Borders::ALL).title("Edit Entry"));
            }
            _ => {}
        }
    }

    async fn handle_thread_list_keys(
        &mut self,
        key: KeyEvent,
    ) -> Result<(), Box<dyn std::error::Error>> {
        match key.code {
            KeyCode::Char('q') => {
                log::debug!("Quitting application");
                self.should_quit = true;
            }
            KeyCode::Char('n') => {
                log::debug!("Switching to CreateThread state");
                self.state = AppState::CreateThread;
                self.text_editor.clear();
                self.setup_text_editor_block();
            }
            KeyCode::Char('d') => {
                log::debug!("Creating thread with datestamp");
                self.create_thread_with_datestamp().await?;
            }
            KeyCode::Up => {
                if self.selected_thread_index > 0 {
                    self.selected_thread_index -= 1;
                }
            }
            KeyCode::Down => {
                if self.selected_thread_index < self.threads.len().saturating_sub(1) {
                    self.selected_thread_index += 1;
                }
            }
            KeyCode::Enter => {
                if self.selected_thread_index < self.threads.len() {
                    let thread_id = self.threads[self.selected_thread_index].id.clone();
                    self.load_thread(&thread_id).await?;
                    self.state = AppState::ThreadView(thread_id);
                    // selected_thread_index is already correct since we're using it to select the thread
                }
            }
            KeyCode::Char('r') => {
                if self.selected_thread_index < self.threads.len() {
                    let thread_id = self.threads[self.selected_thread_index].id.clone();
                    let thread_title = self.threads[self.selected_thread_index].title.clone();
                    self.state = AppState::EditThread(thread_id);
                    self.text_editor.clear();
                    self.text_editor.set_wrap_width(self.wrap_width);
                    self.text_editor.set_text_without_image_processing(&thread_title);
                    self.setup_text_editor_block();
                }
            }
            KeyCode::Delete | KeyCode::Backspace => {
                if self.selected_thread_index < self.threads.len() {
                    let thread_id = self.threads[self.selected_thread_index].id.clone();
                    self.state = AppState::ConfirmDeleteThread(thread_id);
                }
            }
            _ => {}
        }
        Ok(())
    }

    async fn handle_thread_view_keys(
        &mut self,
        key: KeyEvent,
        thread_id: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => {
                self.state = AppState::ThreadList;
                self.current_thread = None;
                self.selected_entry_index = 0;
            }
            KeyCode::Char('n') => {
                self.state = AppState::CreateEntry(thread_id.to_string());
                self.current_entry_image_path = None;
                self.text_editor.clear();
                self.setup_text_editor_block();
            }
            KeyCode::Char('r') => {
                if let Some(thread) = &self.current_thread {
                    self.state = AppState::EditThread(thread_id.to_string());
                    self.text_editor.clear();
                    self.text_editor.set_wrap_width(self.wrap_width);
                    self.text_editor.set_text_without_image_processing(&thread.title);
                    self.setup_text_editor_block();
                }
            }
            KeyCode::Up => {
                let has_ctrl = key
                    .modifiers
                    .contains(ratatui::crossterm::event::KeyModifiers::CONTROL);
                let has_shift = key
                    .modifiers
                    .contains(ratatui::crossterm::event::KeyModifiers::SHIFT);
                
                if has_ctrl && has_shift {
                    // Ctrl+Shift+Up: Move entry up
                    self.move_entry_up(thread_id).await?;
                } else if self.selected_entry_index > 0 {
                    let new_index = self.selected_entry_index - 1;
                    self.selected_entry_index = new_index;
                }
            }
            KeyCode::Down => {
                let has_ctrl = key
                    .modifiers
                    .contains(ratatui::crossterm::event::KeyModifiers::CONTROL);
                let has_shift = key
                    .modifiers
                    .contains(ratatui::crossterm::event::KeyModifiers::SHIFT);
                
                if has_ctrl && has_shift {
                    // Ctrl+Shift+Down: Move entry down
                    self.move_entry_down(thread_id).await?;
                } else if let Some(thread) = &self.current_thread {
                    if self.selected_entry_index < thread.entries.len().saturating_sub(1) {
                        let new_index = self.selected_entry_index + 1;
                        self.selected_entry_index = new_index;
                    }
                }
            }
            KeyCode::PageUp => {
                // For single entry mode, use old scroll method
                if self.preview_scroll_offset > 0 {
                    self.preview_scroll_offset = self.preview_scroll_offset.saturating_sub(5);
                }
            }
            KeyCode::PageDown => {
                // For single entry mode, use old scroll method
                self.preview_scroll_offset = self.preview_scroll_offset.saturating_add(5);
            }
            KeyCode::Enter | KeyCode::Char('e') => {
                if let Some(thread) = &self.current_thread {
                    if self.selected_entry_index < thread.entries.len() {
                        let entry_id = thread.entries[self.selected_entry_index].id.clone();
                        let content = thread.entries[self.selected_entry_index].content.clone();
                        let image_path = thread.entries[self.selected_entry_index].image_path.clone();
                        self.state = AppState::EditEntry(thread_id.to_string(), entry_id.clone());
                        self.current_entry_image_path = image_path;
                        self.original_entry_content = Some(content.clone());
                        
                        // Clear text editor first
                        self.text_editor.clear();
                        
                        // Load cached thumbnail to avoid reprocessing
                        self.load_cached_thumbnail_into_editor(&entry_id);
                        
                        // Content is already unwrapped in database - use directly for editing
                        self.text_editor.set_wrap_width(self.wrap_width);
                        self.text_editor.set_text_without_image_processing(&content);
                        self.text_editor.move_cursor_to_start();
                        self.setup_text_editor_block();
                    }
                }
            }
            KeyCode::Delete | KeyCode::Backspace => {
                if let Some(thread) = &self.current_thread {
                    if self.selected_entry_index < thread.entries.len() {
                        let entry_id = thread.entries[self.selected_entry_index].id.clone();
                        self.state = AppState::ConfirmDeleteEntry(thread_id.to_string(), entry_id);
                    }
                }
            }
            KeyCode::Char('t')
                if key
                    .modifiers
                    .contains(ratatui::crossterm::event::KeyModifiers::CONTROL) =>
            {
                // Toggle thread view image preview with Ctrl+T
                self.thread_view_image_preview_visible = !self.thread_view_image_preview_visible;
                log::debug!(
                    "ThreadView: toggled image preview visibility to: {}",
                    self.thread_view_image_preview_visible
                );
                // Thumbnails are already pre-cached, so toggle is instant
            }
            KeyCode::Char('x') => {
                match self.export_thread(thread_id).await {
                    Ok(markdown) => {
                        // For now, let's save it to a file and log success
                        if let Some(thread) = &self.current_thread {
                            let sanitized_title = thread
                                .title
                                .chars()
                                .map(|c| {
                                    if c.is_alphanumeric() || c == '_' || c == '-' {
                                        c
                                    } else {
                                        '_'
                                    }
                                })
                                .collect::<String>()
                                .to_lowercase();
                            let filename = format!("{sanitized_title}.md");
                            match tokio::fs::write(&filename, markdown).await {
                                Ok(_) => {
                                    log::info!("Thread exported to {filename}");
                                    // Could add a status message to the UI here
                                }
                                Err(e) => {
                                    log::error!("Failed to save exported file: {e}");
                                }
                            }
                        }
                    }
                    Err(e) => {
                        log::error!("Failed to export thread: {e}");
                    }
                }
            }
            _ => {}
        }
        Ok(())
    }

    pub async fn move_entry_up(&mut self, thread_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(thread) = &self.current_thread {
            if self.selected_entry_index > 0 && !thread.entries.is_empty() {
                let current_index = self.selected_entry_index;
                let target_index = current_index - 1;
                
                // Create the reordered entries list for the API call
                let mut reordered_entries = Vec::new();
                
                for (index, entry) in thread.entries.iter().enumerate() {
                    let new_order_num = if index == current_index {
                        // Current entry moves up (gets previous entry's order)
                        (target_index + 1) as i32
                    } else if index == target_index {
                        // Previous entry moves down (gets current entry's order)
                        (current_index + 1) as i32
                    } else {
                        // Other entries keep their current order
                        entry.order_num
                    };
                    
                    reordered_entries.push((entry.id.clone(), new_order_num));
                }
                
                // Send reorder request to API
                match self.api_client.reorder_entries(thread_id, reordered_entries).await {
                    Ok(updated_thread) => {
                        self.current_thread = Some(updated_thread.clone());
                        
                        // Also update the thread in the threads list to keep it in sync
                        if let Some(thread_index) = self.find_thread_index_by_id(thread_id) {
                            self.threads[thread_index] = updated_thread;
                        }
                        
                        self.selected_entry_index = target_index; // Follow the moved entry
                        self.mark_dirty();
                        log::info!("Entry {} moved up to position {}", current_index + 1, target_index + 1);
                    }
                    Err(e) => {
                        log::error!("Failed to move entry up: {e}");
                    }
                }
            }
        }
        Ok(())
    }

    pub async fn move_entry_down(&mut self, thread_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(thread) = &self.current_thread {
            if self.selected_entry_index < thread.entries.len().saturating_sub(1) && !thread.entries.is_empty() {
                let current_index = self.selected_entry_index;
                let target_index = current_index + 1;
                
                // Create the reordered entries list for the API call
                let mut reordered_entries = Vec::new();
                
                for (index, entry) in thread.entries.iter().enumerate() {
                    let new_order_num = if index == current_index {
                        // Current entry moves down (gets next entry's order)
                        (target_index + 1) as i32
                    } else if index == target_index {
                        // Next entry moves up (gets current entry's order)
                        (current_index + 1) as i32
                    } else {
                        // Other entries keep their current order
                        entry.order_num
                    };
                    
                    reordered_entries.push((entry.id.clone(), new_order_num));
                }
                
                // Send reorder request to API
                match self.api_client.reorder_entries(thread_id, reordered_entries).await {
                    Ok(updated_thread) => {
                        self.current_thread = Some(updated_thread.clone());
                        
                        // Also update the thread in the threads list to keep it in sync
                        if let Some(thread_index) = self.find_thread_index_by_id(thread_id) {
                            self.threads[thread_index] = updated_thread;
                        }
                        
                        self.selected_entry_index = target_index; // Follow the moved entry
                        self.mark_dirty();
                        log::info!("Entry {} moved down to position {}", current_index + 1, target_index + 1);
                    }
                    Err(e) => {
                        log::error!("Failed to move entry down: {e}");
                    }
                }
            }
        }
        Ok(())
    }

    pub async fn handle_key_event(
        &mut self,
        key: KeyEvent,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // log::debug!("Handling key: {:?} in state: {:?}", key, self.state);
        log::debug!(
            "Key details - Code: {:?}, Modifiers: {:?}, Kind: {:?}",
            key.code,
            key.modifiers,
            key.kind
        );
        log::debug!("Raw modifier bits: {:?}", key.modifiers.bits());
        let current_state = self.state.clone();
        match &current_state {
            AppState::ThreadList => {
                self.handle_thread_list_keys(key).await?;
            }
            AppState::ThreadView(thread_id) => {
                self.handle_thread_view_keys(key, thread_id).await?;
            }
            AppState::CreateThread => {
                log::debug!(
                    "CreateThread state - Key: {:?}, Modifiers: {:?}",
                    key.code,
                    key.modifiers
                );

                // Check for Cmd+S or Ctrl+S first
                if key.code == KeyCode::Char('s') || key.code == KeyCode::Char('S') {
                    let has_ctrl = key
                        .modifiers
                        .contains(ratatui::crossterm::event::KeyModifiers::CONTROL);
                    let has_super = key
                        .modifiers
                        .contains(ratatui::crossterm::event::KeyModifiers::SUPER);
                    if has_ctrl || has_super {
                        log::debug!(
                            "Submit key combination detected for creating thread (Ctrl+S or Cmd+S)"
                        );
                        self.handle_save_operation().await?;
                        // self.mark_dirty()
                        return Ok(());
                    }
                }

                match key.code {
                    KeyCode::Enter => {
                        log::debug!("Enter key pressed for creating thread");
                        self.handle_save_operation().await?;
                        return Ok(());
                    }
                    KeyCode::Esc => {
                        self.state = AppState::ThreadList;
                        self.text_editor.clear();
                    }
                    _ => {
                        // Delegate to text editor for simple text input (no images or newlines needed for thread titles)
                        if let Some(KeyResult::ImageNamingModal(image_data)) = self.text_editor.handle_key_event(key) {
                            self.transition_to_image_naming(image_data);
                        }
                    }
                }
            }
            AppState::EditThread(thread_id) => {
                log::debug!(
                    "EditThread state - Key: {:?}, Modifiers: {:?}",
                    key.code,
                    key.modifiers
                );

                // Check for Cmd+S or Ctrl+S first
                if key.code == KeyCode::Char('s') || key.code == KeyCode::Char('S') {
                    let has_ctrl = key
                        .modifiers
                        .contains(ratatui::crossterm::event::KeyModifiers::CONTROL);
                    let has_super = key
                        .modifiers
                        .contains(ratatui::crossterm::event::KeyModifiers::SUPER);
                    if has_ctrl || has_super {
                        log::debug!(
                            "Submit key combination detected for updating thread (Ctrl+S or Cmd+S)"
                        );
                        self.handle_save_operation().await?;
                        return Ok(());
                    }
                }

                match key.code {
                    KeyCode::Enter => {
                        log::debug!("Enter key pressed for updating thread");
                        self.handle_save_operation().await?;
                        return Ok(());
                    }
                    KeyCode::Esc => {
                        // Determine where to go back to based on context
                        if let Some(current_thread) = &self.current_thread {
                            if current_thread.id == *thread_id {
                                self.state = AppState::ThreadView(thread_id.clone());
                                // Update selected index to point to the current thread
                                if let Some(index) = self.find_thread_index_by_id(thread_id) {
                                    self.selected_thread_index = index;
                                }
                            } else {
                                self.state = AppState::ThreadList;
                            }
                        } else {
                            self.state = AppState::ThreadList;
                        }
                        self.text_editor.clear();
                    }
                    _ => {
                        // Delegate to text editor for simple text input (no images or newlines needed for thread titles)
                        if let Some(KeyResult::ImageNamingModal(image_data)) = self.text_editor.handle_key_event(key) {
                            self.transition_to_image_naming(image_data);
                        }
                    }
                }
            }
            AppState::CreateEntry(thread_id) => {
                log::debug!(
                    "CreateEntry state - Key: {:?}, Modifiers: {:?}",
                    key.code,
                    key.modifiers
                );

                // Check for Cmd+S or Ctrl+S first
                if key.code == KeyCode::Char('s') || key.code == KeyCode::Char('S') {
                    let has_ctrl = key
                        .modifiers
                        .contains(ratatui::crossterm::event::KeyModifiers::CONTROL);
                    let has_super = key
                        .modifiers
                        .contains(ratatui::crossterm::event::KeyModifiers::SUPER);
                    if has_ctrl || has_super {
                        log::debug!(
                            "Submit key combination detected for creating entry (Ctrl+S or Cmd+S)"
                        );
                        self.handle_save_operation().await?;
                        return Ok(());
                    }
                }

                match key.code {
                    KeyCode::Esc => {
                        // Check if there's any content that would be lost
                        let current_content = self.text_editor.lines().join("\n");
                        let has_content = !current_content.trim().is_empty();

                        if has_content {
                            // Show confirmation modal
                            self.state = AppState::ConfirmDiscardNewEntry(thread_id.clone());
                        } else {
                            // No content, go back directly
                            self.state = AppState::ThreadView(thread_id.clone());
                            // Update selected index to point to the current thread
                            if let Some(index) = self.find_thread_index_by_id(thread_id) {
                                self.selected_thread_index = index;
                            }
                            self.text_editor.clear();
                        }
                    }
                    KeyCode::Char('t')
                        if key
                            .modifiers
                            .contains(ratatui::crossterm::event::KeyModifiers::CONTROL) =>
                    {
                        // Toggle image preview with Ctrl+T (T for Toggle)
                        self.text_editor.toggle_image_preview();
                    }
                    KeyCode::Char('f')
                        if key
                            .modifiers
                            .contains(ratatui::crossterm::event::KeyModifiers::CONTROL) =>
                    {
                        // Toggle image full screen with Ctrl+F (F for Full screen)
                        self.text_editor.toggle_image_full_screen();
                    }
                    _ => {
                        // Delegate to text editor (including Enter for newlines)
                        if let Some(KeyResult::ImageNamingModal(image_data)) = self.text_editor.handle_key_event(key) {
                            self.transition_to_image_naming(image_data);
                        }
                    }
                }
            }
            AppState::EditEntry(thread_id, _entry_id) => {
                log::debug!(
                    "EditEntry state - Key: {:?}, Modifiers: {:?}",
                    key.code,
                    key.modifiers
                );

                // Check for Cmd+S or Ctrl+S first - only if modifiers are present
                if (key.code == KeyCode::Char('s') || key.code == KeyCode::Char('S'))
                    && (key
                        .modifiers
                        .contains(ratatui::crossterm::event::KeyModifiers::CONTROL)
                        || key
                            .modifiers
                            .contains(ratatui::crossterm::event::KeyModifiers::SUPER))
                {
                    log::debug!(
                        "Submit key combination detected for updating entry (Ctrl+S or Cmd+S)"
                    );
                    self.handle_save_operation().await?;
                    return Ok(());
                } else {
                    match key.code {
                        KeyCode::Esc => {
                            // Check if there are unsaved changes
                            let current_content = self.text_editor.lines().join("\n");
                            let has_changes = if let Some(original) = &self.original_entry_content {
                                // Original content is now stored unwrapped, so compare directly
                                current_content != *original
                            } else {
                                !current_content.trim().is_empty()
                            };

                            if has_changes {
                                // Show confirmation modal
                                self.state = AppState::ConfirmDiscardEntryChanges(thread_id.clone(), _entry_id.clone());
                            } else {
                                // No changes, go back directly
                                self.state = AppState::ThreadView(thread_id.clone());
                                // Update selected index to point to the current thread
                                if let Some(index) = self.find_thread_index_by_id(thread_id) {
                                    self.selected_thread_index = index;
                                }
                                // Update selected entry index to point to the entry we were editing
                                if let Some(entry_index) = self.find_entry_index_by_id(_entry_id) {
                                    self.selected_entry_index = entry_index;
                                }
                                self.text_editor.clear();
                                self.original_entry_content = None;
                            }
                        }
                        KeyCode::Char('t')
                            if key
                                .modifiers
                                .contains(ratatui::crossterm::event::KeyModifiers::CONTROL) =>
                        {
                            // Toggle image preview with Ctrl+T (T for Toggle)
                            self.text_editor.toggle_image_preview();
                        }
                        KeyCode::Char('f')
                            if key
                                .modifiers
                                .contains(ratatui::crossterm::event::KeyModifiers::CONTROL) =>
                        {
                            // Toggle image full screen with Ctrl+F (F for Full screen)
                            self.text_editor.toggle_image_full_screen();
                        }
                        _ => {
                            // Delegate to text editor (including Enter for newlines)
                            if let Some(KeyResult::ImageNamingModal(image_data)) = self.text_editor.handle_key_event(key) {
                                self.transition_to_image_naming(image_data);
                            }
                        }
                    }
                }
            }
            AppState::ConfirmDeleteThread(thread_id) => {
                match key.code {
                    KeyCode::Char('y') | KeyCode::Char('Y') => {
                        match self.api_client.delete_thread(thread_id).await {
                            Ok(_) => {
                                self.load_threads().await?;
                                // Adjust selected index if needed
                                if self.selected_thread_index >= self.threads.len()
                                    && !self.threads.is_empty()
                                {
                                    self.selected_thread_index = self.threads.len() - 1;
                                }
                                log::debug!("Successfully deleted thread: {thread_id}");
                            }
                            Err(e) => {
                                log::error!("Failed to delete thread: {e}");
                            }
                        }
                        self.state = AppState::ThreadList;
                    }
                    KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                        self.state = AppState::ThreadList;
                    }
                    _ => {}
                }
            }
            AppState::ConfirmDeleteEntry(thread_id, entry_id) => {
                match key.code {
                    KeyCode::Char('y') | KeyCode::Char('Y') => {
                        match self.api_client.delete_entry(entry_id).await {
                            Ok(_) => {
                                self.load_thread(thread_id).await?;
                                // Adjust selected index if needed
                                if let Some(updated_thread) = &self.current_thread {
                                    if self.selected_entry_index >= updated_thread.entries.len()
                                        && !updated_thread.entries.is_empty()
                                    {
                                        self.selected_entry_index =
                                            updated_thread.entries.len() - 1;
                                    }
                                }
                                log::debug!("Successfully deleted entry: {entry_id}");
                            }
                            Err(e) => {
                                log::error!("Failed to delete entry: {e}");
                            }
                        }
                        self.state = AppState::ThreadView(thread_id.clone());
                        // Update selected index to point to the current thread
                        if let Some(index) = self.find_thread_index_by_id(thread_id) {
                            self.selected_thread_index = index;
                        }
                    }
                    KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                        self.state = AppState::ThreadView(thread_id.clone());
                        // Update selected index to point to the current thread
                        if let Some(index) = self.find_thread_index_by_id(thread_id) {
                            self.selected_thread_index = index;
                        }
                    }
                    _ => {}
                }
            }
            AppState::ConfirmDiscardEntryChanges(thread_id, entry_id) => {
                match key.code {
                    KeyCode::Char('y') | KeyCode::Char('Y') => {
                        // Discard changes and exit
                        self.state = AppState::ThreadView(thread_id.clone());
                        // Update selected index to point to the current thread
                        if let Some(index) = self.find_thread_index_by_id(thread_id) {
                            self.selected_thread_index = index;
                        }
                        // Update selected entry index to point to the entry we were editing
                        if let Some(entry_index) = self.find_entry_index_by_id(entry_id) {
                            self.selected_entry_index = entry_index;
                        }
                        self.text_editor.clear();
                        self.original_entry_content = None;
                    }
                    KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                        // Return to editing
                        self.state = AppState::EditEntry(thread_id.clone(), entry_id.clone());
                    }
                    _ => {}
                }
            }
            AppState::ConfirmDiscardNewEntry(thread_id) => {
                match key.code {
                    KeyCode::Char('y') | KeyCode::Char('Y') => {
                        // Discard new entry and exit
                        self.state = AppState::ThreadView(thread_id.clone());
                        // Update selected index to point to the current thread
                        if let Some(index) = self.find_thread_index_by_id(thread_id) {
                            self.selected_thread_index = index;
                        }
                        self.text_editor.clear();
                    }
                    KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                        // Return to creating entry
                        self.state = AppState::CreateEntry(thread_id.clone());
                    }
                    _ => {}
                }
            }
            AppState::ImageNaming(prev_state, image_data) => {
                match key.code {
                    KeyCode::Enter => {
                        // Save image with custom filename
                        let filename = self.modal_text_editor.lines().join("").trim().to_string();
                        if !filename.is_empty() {
                            // Extract context from previous state
                            let (thread_id, entry_id) = match &**prev_state {
                                AppState::CreateThread => (None, None),
                                AppState::CreateEntry(t_id) => (Some(t_id.as_str()), None),
                                AppState::EditEntry(t_id, e_id) => {
                                    (Some(t_id.as_str()), Some(e_id.as_str()))
                                }
                                _ => (None, None),
                            };

                            match save_image_with_context(
                                image_data, &filename, thread_id, entry_id,
                            ) {
                                Ok(image_path) => {
                                    // Store image path for database storage
                                    self.current_entry_image_path = Some(image_path.clone());

                                    // Restore previous state and text content
                                    let prev_state_clone = (**prev_state).clone();
                                    self.state = prev_state_clone;

                                    // Restore saved text content
                                    if let Some(saved_content) = &self.saved_text_content {
                                        self.text_editor.set_text(saved_content);
                                    }

                                    // Setup text editor based on previous state
                                    self.setup_text_editor_block();

                                    // Clear saved content
                                    self.saved_text_content = None;

                                    // Load image into preview
                                    if let Err(e) = self.text_editor.load_image(&image_path) {
                                        log::error!("Failed to load image into preview: {e}");
                                    }
                                }
                                Err(e) => {
                                    log::error!("Failed to save image: {e}");
                                }
                            }
                        }
                    }
                    KeyCode::Esc => {
                        // Cancel and restore previous state and text editor state
                        let prev_state_clone = (**prev_state).clone();
                        self.state = prev_state_clone;

                        // Restore saved text content
                        if let Some(saved_content) = &self.saved_text_content {
                            self.text_editor.set_text(saved_content);
                        }

                        // Clear saved content
                        self.saved_text_content = None;

                        // Restore text editor block title based on previous state
                        self.setup_text_editor_block();
                    }
                    _ => {
                        // Handle text input for filename using modal text editor
                        let _result = self.modal_text_editor.handle_key_event(key);
                    }
                }
            }
            AppState::CharacterLimitError(prev_state) => {
                match key.code {
                    KeyCode::Enter | KeyCode::Esc => {
                        // Return to previous state
                        let prev_state_clone = (**prev_state).clone();
                        self.state = prev_state_clone;
                    }
                    _ => {
                        // Ignore other keys in error modal
                    }
                }
            }
        }

        // Mark dirty for all key events to ensure redraws
        // This is more efficient than individual mark_dirty calls scattered throughout
        // Mouse events are filtered separately to avoid unnecessary redraws on mouse moves
        self.mark_dirty();

        Ok(())
    }

    pub async fn handle_mouse(
        &mut self,
        mouse: MouseEvent,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // log::info!("App::handle_mouse - Event: {:?} at ({}, {}) in state: {:?}", mouse.kind, mouse.column, mouse.row, self.state);
        // Filter out mouse move events that don't need redraws
        if matches!(mouse.kind, ratatui::crossterm::event::MouseEventKind::Moved) {
            // Only handle move events if we're in a text editing state (for text selection)
            if !matches!(
                self.state,
                AppState::CreateThread
                    | AppState::EditThread(_)
                    | AppState::CreateEntry(_)
                    | AppState::EditEntry(_, _)
                    | AppState::ImageNaming(_, _)
            ) {
                return Ok(());
            }
        }

        // Only log non-move mouse events to reduce log noise
        if !matches!(mouse.kind, ratatui::crossterm::event::MouseEventKind::Moved) {
            log::debug!(
                "Mouse event: {:?} at ({}, {}) in state: {:?}",
                mouse.kind,
                mouse.column,
                mouse.row,
                self.state
            );
        }

        // For text editing states, handle mouse events specially
        match &self.state {
            AppState::CreateThread | AppState::EditThread(_) | AppState::CreateEntry(_) | AppState::EditEntry(_, _) => {
                // Check if click is on submit button first
                if mouse.kind
                    == ratatui::crossterm::event::MouseEventKind::Down(
                        ratatui::crossterm::event::MouseButton::Left,
                    )
                    && self.is_click_on_submit_button(mouse.column, mouse.row) {
                        self.handle_submit_button_click().await?;
                        return Ok(());
                    }

                // Try to handle mouse event in text editor
                if self.text_editor.handle_mouse_event(mouse)? {
                    self.mark_dirty();
                    return Ok(());
                }
            }
            AppState::ImageNaming(_, _) => {
                // For image naming modal, use modal text editor
                if self.modal_text_editor.handle_mouse_event(mouse)? {
                    self.mark_dirty();
                    return Ok(());
                }
            }
            AppState::CharacterLimitError(_) => {
                // For character limit error modal, ignore mouse events
                return Ok(());
            }
            _ => {}
        }

        // For other mouse events, use the original handling
        match mouse.kind {
            ratatui::crossterm::event::MouseEventKind::Down(
                ratatui::crossterm::event::MouseButton::Left,
            ) => {
                self.handle_mouse_click(mouse.column, mouse.row).await?;
            }
            ratatui::crossterm::event::MouseEventKind::ScrollUp => {
                self.handle_scroll_up().await?;
            }
            ratatui::crossterm::event::MouseEventKind::ScrollDown => {
                self.handle_scroll_down().await?;
            }
            ratatui::crossterm::event::MouseEventKind::Down(_)
            | ratatui::crossterm::event::MouseEventKind::Up(_)
            | ratatui::crossterm::event::MouseEventKind::Drag(_)
            | ratatui::crossterm::event::MouseEventKind::Moved
            | ratatui::crossterm::event::MouseEventKind::ScrollLeft
            | ratatui::crossterm::event::MouseEventKind::ScrollRight => {
                // These events are now handled in the text editor
            }
        }
        Ok(())
    }

    async fn handle_mouse_click(
        &mut self,
        _column: u16,
        row: u16,
    ) -> Result<(), Box<dyn std::error::Error>> {
        match &self.state {
            AppState::ThreadList => {
                // Calculate which thread was clicked based on row
                // Skip the first row (header)
                if row > 0 {
                    let thread_index = (row - 1) as usize;
                    if thread_index < self.threads.len() {
                        self.selected_thread_index = thread_index;
                        // Double-click behavior: open thread
                        let thread_id = self.threads[self.selected_thread_index].id.clone();
                        self.load_thread(&thread_id).await?;
                        self.state = AppState::ThreadView(thread_id);
                        // selected_thread_index is already correct since we're using it to select the thread
                        self.mark_dirty();
                    }
                }
            }
            AppState::ThreadView(thread_id) => {
                let thread_id = thread_id.clone(); // Clone to avoid borrow issues
                
                // Check if the click was on a specific entry in the list
                if let Some(clicked_entry_index) = self.get_clicked_entry_index(_column, row) {
                    if clicked_entry_index == self.selected_entry_index {
                        // Clicked on already selected entry - open for editing
                        self.open_selected_entry_for_editing(&thread_id).await?;
                    } else {
                        // Clicked on different entry - select it
                        if let Some(thread) = &self.current_thread {
                            if clicked_entry_index < thread.entries.len() {
                                self.selected_entry_index = clicked_entry_index;
                                self.mark_dirty();
                            }
                        }
                    }
                } else {
                    // Check if click was within the entry list area but not on an entry
                    if let Some(entry_list_area) = self.entry_list_area {
                        if _column >= entry_list_area.x 
                            && _column < entry_list_area.x + entry_list_area.width 
                            && row >= entry_list_area.y 
                            && row < entry_list_area.y + entry_list_area.height 
                        {
                            // Click was in entry list area but not on an entry - do nothing
                            // This prevents accidental editing when clicking on empty space
                        } else {
                            // Click was completely outside the entry list - open currently selected entry for editing
                            self.open_selected_entry_for_editing(&thread_id).await?;
                        }
                    } else {
                        // No entry list area stored - fall back to original behavior
                        self.open_selected_entry_for_editing(&thread_id).await?;
                    }
                }
            }
            AppState::CreateThread | AppState::EditThread(_) | AppState::CreateEntry(_) | AppState::EditEntry(_, _) => {
                // Mouse events for text areas are now handled by the text editor
                log::debug!("Mouse click in text editing state handled by text editor");
            }
            AppState::ImageNaming(_, _) => {
                // Mouse events for image naming modal are handled by the text editor
                log::debug!("Mouse click in image naming modal handled by text editor");
            }
            AppState::CharacterLimitError(_) => {
                // Ignore mouse events in character limit error modal - only keyboard input accepted
                log::debug!("Mouse click in character limit error modal ignored");
            }
            AppState::ConfirmDeleteThread(_) | AppState::ConfirmDeleteEntry(_, _) | AppState::ConfirmDiscardEntryChanges(_, _) | AppState::ConfirmDiscardNewEntry(_) => {
                // Ignore mouse events in confirmation dialogs - only keyboard input accepted
            }
        }
        Ok(())
    }

    fn is_click_on_submit_button(&self, column: u16, row: u16) -> bool {
        if let Some(area) = self.submit_button_area {
            row >= area.y
                && row < area.y + area.height
                && column >= area.x
                && column < area.x + area.width
        } else {
            false
        }
    }

    async fn handle_submit_button_click(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.handle_save_operation().await
    }

    async fn handle_scroll_up(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Check if enough time has passed since last scroll
        let now = std::time::Instant::now();
        if now.duration_since(self.last_scroll_time).as_millis() < self.scroll_throttle_ms as u128 {
            return Ok(());
        }
        self.last_scroll_time = now;

        match &self.state {
            AppState::ThreadList => {
                // Scroll up in thread list
                if self.selected_thread_index > 0 {
                    self.selected_thread_index -= 1;
                    self.mark_dirty();
                }
            }
            AppState::ThreadView(_) => {
                // Scroll up in entry list - update both selection and scroll offset
                if self.selected_entry_index > 0 {
                    self.selected_entry_index -= 1;
                    
                    // Update scroll offset to keep selected entry visible
                    if self.selected_entry_index < self.entry_list_scroll_offset as usize {
                        self.entry_list_scroll_offset = self.selected_entry_index as u16;
                    }
                    
                    self.mark_dirty();
                }
            }
            AppState::CreateThread | AppState::EditThread(_) | AppState::CreateEntry(_) | AppState::EditEntry(_, _) => {
                // Scroll up in text editor
                self.text_editor.scroll_up();
                self.mark_dirty();
            }
            _ => {}
        }
        Ok(())
    }

    async fn handle_scroll_down(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Check if enough time has passed since last scroll
        let now = std::time::Instant::now();
        if now.duration_since(self.last_scroll_time).as_millis() < self.scroll_throttle_ms as u128 {
            return Ok(());
        }
        self.last_scroll_time = now;

        match &self.state {
            AppState::ThreadList => {
                // Scroll down in thread list
                if self.selected_thread_index < self.threads.len().saturating_sub(1) {
                    self.selected_thread_index += 1;
                    self.mark_dirty();
                }
            }
            AppState::ThreadView(_) => {
                // Scroll down in entry list - update both selection and scroll offset
                if let Some(thread) = &self.current_thread {
                    if self.selected_entry_index < thread.entries.len().saturating_sub(1) {
                        self.selected_entry_index += 1;
                        
                        // Update scroll offset to keep selected entry visible
                        // Calculate visible height (account for borders and padding)
                        if let Some(list_area) = self.entry_list_area {
                            let visible_height = list_area.height.saturating_sub(4) as usize; // -2 borders, -2 padding
                            let max_visible_index = self.entry_list_scroll_offset as usize + visible_height.saturating_sub(1);
                            
                            if self.selected_entry_index > max_visible_index {
                                self.entry_list_scroll_offset = (self.selected_entry_index + 1).saturating_sub(visible_height) as u16;
                            }
                        }
                        
                        self.mark_dirty();
                    }
                }
            }
            AppState::CreateThread | AppState::EditThread(_) | AppState::CreateEntry(_) | AppState::EditEntry(_, _) => {
                // Scroll down in text editor
                self.text_editor.scroll_down();
                self.mark_dirty();
            }
            _ => {}
        }
        Ok(())
    }

    async fn open_selected_entry_for_editing(&mut self, thread_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(thread) = &self.current_thread {
            if self.selected_entry_index < thread.entries.len() {
                let entry_id = thread.entries[self.selected_entry_index].id.clone();
                let content = thread.entries[self.selected_entry_index].content.clone();
                let image_path = thread.entries[self.selected_entry_index].image_path.clone();
                self.state = AppState::EditEntry(thread_id.to_string(), entry_id.clone());
                self.current_entry_image_path = image_path;
                self.original_entry_content = Some(content.clone());
                
                // Clear text editor first
                self.text_editor.clear();
                
                // Load cached thumbnail to avoid reprocessing
                self.load_cached_thumbnail_into_editor(&entry_id);
                
                // Content is already unwrapped in database - use directly for editing
                self.text_editor.set_wrap_width(self.wrap_width);
                self.text_editor.set_text_without_image_processing(&content);
                self.text_editor.move_cursor_to_start();
                self.setup_text_editor_block();
                self.mark_dirty();
            }
        }
        Ok(())
    }
}
