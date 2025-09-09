mod api;
mod app;
mod block_styles;
mod clipboard_manager;
mod editor_views;
mod handlers;
mod image_clip;
mod image_preview;
mod key_handler;
mod models;
mod operations;
mod state;
mod text_editor;
mod thread_view;
mod ui;
mod ui_utils;
mod widgets;

use app::App;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    log::info!("Starting stream2blog TUI");

    let mut app = App::new("http://localhost:4001");
    app.run().await?;
    Ok(())
}
