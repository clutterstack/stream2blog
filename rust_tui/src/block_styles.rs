use ratatui::{
    style::{Color, Style},
    widgets::{Block, Borders, Padding},
};

pub fn bordered() -> Block<'static> {
    Block::default().borders(Borders::ALL)
}

pub fn titled(title: impl Into<String>) -> Block<'static> {
    bordered().title(title.into())
}

pub fn content_block(title: impl Into<String>) -> Block<'static> {
    titled(title).padding(Padding::uniform(1))
}

pub fn styled_block(title: impl Into<String>, style: Style) -> Block<'static> {
    titled(title).style(style)
}

pub fn error_block(title: impl Into<String>) -> Block<'static> {
    styled_block(title, Style::default().fg(Color::Red))
}

pub fn warning_block(title: impl Into<String>) -> Block<'static> {
    styled_block(title, Style::default().fg(Color::Yellow))
}