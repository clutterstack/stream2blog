use arboard::Clipboard;
use std::sync::Mutex;

/// Centralized clipboard manager to prevent race conditions from multiple clipboard instances
pub struct ClipboardManager {
    clipboard: Mutex<Clipboard>,
}

impl ClipboardManager {
    pub fn new() -> Result<Self, Box<dyn std::error::Error + Send + Sync + 'static>> {
        Ok(Self {
            clipboard: Mutex::new(Clipboard::new()?),
        })
    }

    pub fn get_text(&self) -> Result<String, Box<dyn std::error::Error + Send + Sync + 'static>> {
        let mut clipboard = self.clipboard.lock().map_err(|_| "Clipboard lock poisoned")?;
        Ok(clipboard.get_text()?)
    }

    pub fn set_text(&self, text: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
        let mut clipboard = self.clipboard.lock().map_err(|_| "Clipboard lock poisoned")?;
        clipboard.set_text(text)?;
        Ok(())
    }

    pub fn get_image(&self) -> Result<arboard::ImageData<'_>, Box<dyn std::error::Error + Send + Sync + 'static>> {
        let mut clipboard = self.clipboard.lock().map_err(|_| "Clipboard lock poisoned")?;
        Ok(clipboard.get_image()?)
    }
}

impl Default for ClipboardManager {
    fn default() -> Self {
        Self::new().expect("Failed to initialize clipboard manager")
    }
}

// Global singleton instance
static CLIPBOARD_MANAGER: std::sync::OnceLock<ClipboardManager> = std::sync::OnceLock::new();

pub fn get_clipboard_manager() -> &'static ClipboardManager {
    CLIPBOARD_MANAGER.get_or_init(|| {
        ClipboardManager::new().expect("Failed to initialize global clipboard manager")
    })
}