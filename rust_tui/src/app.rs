use crate::api::ApiClient;
use crate::image_preview::ImagePreview;
use crate::models::Thread;
use crate::state::AppState;
use crate::text_editor::TextEditor;
use ratatui::crossterm::event::{self, Event, poll};
use ratatui::{backend::CrosstermBackend, Terminal};
use ratatui::widgets::ListState;
use std::io;
use std::time::{Duration, Instant};

pub struct App {
    pub api_client: ApiClient,
    pub state: AppState,
    pub threads: Vec<Thread>,
    pub current_thread: Option<Thread>,
    pub text_editor: TextEditor,
    pub should_quit: bool,
    pub selected_thread_index: usize,
    pub selected_entry_index: usize,
    pub submit_button_area: Option<ratatui::layout::Rect>,
    pub entry_list_area: Option<ratatui::layout::Rect>,
    pub entry_positions: Vec<ratatui::layout::Rect>, // Track individual entry positions for mouse selection
    pub thread_list_area: Option<ratatui::layout::Rect>, // Track thread list area for mouse clicks
    pub thread_positions: Vec<ratatui::layout::Rect>, // Track individual thread positions for mouse selection
    pub last_scroll_time: Instant,
    pub scroll_throttle_ms: u64,
    pub preview_scroll_offset: u16,
    pub needs_redraw: bool,
    pub saved_text_content: Option<String>, // For storing text editor content during modal states
    pub modal_text_editor: TextEditor,      // Separate text editor for modal input
    pub thread_view_image_preview_visible: bool, // Toggle for ThreadView image preview panel
    pub entry_thumbnails: std::collections::HashMap<String, ImagePreview>, // Cached thumbnails by entry_id
    pub current_entry_image_path: Option<String>, // Track image path for current entry
    pub original_entry_content: Option<String>, // Store original content when editing entry
    pub wrap_width: usize, // Centralized wrap width for consistent text wrapping
    pub last_resize_time: Option<Instant>, // Track last resize event for debouncing
    pub resize_debounce_ms: u64, // Debounce duration in milliseconds
    pub thread_list_state: ListState, // Persistent scroll state for thread view
    pub main_thread_list_state: ListState, // Persistent scroll state for main thread list
}

impl App {
    pub fn new(api_url: &str) -> Self {
        

        // Note: Backups are now handled via API calls when data is saved
        
        Self {
            api_client: ApiClient::new(api_url),
            state: AppState::ThreadList,
            threads: Vec::new(),
            current_thread: None,
            text_editor: TextEditor::new(),
            should_quit: false,
            selected_thread_index: 0,
            selected_entry_index: 0,
            submit_button_area: None,
            entry_list_area: None,
            entry_positions: Vec::new(),
            thread_list_area: None,
            thread_positions: Vec::new(),
            last_scroll_time: Instant::now(),
            scroll_throttle_ms: 150, // 150ms throttle between scroll actions
            preview_scroll_offset: 0,
            needs_redraw: true,
            saved_text_content: None,
            modal_text_editor: TextEditor::new(),
            thread_view_image_preview_visible: true,
            entry_thumbnails: std::collections::HashMap::new(),
            current_entry_image_path: None,
            original_entry_content: None,
            wrap_width: 80, // Default wrap width, will be updated based on terminal size
            last_resize_time: None,
            resize_debounce_ms: 100, // 100ms debounce for resize events
            thread_list_state: ListState::default(),
            main_thread_list_state: ListState::default(),
        }
    }

    pub fn mark_dirty(&mut self) {
        self.needs_redraw = true;
    }


    /// Calculate wrap width based on terminal size
    pub fn calculate_wrap_width(&mut self, terminal_width: u16) {
        // Use consistent padding of 12 characters for borders and margins
        self.wrap_width = (terminal_width.saturating_sub(12) as usize).max(20); // Minimum width of 20
    }

    /// Calculate word count statistics for a thread
    pub fn calculate_thread_word_count(&self, thread: &Thread) -> (usize, usize, f32) {
        let total_words = thread.entries.iter()
            .map(|entry| {
                entry.content
                    .split_whitespace()
                    .filter(|word| !word.is_empty())
                    .count()
            })
            .sum();
        
        let entry_count = thread.entries.len();
        let avg_words_per_entry = if entry_count > 0 {
            total_words as f32 / entry_count as f32
        } else {
            0.0
        };
        
        (total_words, entry_count, avg_words_per_entry)
    }

    pub fn resolve_image_path(&self, image_path: &str) -> Option<String> {
        let path = std::path::Path::new(image_path);

        // Try the path as-is first
        if path.exists() {
            return Some(image_path.to_string());
        }

        // Try as relative to current directory
        if let Ok(current_dir) = std::env::current_dir() {
            let full_path = current_dir.join(image_path);
            if full_path.exists() {
                return Some(full_path.to_string_lossy().to_string());
            }

            // If it's just a filename, search in organized directories
            if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
                let search_paths = [
                    current_dir.join("images/global").join(filename),
                    current_dir.join(filename), // Legacy
                ];

                for search_path in &search_paths {
                    if search_path.exists() {
                        return Some(search_path.to_string_lossy().to_string());
                    }
                }
            }
        }

        None
    }

    pub async fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Setup terminal
        ratatui::crossterm::terminal::enable_raw_mode()?;
        let mut stdout = io::stdout();
        ratatui::crossterm::execute!(
            stdout,
            ratatui::crossterm::terminal::EnterAlternateScreen,
            ratatui::crossterm::event::EnableMouseCapture
        )?;

        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;
        log::info!("Terminal created, calling show_cursor()");
        terminal.show_cursor()?;
        log::info!("show_cursor() called successfully");

        // Initialize wrap width based on terminal size
        let terminal_size = terminal.size()?;
        self.calculate_wrap_width(terminal_size.width);

        // Load initial data
        self.load_threads().await?;

        // Main loop
        while !self.should_quit {
            if self.needs_redraw {
                terminal.draw(|f| {
                    self.draw(f);
                })?;
                self.needs_redraw = false;
            }

            // Check for resize debounce timeout
            let timeout_duration = if let Some(last_resize) = self.last_resize_time {
                let elapsed = last_resize.elapsed();
                let debounce_duration = Duration::from_millis(self.resize_debounce_ms);
                if elapsed >= debounce_duration {
                    // Debounce period elapsed, force redraw and clear resize time
                    terminal.draw(|f| {
                        self.draw(f);
                    })?;
                    self.last_resize_time = None;
                    if self.needs_redraw {
                        Duration::from_millis(16) // Fast polling when redraw needed (~60fps)
                    } else {
                        Duration::from_millis(100) // Normal polling interval
                    }
                } else {
                    // Wait for remaining debounce time
                    debounce_duration - elapsed
                }
            } else if self.needs_redraw {
                Duration::from_millis(16) // Fast polling when redraw needed (~60fps)
            } else {
                Duration::from_millis(100) // Normal polling interval
            };

            if poll(timeout_duration)? {
                match event::read()? {
                Event::Key(key) => {
                    self.handle_key_event(key).await?;
                }
                Event::Mouse(mouse) => {
                    self.handle_mouse(mouse).await?;
                }
                Event::Resize(width, _) => {
                    // Terminal was resized - use debouncing to avoid excessive redraws
                    let now = Instant::now();
                    self.last_resize_time = Some(now);
                    
                    // Update wrap width immediately to prepare for eventual redraw
                    self.calculate_wrap_width(width);
                    self.text_editor.set_wrap_width(self.wrap_width);
                    
                    // Mark dirty for eventual redraw
                    self.mark_dirty();
                }
                _ => {}
                }
            }
        }

        // Restore terminal
        log::info!("Restoring terminal, calling show_cursor() before cleanup");
        terminal.show_cursor()?;
        ratatui::crossterm::terminal::disable_raw_mode()?;
        ratatui::crossterm::execute!(
            terminal.backend_mut(),
            ratatui::crossterm::terminal::LeaveAlternateScreen,
            ratatui::crossterm::event::DisableMouseCapture
        )?;
        log::info!("Terminal cleanup completed");

        Ok(())
    }

    /// Pre-generates and caches thumbnails for all entries with images in the current thread.
    /// This is called unconditionally when entering ThreadView to ensure instant preview toggle.
    pub async fn generate_entry_thumbnails(&mut self) {
        if let Some(thread) = &self.current_thread {
            log::debug!("Pre-generating thumbnails for {} entries", thread.entries.len());
            
            for entry in &thread.entries {
                // Skip if we already have a cached thumbnail for this entry
                if self.entry_thumbnails.contains_key(&entry.id) {
                    continue;
                }
                
                if let Some(image_path) = &entry.image_path {
                    log::debug!("Entry {} has image_path: {}", entry.id, image_path);
                    if let Some(resolved_path) = self.resolve_image_path(image_path) {
                        log::debug!("Resolved image path for entry {}: {}", entry.id, resolved_path);
                        let mut thumbnail = ImagePreview::new();
                        if let Ok(()) = thumbnail.load_image(&resolved_path) {
                            log::debug!("Pre-cached thumbnail for entry {}", entry.id);
                            self.entry_thumbnails.insert(entry.id.clone(), thumbnail);
                        } else {
                            log::warn!("Failed to generate thumbnail for entry {} at path {}", entry.id, resolved_path);
                        }
                    } else {
                        log::warn!("Could not resolve image path for entry {}: {}", entry.id, image_path);
                    }
                } else {
                    log::debug!("Entry {} has no image_path", entry.id);
                }
            }
        }
    }


    pub fn clear_entry_thumbnails(&mut self) {
        log::debug!("Clearing {} cached thumbnails", self.entry_thumbnails.len());
        self.entry_thumbnails.clear();
    }

    /// Loads a cached thumbnail into the text editor for use in EditEntry state.
    /// This avoids re-loading and re-processing images that are already cached.
    pub fn load_cached_thumbnail_into_editor(&mut self, entry_id: &str) {
        if let Some(cached_thumbnail) = self.entry_thumbnails.get(entry_id) {
            // Copy the cached image data to the text editor's image preview
            if let Some(cached_image) = &cached_thumbnail.cached_image {
                if let Some(cached_picker) = &cached_thumbnail.cached_picker {
                    log::debug!("Loading cached thumbnail for entry {entry_id} into text editor");
                    
                    // Create a new ImagePreview for the text editor using the cached data
                    let text_editor_preview = self.text_editor.image_preview_mut();
                    text_editor_preview.cached_image = Some(cached_image.clone());
                    text_editor_preview.cached_picker = Some(*cached_picker);
                    
                    // Create the protocol from the cached image
                    text_editor_preview.create_fixed_protocol_from_cached();
                    text_editor_preview.set_visible(true);
                    
                    // Also ensure the text editor's show_image_preview flag is set
                    self.text_editor.set_image_preview_visible(true);
                    
                    log::debug!("Successfully loaded cached thumbnail into text editor");
                } else {
                    log::warn!("Cached thumbnail missing picker for entry {entry_id}");
                }
            } else {
                log::warn!("Cached thumbnail missing image data for entry {entry_id}");
            }
        } else {
            log::debug!("No cached thumbnail found for entry {entry_id}");
        }
    }

    /// Finds the index of a thread in the threads vector by its ID
    pub fn find_thread_index_by_id(&self, thread_id: &str) -> Option<usize> {
        self.threads.iter().position(|thread| thread.id == thread_id)
    }

    /// Finds the index of an entry in the current thread by its ID
    pub fn find_entry_index_by_id(&self, entry_id: &str) -> Option<usize> {
        self.current_thread.as_ref()?.entries.iter().position(|entry| entry.id == entry_id)
    }

    pub fn calculate_thread_positions(&mut self, list_area: ratatui::layout::Rect) {
        self.thread_positions.clear();
        
        // Get the current scroll offset from ListState
        let scroll_offset = self.main_thread_list_state.offset();
        
        // Account for List widget borders and padding
        // List widget has 1 unit border on all sides
        let content_x = list_area.x + 1;
        let content_y = list_area.y + 1; // +1 for border
        let content_width = list_area.width.saturating_sub(2); // -2 for left and right borders
        
        // Calculate visible area height for bounds checking
        let visible_height = list_area.height.saturating_sub(2); // -2 for borders
        
        // Each list item takes exactly 1 row
        for (index, _thread) in self.threads.iter().enumerate() {
            // Calculate the screen position accounting for scroll offset
            let screen_row = if index >= scroll_offset {
                content_y + (index - scroll_offset) as u16
            } else {
                // Thread is scrolled out of view above - set to 0 to mark as invisible
                0
            };
            
            let thread_rect = ratatui::layout::Rect {
                x: content_x,
                y: screen_row,
                width: content_width,
                height: 1,
            };
            
            // Only add positions that are visible within the list area
            if index >= scroll_offset && screen_row < content_y + visible_height {
                self.thread_positions.push(thread_rect);
            } else {
                // Thread is not visible (either scrolled out or beyond visible area)
                self.thread_positions.push(ratatui::layout::Rect {
                    x: 0,
                    y: 0,
                    width: 0,
                    height: 0,
                });
            }
        }
    }

    pub fn get_clicked_thread_index(&self, column: u16, row: u16) -> Option<usize> {
        for (index, rect) in self.thread_positions.iter().enumerate() {
            if rect.width > 0 && rect.height > 0 {  // Only check visible threads
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

    /// Creates a backup via the API
    pub async fn create_backup(&self) -> Result<(), Box<dyn std::error::Error>> {
        let client = reqwest::Client::new();
        let response = client
            .post("http://localhost:4001/api/admin/backup")
            .header("Content-Type", "application/json")
            .send()
            .await?;

        if response.status().is_success() {
            log::info!("Backup created successfully via API");
            Ok(())
        } else {
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            Err(format!("Backup API call failed: {error_text}").into())
        }
    }
}
