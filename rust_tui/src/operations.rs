use crate::app::App;

impl App {
    pub async fn load_threads(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        log::debug!("Loading threads from API");
        match self.api_client.get_threads().await {
            Ok(threads) => {
                log::debug!("Successfully loaded {} threads", threads.len());
                self.threads = threads;
                // Reset selected index if it's out of bounds
                if self.selected_thread_index >= self.threads.len() {
                    self.selected_thread_index = 0;
                }
            }
            Err(e) => {
                // Handle API errors gracefully - continue with empty list
                log::error!("Failed to load threads: {e}");
                self.threads = Vec::new();
                self.selected_thread_index = 0;
            }
        }
        Ok(())
    }

    pub async fn load_thread(&mut self, thread_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        log::debug!("Loading thread: {thread_id}");
        
        match self.api_client.get_thread(thread_id).await {
            Ok(thread) => {
                self.current_thread = Some(thread);
                self.selected_entry_index = 0;
                self.preview_scroll_offset = 0;
                // Reset thread list state for new thread
                self.thread_list_state.select(Some(0));
                
                // Incrementally update thumbnails only for changed entries
                self.generate_entry_thumbnails().await;
            }
            Err(e) => {
                log::error!("Failed to load thread: {e}");
            }
        }
        Ok(())
    }

    pub async fn export_thread(
        &mut self,
        thread_id: &str,
    ) -> Result<String, Box<dyn std::error::Error>> {
        log::debug!("Exporting thread: {thread_id}");
        match self.api_client.export_thread(thread_id).await {
            Ok(markdown) => {
                log::debug!("Successfully exported thread to markdown");
                Ok(markdown)
            }
            Err(e) => {
                log::error!("Failed to export thread: {e}");
                Err(e.into())
            }
        }
    }
}
